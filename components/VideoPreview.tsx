"use client";
import React, { useEffect, useRef, useState } from 'react';

export function VideoPreview() {
    const videoRef = useRef<HTMLVideoElement>(null);
    const [streamKey, setStreamKey] = useState<string | null>(null);
    const [isStreaming, setIsStreaming] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const playerRef = useRef<any>(null);
    const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null);

    useEffect(() => {
        // Check for stream key
        const checkStreamKey = async () => {
            try {
                const response = await fetch('/api/stream-key');
                const data = await response.json();
                if (data.streamKey) {
                    setStreamKey(data.streamKey);
                }
            } catch (err) {
                console.error('Failed to get stream key:', err);
            }
        };

        checkStreamKey();
        const interval = setInterval(checkStreamKey, 3000);
        return () => clearInterval(interval);
    }, []);

    useEffect(() => {
        if (!streamKey || !videoRef.current) return;

        const httpPort = parseInt(process.env.NEXT_PUBLIC_HTTP_FLV_PORT || '8888');
        const flvUrl = `http://localhost:${httpPort}/live/${streamKey}.flv`;

        const initPlayer = async () => {
            try {
                const mpegts = (await import('mpegts.js')).default;

                if (mpegts.getFeatureList().mseLivePlayback) {
                    if (playerRef.current) {
                        playerRef.current.destroy();
                        playerRef.current = null;
                    }

                    const player = mpegts.createPlayer({
                        type: 'flv',
                        isLive: true,
                        url: flvUrl,
                        hasAudio: true,
                    }, {
                        enableWorker: false,
                        lazyLoad: false,
                        lazyLoadMaxDuration: 0,
                        seekType: 'range',
                        enableStashBuffer: false,
                        liveBufferLatencyChasing: true,
                        liveBufferLatencyMaxLatency: 2.0,
                        liveBufferLatencyMinRemain: 0.5,
                    });

                    playerRef.current = player;
                    player.attachMediaElement(videoRef.current!);
                    
                    player.on(mpegts.Events.ERROR, (type: any, details: any, data: any) => {
                        console.log('[VideoPreview] Player Error:', type, details, data);
                        if (type === mpegts.ErrorTypes.NETWORK_ERROR) {
                            setIsStreaming(false);
                            setError('接続待機中...');
                            
                            // Auto-reconnect after 2 seconds
                            if (reconnectTimeoutRef.current) {
                                clearTimeout(reconnectTimeoutRef.current);
                            }
                            reconnectTimeoutRef.current = setTimeout(() => {
                                console.log('[VideoPreview] Attempting to reconnect...');
                                if (playerRef.current) {
                                    playerRef.current.unload();
                                    playerRef.current.load();
                                }
                            }, 2000);
                        }
                    });

                    player.on(mpegts.Events.LOADING_COMPLETE, () => {
                        console.log('[VideoPreview] Loading complete');
                    });

                    player.on(mpegts.Events.METADATA_ARRIVED, () => {
                        console.log('[VideoPreview] Metadata arrived');
                        if (!isStreaming) {
                            setIsStreaming(true);
                            setError(null);
                        }
                    });

                    player.on(mpegts.Events.STATISTICS_INFO, () => {
                        if (!isStreaming) {
                            setIsStreaming(true);
                            setError(null);
                        }
                    });

                    // Start loading and playing
                    player.load();
                    player.play().catch(e => {
                        console.log('[VideoPreview] Autoplay prevented:', e);
                        // Try to play when user interacts
                        videoRef.current?.addEventListener('click', () => {
                            player.play().catch(console.error);
                        }, { once: true });
                    });
                }
            } catch (err) {
                console.error('Failed to initialize player:', err);
                setError('プレイヤーの初期化に失敗しました');
            }
        };

        initPlayer();

        return () => {
            if (reconnectTimeoutRef.current) {
                clearTimeout(reconnectTimeoutRef.current);
            }
            if (playerRef.current) {
                playerRef.current.pause();
                playerRef.current.unload();
                playerRef.current.detachMediaElement();
                playerRef.current.destroy();
                playerRef.current = null;
            }
        };
    }, [streamKey]);

    return (
        <div className="w-full h-full bg-zinc-900 flex flex-col items-center justify-center relative group overflow-hidden">
            {/* Grid Pattern Background */}
            <div className="absolute inset-0 opacity-10 pointer-events-none"
                style={{ backgroundImage: 'radial-gradient(circle, #ffffff 1px, transparent 1px)', backgroundSize: '24px 24px' }}>
            </div>

            {streamKey ? (
                <>
                    <video
                        ref={videoRef}
                        className="w-full h-full object-contain"
                        muted
                        playsInline
                        autoPlay
                    />
                    {!isStreaming && (
                        <div className="absolute inset-0 flex items-center justify-center pointer-events-none">
                            <div className="text-center space-y-4">
                                <div className="w-20 h-20 rounded-full border-2 border-zinc-700 flex items-center justify-center mx-auto text-zinc-600 animate-pulse">
                                    <span className="text-2xl">📡</span>
                                </div>
                                <div className="space-y-1">
                                    <h3 className="text-zinc-400 font-medium">ストリーム待機中</h3>
                                    <p className="text-zinc-600 text-sm">
                                        {error || 'OBSから配信を開始してください'}
                                    </p>
                                </div>
                            </div>
                        </div>
                    )}
                    {isStreaming && (
                        <div className="absolute top-4 right-4 bg-red-600 px-3 py-1 rounded-full text-xs font-medium flex items-center gap-2 z-10">
                            <span className="w-2 h-2 bg-white rounded-full animate-pulse"></span>
                            LIVE
                        </div>
                    )}
                </>
            ) : (
                <div className="z-10 text-center space-y-4">
                    <div className="w-20 h-20 rounded-full border-2 border-zinc-700 flex items-center justify-center mx-auto text-zinc-600">
                        <span className="text-2xl">📷</span>
                    </div>
                    <div className="space-y-1">
                        <h3 className="text-zinc-400 font-medium">接続待機中</h3>
                        <p className="text-zinc-600 text-sm">ストリームキーを生成してください</p>
                    </div>
                </div>
            )}

            <div className="absolute bottom-4 left-4 bg-black/60 backdrop-blur px-2 py-1 rounded text-xs font-mono text-zinc-400 z-10">
                {isStreaming ? 'STREAMING' : 'OFFLINE'}
            </div>
        </div>
    );
}
