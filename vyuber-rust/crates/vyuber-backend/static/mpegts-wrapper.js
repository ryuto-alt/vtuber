// mpegts.js wrapper for WASM integration
(function() {
    let player = null;
    let retryTimer = null;
    const MAX_PLAYBACK_ERRORS = 30;
    const RETRY_INTERVAL = 3000;

    window.initMpegtsPlayer = function(videoElementId, streamUrl) {
        if (typeof mpegts === 'undefined') {
            console.error('mpegts.js not loaded');
            return false;
        }

        if (!mpegts.getFeatureList().mseLivePlayback) {
            console.error('MSE LivePlayback not supported');
            return false;
        }

        startWithRetry(videoElementId, streamUrl, 0);
        return true;
    };

    function startWithRetry(videoElementId, streamUrl, playbackErrors) {
        if (playbackErrors >= MAX_PLAYBACK_ERRORS) {
            console.warn('Max playback errors reached, stopping retries');
            return;
        }

        var videoElement = document.getElementById(videoElementId);
        if (!videoElement) {
            console.error('Video element not found:', videoElementId);
            return;
        }

        if (player) {
            try { player.destroy(); } catch(e) {}
            player = null;
        }

        // 相対URLをWorker内でも使えるよう絶対URLに変換
        var absoluteUrl = streamUrl;
        if (streamUrl.startsWith('/')) {
            absoluteUrl = window.location.origin + streamUrl;
        }
        console.log('Creating mpegts player (type: flv, url: ' + absoluteUrl + ', errors: ' + playbackErrors + ')');

        // ミュート状態で初期化（autoplay policy対策）
        videoElement.muted = true;

        player = mpegts.createPlayer({
            type: 'flv',
            url: absoluteUrl,
            isLive: true,
            cors: true,
        }, {
            enableWorker: true,
            enableStashBuffer: false,
            stashInitialSize: 128,
            autoCleanupSourceBuffer: true,
            autoCleanupMaxBackwardDuration: 5,
            autoCleanupMinBackwardDuration: 2,
            liveBufferLatencyChasing: true,
            liveBufferLatencyMaxLatency: 1.5,
            liveBufferLatencyMinRemain: 0.3,
            liveSyncTargetLatency: 0.8,
        });

        player.attachMediaElement(videoElement);
        player.load();
        // play()はload()直後に呼ばない。MEDIA_INFOイベントでデータ到着を確認してから再生開始

        // メディア情報受信 → プレースホルダー非表示 + 再生開始
        player.on(mpegts.Events.MEDIA_INFO, function(info) {
            console.log('Media info received, starting playback:', info.mimeType);
            // プレースホルダーをすぐ非表示
            var placeholder = document.getElementById('video-placeholder');
            if (placeholder) {
                placeholder.style.display = 'none';
                console.log('Placeholder hidden');
            }
            var p = videoElement.play();
            if (p && p.then) {
                p.then(function() {
                    console.log('play() succeeded');
                }).catch(function(e) {
                    console.warn('play() failed:', e.message);
                });
            }
        });

        // バッファ状態を定期的にログ
        var debugInterval = setInterval(function() {
            if (!videoElement || !player) { clearInterval(debugInterval); return; }
            var buffered = videoElement.buffered;
            var bufStr = '';
            for (var i = 0; i < buffered.length; i++) {
                bufStr += '[' + buffered.start(i).toFixed(2) + '-' + buffered.end(i).toFixed(2) + '] ';
            }
            console.log('Video state: readyState=' + videoElement.readyState +
                ' paused=' + videoElement.paused +
                ' currentTime=' + videoElement.currentTime.toFixed(2) +
                ' buffered=' + bufStr +
                ' videoWidth=' + videoElement.videoWidth +
                ' error=' + (videoElement.error ? videoElement.error.message : 'none'));
        }, 2000);

        videoElement.addEventListener('playing', function onPlaying() {
            console.log('Video is playing!');
            videoElement.removeEventListener('playing', onPlaying);
        });

        player.on(mpegts.Events.ERROR, function(errorType, errorDetail, errorInfo) {
            console.warn('mpegts.js error:', errorType, errorDetail, JSON.stringify(errorInfo));
            if (errorInfo && errorInfo.code && (errorInfo.code === 503 || errorInfo.code === 404)) {
                console.log('Stream not ready (HTTP ' + errorInfo.code + '), retrying without penalty...');
                scheduleRetry(videoElementId, streamUrl, playbackErrors);
            } else {
                scheduleRetry(videoElementId, streamUrl, playbackErrors + 1);
            }
        });

        player.on(mpegts.Events.LOADING_COMPLETE, function() {
            console.log('Stream loading complete, retrying...');
            scheduleRetry(videoElementId, streamUrl, 0);
        });

        console.log('mpegts player initialized for:', absoluteUrl);
    }

    function scheduleRetry(videoElementId, streamUrl, nextErrors) {
        if (retryTimer) clearTimeout(retryTimer);
        retryTimer = setTimeout(function() {
            console.log('Retrying stream connection (errors: ' + nextErrors + ')...');
            startWithRetry(videoElementId, streamUrl, nextErrors);
        }, RETRY_INTERVAL);
    }

    window.destroyMpegtsPlayer = function() {
        if (retryTimer) {
            clearTimeout(retryTimer);
            retryTimer = null;
        }
        if (player) {
            try { player.destroy(); } catch(e) {}
            player = null;
            var placeholder = document.getElementById('video-placeholder');
            if (placeholder) placeholder.style.display = '';
            console.log('mpegts player destroyed');
        }
    };
})();
