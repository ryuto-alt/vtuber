// mpegts.js wrapper for WASM integration
// This script expects mpegts to be loaded from CDN first

(function() {
    let player = null;

    window.initMpegtsPlayer = function(videoElementId, streamUrl) {
        if (typeof mpegts === 'undefined') {
            console.error('mpegts.js not loaded');
            return false;
        }

        if (mpegts.getFeatureList().mseLivePlayback) {
            const videoElement = document.getElementById(videoElementId);

            if (!videoElement) {
                console.error('Video element not found:', videoElementId);
                return false;
            }

            if (player) {
                player.destroy();
            }

            player = mpegts.createPlayer({
                type: 'flv',
                url: streamUrl,
                isLive: true,
                cors: true,
                enableWorker: true,
                enableStashBuffer: false,
                stashInitialSize: 128,
                liveBufferLatencyChasing: true,
                liveBufferLatencyMaxLatency: 3,
                liveBufferLatencyMinRemain: 0.5,
            });

            player.attachMediaElement(videoElement);
            player.load();
            player.play();

            player.on(mpegts.Events.ERROR, function(errorType, errorDetail) {
                console.error('mpegts.js error:', errorType, errorDetail);
            });

            console.log('mpegts player initialized for:', streamUrl);
            return true;
        } else {
            console.error('MSE LivePlayback not supported');
            return false;
        }
    };

    window.destroyMpegtsPlayer = function() {
        if (player) {
            player.destroy();
            player = null;
            console.log('mpegts player destroyed');
        }
    };
})();
