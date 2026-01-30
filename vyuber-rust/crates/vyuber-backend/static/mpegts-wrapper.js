// mpegts.js wrapper for WASM integration
(function() {
    let player = null;
    let retryTimer = null;
    const MAX_PLAYBACK_ERRORS = 30;
    const RETRY_INTERVAL = 3000;

    window.initMpegtsPlayer = function(videoElementId, streamUrl) {
        if (typeof mpegts === 'undefined') return false;
        if (!mpegts.getFeatureList().mseLivePlayback) return false;
        startWithRetry(videoElementId, streamUrl, 0);
        return true;
    };

    function startWithRetry(videoElementId, streamUrl, playbackErrors) {
        if (playbackErrors >= MAX_PLAYBACK_ERRORS) return;

        var videoElement = document.getElementById(videoElementId);
        if (!videoElement) return;

        if (player) {
            try { player.destroy(); } catch(e) {}
            player = null;
        }

        var absoluteUrl = streamUrl;
        if (streamUrl.startsWith('/')) {
            absoluteUrl = window.location.origin + streamUrl;
        }

        videoElement.muted = true;

        player = mpegts.createPlayer({
            type: 'flv',
            url: absoluteUrl,
            isLive: true,
            cors: true,
        }, {
            enableWorker: true,
            enableStashBuffer: false,
            stashInitialSize: 64,
            autoCleanupSourceBuffer: true,
            autoCleanupMaxBackwardDuration: 3,
            autoCleanupMinBackwardDuration: 1,
            liveBufferLatencyChasing: true,
            liveBufferLatencyMaxLatency: 1.0,
            liveBufferLatencyMinRemain: 0.2,
            liveSyncTargetLatency: 0.5,
            liveSyncPlaybackRate: 1.2,
            fixAudioTimestampGap: false,
        });

        player.attachMediaElement(videoElement);
        player.load();

        player.on(mpegts.Events.MEDIA_INFO, function() {
            var placeholder = document.getElementById('video-placeholder');
            if (placeholder) placeholder.style.display = 'none';
            var p = videoElement.play();
            if (p && p.catch) p.catch(function() {});
        });

        player.on(mpegts.Events.ERROR, function(errorType, errorDetail, errorInfo) {
            if (errorInfo && errorInfo.code && (errorInfo.code === 503 || errorInfo.code === 404)) {
                scheduleRetry(videoElementId, streamUrl, playbackErrors);
            } else {
                scheduleRetry(videoElementId, streamUrl, playbackErrors + 1);
            }
        });

        player.on(mpegts.Events.LOADING_COMPLETE, function() {
            scheduleRetry(videoElementId, streamUrl, 0);
        });
    }

    function scheduleRetry(videoElementId, streamUrl, nextErrors) {
        if (retryTimer) clearTimeout(retryTimer);
        retryTimer = setTimeout(function() {
            startWithRetry(videoElementId, streamUrl, nextErrors);
        }, RETRY_INTERVAL);
    }

    window.destroyMpegtsPlayer = function() {
        if (retryTimer) { clearTimeout(retryTimer); retryTimer = null; }
        if (player) {
            try { player.destroy(); } catch(e) {}
            player = null;
            var placeholder = document.getElementById('video-placeholder');
            if (placeholder) placeholder.style.display = '';
        }
    };
})();
