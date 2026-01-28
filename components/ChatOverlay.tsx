"use client";

import React, { useEffect, useRef } from 'react';
import { cn } from '@/lib/utils';

// 型定義
export interface ChatMessage {
    id: number;
    user: string;
    text: string;
    color: string;
}

interface ChatOverlayProps {
    transparent?: boolean;
    messages: ChatMessage[]; // ★ここを変更：親からデータをもらう
}

export function ChatOverlay({ transparent = false, messages }: ChatOverlayProps) {
    const scrollRef = useRef<HTMLDivElement>(null);

    // メッセージが増えたら自動スクロール
    useEffect(() => {
        if (scrollRef.current) {
            scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
        }
    }, [messages]);

    return (
        <div className={cn(
            "flex flex-col h-full",
            transparent ? "bg-transparent" : "bg-zinc-950/80 backdrop-blur-sm"
        )}>
            {!transparent && (
                <div className="h-10 border-b border-zinc-800 flex items-center px-4 bg-zinc-950 shrink-0">
                    <span className="text-xs font-semibold text-zinc-400 uppercase tracking-wider">チャット (LIVE)</span>
                </div>
            )}

            <div 
                ref={scrollRef}
                className={cn(
                "flex-1 overflow-y-auto space-y-3 font-sans text-[13px]",
                transparent ? "p-4 scrollbar-hide" : "p-4"
            )}>
                {messages.map((msg) => (
                    <div key={msg.id} className={cn(
                        "animate-in fade-in slide-in-from-left-2 duration-300",
                        transparent && "text-shadow-sm"
                    )}>
                        <span className={`font-bold ${msg.color} mr-2`}>{msg.user}</span>
                        <span className={cn(
                            "leading-relaxed",
                            transparent ? "text-white drop-shadow-md font-medium" : "text-zinc-300"
                        )}>{msg.text}</span>
                    </div>
                ))}
            </div>
            
            {/* "AI待機中" の表示部分はそのまま... */}
             {!transparent && (
                <div className="h-12 border-t border-zinc-800 bg-zinc-950 px-3 flex items-center shrink-0">
                    <span className="text-xs text-zinc-600 block w-full text-center italic">
                         Waiting for voice input...
                    </span>
                </div>
            )}
        </div>
    );
}