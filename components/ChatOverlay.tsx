"use client";

import React, { useEffect, useRef, useState } from 'react';
import { cn } from '@/lib/utils';

interface Message {
    id: number;
    user: string;
    text: string;
    color: string;
}

interface ChatOverlayProps {
    transparent?: boolean;
    isStreaming?: boolean;
}

export function ChatOverlay({ transparent = false, isStreaming = false }: ChatOverlayProps) {
    const [messages, setMessages] = useState<Message[]>([]);
    const messagesEndRef = useRef<HTMLDivElement>(null);

    // Auto-scroll to bottom
    useEffect(() => {
        messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
    }, [messages]);

    useEffect(() => {
        let timeoutId: NodeJS.Timeout;
        let isCancelled = false;

        const fetchChat = async () => {
            if (!isStreaming || isCancelled) return;

            try {
                const res = await fetch('/api/chat', { method: 'POST' });
                if (res.ok) {
                    const data = await res.json();

                    // Add new message
                    setMessages(prev => {
                        // Keep last 50 messages to prevent memory issues
                        const newMsgs = [...prev, {
                            id: Date.now(),
                            user: data.user || 'Viewer',
                            text: data.text || '...',
                            color: data.color || 'text-zinc-300'
                        }];
                        return newMsgs.slice(-50);
                    });
                }
            } catch (err) {
                console.error("[Chat] Error fetching:", err);
            }

            // Schedule next message (random 1-5 seconds)
            if (!isCancelled && isStreaming) {
                const delay = Math.floor(Math.random() * 4000) + 1000; // 1000ms to 5000ms
                timeoutId = setTimeout(fetchChat, delay);
            }
        };

        if (isStreaming) {
            // Start loop immediately
            fetchChat();
        } else {
            // Clear messages when stream stops? Or keep them? 
            // User said "before live stream demo chat remove".
            // Assuming we clear or just don't show new ones.
            // Let's clear to be clean based on "remove demo chat".
            setMessages([]);
        }

        return () => {
            isCancelled = true;
            clearTimeout(timeoutId);
        };
    }, [isStreaming]);

    return (
        <div className={cn(
            "flex flex-col h-full",
            transparent ? "bg-transparent text-shadow-md" : "bg-zinc-950/80 backdrop-blur-sm"
        )}>
            {!transparent && (
                <div className="h-10 border-b border-zinc-800 flex items-center px-4 bg-zinc-950">
                    <span className="text-xs font-semibold text-zinc-400 uppercase tracking-wider">
                        {isStreaming ? '🔴 チャット (LIVE)' : 'チャット (OFFLINE)'}
                    </span>
                </div>
            )}

            <div className={cn(
                "flex-1 overflow-y-auto space-y-3 font-sans text-[13px]",
                transparent ? "p-4 scrollbar-hide" : "p-4"
            )}>
                {messages.length === 0 && isStreaming && (
                    <div className="text-zinc-500 italic text-center text-xs mt-4">
                        チャット接続中...
                    </div>
                )}

                {messages.length === 0 && !isStreaming && (
                    <div className="text-zinc-600 italic text-center text-xs mt-4">
                        配信を開始するとチャットが表示されます
                    </div>
                )}

                {messages.map((msg) => (
                    <div key={msg.id} className={cn(
                        "animate-in fade-in slide-in-from-left-2 duration-300",
                        transparent && "text-shadow-sm"
                    )}>
                        <span className={`font-bold ${msg.color} mr-2`}>{msg.user}</span>
                        <span className={cn(
                            "leading-relaxed",
                            transparent ? "text-white font-medium" : "text-zinc-300"
                        )}>{msg.text}</span>
                    </div>
                ))}

                <div ref={messagesEndRef} />
            </div>

            {!transparent && (
                <div className="h-12 border-t border-zinc-800 bg-zinc-950 px-3 flex items-center">
                    <span className="text-xs text-zinc-600 block w-full text-center italic">
                        {isStreaming ? 'AI視聴者がコメント中...' : '配信待機中'}
                    </span>
                </div>
            )}
        </div>
    );
}
