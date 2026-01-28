"use client";

import React, { useState, useEffect } from 'react';
import { Mic, MicOff, Video, VideoOff, Square, Circle, Key, Copy, Check } from 'lucide-react';
import { cn } from '@/lib/utils';

export function ControlBar() {
    const [isRecording, setIsRecording] = useState(false);
    const [isMicOn, setIsMicOn] = useState(true);
    const [isVideoOn, setIsVideoOn] = useState(true);
    const [streamKey, setStreamKey] = useState<string | null>(null);
    const [serverUrl, setServerUrl] = useState<string>('rtmp://localhost:1935/live');
    const [copied, setCopied] = useState<'key' | 'url' | null>(null);
    const [showStreamInfo, setShowStreamInfo] = useState(false);

    useEffect(() => {
        checkStreamKey();
    }, []);

    const checkStreamKey = async () => {
        try {
            const response = await fetch('/api/stream-key');
            const data = await response.json();
            if (data.streamKey) {
                setStreamKey(data.streamKey);
                setServerUrl(data.serverUrl || 'rtmp://localhost:1935/live');
            }
        } catch (error) {
            console.error('Failed to check stream key:', error);
        }
    };

    const generateStreamKey = async () => {
        try {
            const response = await fetch('/api/stream-key', {
                method: 'POST',
            });
            const data = await response.json();
            if (response.ok) {
                setStreamKey(data.streamKey);
                setServerUrl(data.serverUrl || 'rtmp://localhost:1935/live');
                setShowStreamInfo(true);
            }
        } catch (error) {
            console.error('Failed to generate stream key:', error);
            alert('ストリームキーの生成に失敗しました');
        }
    };

    const copyToClipboard = async (text: string, type: 'key' | 'url') => {
        try {
            await navigator.clipboard.writeText(text);
            setCopied(type);
            setTimeout(() => setCopied(null), 2000);
        } catch (error) {
            console.error('Failed to copy:', error);
        }
    };

    return (
        <div className="flex items-center justify-between w-full">
            <div className="flex items-center gap-6">
                <div className="flex items-center gap-2 mr-4">
                    <button
                        onClick={() => setIsMicOn(!isMicOn)}
                        className={cn(
                            "p-3 rounded-full transition-all duration-200 border",
                            isMicOn
                                ? "bg-zinc-800 border-zinc-700 text-zinc-100 hover:bg-zinc-700 hover:border-zinc-600"
                                : "bg-red-500/10 border-red-500/20 text-red-500 hover:bg-red-500/20"
                        )}
                    >
                        {isMicOn ? <Mic className="w-5 h-5" /> : <MicOff className="w-5 h-5" />}
                    </button>
                    <button
                        onClick={() => setIsVideoOn(!isVideoOn)}
                        className={cn(
                            "p-3 rounded-full transition-all duration-200 border",
                            isVideoOn
                                ? "bg-zinc-800 border-zinc-700 text-zinc-100 hover:bg-zinc-700 hover:border-zinc-600"
                                : "bg-red-500/10 border-red-500/20 text-red-500 hover:bg-red-500/20"
                        )}
                    >
                        {isVideoOn ? <Video className="w-5 h-5" /> : <VideoOff className="w-5 h-5" />}
                    </button>
                </div>

                <button
                    onClick={() => setIsRecording(!isRecording)}
                    className={cn(
                        "h-14 px-8 rounded-full flex items-center gap-3 font-semibold tracking-wide transition-all duration-300 shadow-lg hover:shadow-xl hover:scale-105 active:scale-95",
                        isRecording
                            ? "bg-red-500 hover:bg-red-600 text-white shadow-red-500/20"
                            : "bg-zinc-100 hover:bg-white text-zinc-950 shadow-zinc-500/10"
                    )}
                >
                    {isRecording ? (
                        <>
                            <Square className="w-5 h-5 fill-current" />
                            <span>配信停止</span>
                        </>
                    ) : (
                        <>
                            <div className="w-4 h-4 rounded-full bg-red-500 animate-pulse" />
                            <span>配信開始</span>
                        </>
                    )}
                </button>
            </div>

            <div className="flex items-center gap-3">
                <button
                    onClick={generateStreamKey}
                    className="h-10 px-6 rounded-lg bg-green-600 hover:bg-green-700 text-white font-medium transition-all duration-200 shadow-lg shadow-green-500/20 hover:shadow-xl hover:scale-105 flex items-center gap-2"
                >
                    <Key className="w-4 h-4" />
                    ストリームキー生成
                </button>

                {streamKey && (
                    <button
                        onClick={() => setShowStreamInfo(!showStreamInfo)}
                        className="h-10 px-6 rounded-lg bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 text-zinc-100 font-medium transition-all duration-200"
                    >
                        {showStreamInfo ? '接続情報を隠す' : '接続情報を表示'}
                    </button>
                )}
            </div>

            {showStreamInfo && streamKey && (
                <div className="absolute bottom-20 right-4 bg-zinc-900 border border-zinc-800 rounded-lg p-4 shadow-2xl w-96 z-50">
                    <h3 className="text-zinc-100 font-semibold mb-3">OBS配信設定</h3>
                    <div className="space-y-3">
                        <div>
                            <label className="text-xs text-zinc-500 mb-1 block">サーバーURL</label>
                            <div className="flex items-center gap-2">
                                <code className="flex-1 bg-zinc-950 border border-zinc-800 rounded px-3 py-2 text-sm text-zinc-300 font-mono">
                                    {serverUrl}
                                </code>
                                <button
                                    onClick={() => copyToClipboard(serverUrl, 'url')}
                                    className="p-2 rounded bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 text-zinc-300"
                                >
                                    {copied === 'url' ? <Check className="w-4 h-4" /> : <Copy className="w-4 h-4" />}
                                </button>
                            </div>
                        </div>
                        <div>
                            <label className="text-xs text-zinc-500 mb-1 block">ストリームキー</label>
                            <div className="flex items-center gap-2">
                                <code className="flex-1 bg-zinc-950 border border-zinc-800 rounded px-3 py-2 text-sm text-zinc-300 font-mono">
                                    {streamKey}
                                </code>
                                <button
                                    onClick={() => copyToClipboard(streamKey, 'key')}
                                    className="p-2 rounded bg-zinc-800 hover:bg-zinc-700 border border-zinc-700 text-zinc-300"
                                >
                                    {copied === 'key' ? <Check className="w-4 h-4" /> : <Copy className="w-4 h-4" />}
                                </button>
                            </div>
                        </div>
                        <div className="pt-2 border-t border-zinc-800">
                            <p className="text-xs text-zinc-500">
                                OBSの設定から「配信」を選択し、上記の情報を入力してください。
                            </p>
                        </div>
                    </div>
                </div>
            )}
        </div>
    );
}
