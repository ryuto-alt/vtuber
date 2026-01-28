"use client";

import React from 'react';
import { cn } from '@/lib/utils';

// Mock data for initial UI
const MOCK_MESSAGES = [
    { id: 1, user: "視聴者A", text: "配信画面、すごくきれいですね！", color: "text-blue-400" },
    { id: 2, user: "初見さん", text: "今日の予定はなんですか？", color: "text-purple-400" },
    { id: 3, user: "常連", text: "こんにちは！待ってました！", color: "text-green-400" },
    { id: 4, user: "ROM専", text: "音声クリアで聞きやすいです。", color: "text-orange-400" },
    { id: 5, user: "タヌキ", text: "シンプルで良いデザイン。", color: "text-pink-400" },
];

interface ChatOverlayProps {
    transparent?: boolean;
}

export function ChatOverlay({ transparent = false }: ChatOverlayProps) {
    return (
        <div className={cn(
            "flex flex-col h-full",
            transparent ? "bg-transparent" : "bg-zinc-950/80 backdrop-blur-sm"
        )}>
            {!transparent && (
                <div className="h-10 border-b border-zinc-800 flex items-center px-4 bg-zinc-950">
                    <span className="text-xs font-semibold text-zinc-400 uppercase tracking-wider">チャット (LIVE)</span>
                </div>
            )}

            <div className={cn(
                "flex-1 overflow-y-auto space-y-3 font-sans text-[13px]",
                transparent ? "p-4 scrollbar-hide" : "p-4"
            )}>
                {MOCK_MESSAGES.map((msg) => (
                    <div key={msg.id} className={cn(
                        "animate-in fade-in slide-in-from-left-2 duration-300",
                        transparent && "text-shadow-sm" // Add shadow for readability on overlay
                    )}>
                        <span className={`font-bold ${msg.color} mr-2`}>{msg.user}</span>
                        <span className={cn(
                            "leading-relaxed",
                            transparent ? "text-white drop-shadow-md font-medium" : "text-zinc-300"
                        )}>{msg.text}</span>
                    </div>
                ))}

                {/* Ghost element to show scrolling - Hide in transparent mode to keep it clean */}
                {!transparent && (
                    <div className="opacity-50">
                        <span className="font-bold text-zinc-500 mr-2">システム</span>
                        <span className="text-zinc-500 italic">チャット接続完了...</span>
                    </div>
                )}
            </div>

            {!transparent && (
                <div className="h-12 border-t border-zinc-800 bg-zinc-950 px-3 flex items-center">
                    <span className="text-xs text-zinc-600 block w-full text-center italic">
                        AIがあなたの声を待機中...
                    </span>
                </div>
            )}
        </div>
    );
}
