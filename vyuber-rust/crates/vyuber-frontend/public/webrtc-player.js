// WebRTC WHEP player for MediaMTX
(function() {
    let pc = null;
    let pollTimer = null;
    let active = false;

    window.initWebrtcPlayer = function(videoElementId) {
        console.log('[WebRTC] initWebrtcPlayer called for', videoElementId);
        stopPolling();
        poll(videoElementId);
        return true;
    };

    window.destroyWebrtcPlayer = function() {
        stopPolling();
        cleanup();
        var placeholder = document.getElementById('video-placeholder');
        if (placeholder) placeholder.style.display = '';
    };

    function poll(videoElementId) {
        checkAndConnect(videoElementId);
        pollTimer = setInterval(function() {
            checkAndConnect(videoElementId);
        }, 2000);
    }

    async function checkAndConnect(videoElementId) {
        try {
            var resp = await fetch('/api/live/status');
            var data = await resp.json();
            if (data.active && !active) {
                active = true;
                connect(videoElementId, '/api/live/whep');
            } else if (!data.active && active) {
                active = false;
                cleanup();
                var placeholder = document.getElementById('video-placeholder');
                if (placeholder) placeholder.style.display = '';
            }
        } catch(e) {}
    }

    function stopPolling() {
        if (pollTimer) { clearInterval(pollTimer); pollTimer = null; }
    }

    async function connect(videoElementId, whepUrl) {
        console.log('[WebRTC] Connecting...');
        cleanup();
        var videoElement = document.getElementById(videoElementId);
        if (!videoElement) return;

        try {
            pc = new RTCPeerConnection({ iceServers: [] });

            pc.addTransceiver('video', { direction: 'recvonly' });
            pc.addTransceiver('audio', { direction: 'recvonly' });

            // srcObjectは1回だけセット
            var streamSet = false;
            pc.ontrack = function(event) {
                console.log('[WebRTC] Got track:', event.track.kind);
                if (!streamSet) {
                    streamSet = true;
                    videoElement.srcObject = event.streams[0];
                    var placeholder = document.getElementById('video-placeholder');
                    if (placeholder) placeholder.style.display = 'none';
                    videoElement.play().catch(function() {});
                }
            };

            pc.oniceconnectionstatechange = function() {
                if (!pc) return;
                var state = pc.iceConnectionState;
                console.log('[WebRTC] ICE state:', state);
                // failedのみで切断。disconnectedは一時的なので無視
                if (state === 'failed') {
                    console.warn('[WebRTC] ICE failed, will retry');
                    active = false;
                    cleanup();
                    var placeholder = document.getElementById('video-placeholder');
                    if (placeholder) placeholder.style.display = '';
                }
            };

            var offer = await pc.createOffer();
            await pc.setLocalDescription(offer);

            var resp = await fetch(whepUrl, {
                method: 'POST',
                headers: { 'Content-Type': 'application/sdp' },
                body: pc.localDescription.sdp,
            });

            if (!resp.ok) {
                console.warn('[WebRTC] WHEP failed:', resp.status);
                active = false;
                cleanup();
                return;
            }

            var answerSdp = await resp.text();
            await pc.setRemoteDescription(new RTCSessionDescription({
                type: 'answer',
                sdp: answerSdp,
            }));
            console.log('[WebRTC] Connected');
        } catch(e) {
            console.warn('[WebRTC] Connection failed:', e);
            active = false;
            cleanup();
        }
    }

    function cleanup() {
        if (pc) {
            try { pc.close(); } catch(e) {}
            pc = null;
        }
        var video = document.getElementById('video-preview');
        if (video) video.srcObject = null;
    }
})();
