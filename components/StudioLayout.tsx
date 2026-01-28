import React from 'react';
import { Settings, Mic, Video, Users, Clock } from 'lucide-react';
import { cn } from '@/lib/utils';

interface StudioLayoutProps {
    children: React.ReactNode;
    rightPanel: React.ReactNode;
    controlBar: React.ReactNode;
}

export function StudioLayout({ children, rightPanel, controlBar }: StudioLayoutProps) {
    return (
        <div className="flex flex-col h-screen w-full bg-zinc-950 text-zinc-100 overflow-hidden">
            {/* Header / Status Bar */}
            <header className="h-10 border-b border-zinc-800 flex items-center justify-between px-4 bg-zinc-950 shrink-0">
                <div className="flex items-center gap-2">
                    <span className="font-bold text-sm tracking-wider text-zinc-400">VYUBER</span>
                    <span className="px-1.5 py-0.5 rounded-full bg-zinc-800 text-[10px] text-zinc-400 font-mono">MVP</span>
                </div>

                <div className="flex items-center gap-6 text-xs text-zinc-500">
                    <div className="flex items-center gap-1.5">
                        <Clock className="w-3.5 h-3.5" />
                        <span className="font-mono">録画時間: 00:00:00</span>
                    </div>
                    <div className="flex items-center gap-1.5">
                        <Users className="w-3.5 h-3.5" />
                        <span className="font-mono">視聴者: 0人</span>
                    </div>
                </div>

                <div className="flex items-center gap-2">
                    <button className="p-1.5 hover:bg-zinc-800 rounded-md text-zinc-400 transition-colors">
                        <Settings className="w-4 h-4" />
                    </button>
                </div>
            </header>

            {/* Main Content Area */}
            <div className="flex-1 flex overflow-hidden">
                {/* Main Stage (Video Preview) */}
                <main className="flex-1 relative bg-black/50 overflow-hidden flex items-center justify-center p-4">
                    <div className="w-full h-full relative shadow-2xl ring-1 ring-zinc-800/50 rounded-sm overflow-hidden bg-zinc-900">
                        {children}
                    </div>
                </main>

                {/* Right Sidebar (Chat) */}
                <aside className="w-80 border-l border-zinc-800 bg-zinc-950/50 flex flex-col shrink-0">
                    <div className="h-full overflow-hidden">
                        {rightPanel}
                    </div>
                </aside>
            </div>

            {/* Footer / Control Bar */}
            <footer className="h-20 border-t border-zinc-800 bg-zinc-950 shrink-0 flex items-center justify-center px-6 relative z-10">
                {controlBar}
            </footer>
        </div>
    );
}
