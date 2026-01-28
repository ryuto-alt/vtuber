"use client";

import React, { useState, useEffect, useRef } from 'react';
import { StudioLayout } from "@/components/StudioLayout";
import { ChatOverlay, ChatMessage } from "@/components/ChatOverlay";
import { ControlBar } from "@/components/ControlBar";
import { VideoPreview } from "@/components/VideoPreview";
import { useWebSpeech } from "@/hooks/useWebSpeech"; // 前回のフック

export default function Home() {
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  
  // Web Speech API フックを使用
  const { isListening, startListening, stopListening, transcript, interim } = useWebSpeech();
  
  // "前回の確定テキスト" を覚えておき、変化したらAPIを送るためのRef
  const lastProcessedText = useRef("");

  // メッセージを追加するヘルパー関数
  const addMessage = (user: string, text: string, color: string) => {
    setMessages(prev => [...prev, {
      id: Date.now() + Math.random(),
      user,
      text,
      color
    }]);
  };

  // 音声認識の結果を監視して処理する
  useEffect(() => {
    // transcript（確定テキスト）が入っていて、かつ前回処理したものと違う場合のみ実行
    if (transcript && transcript !== lastProcessedText.current) {
      const currentText = transcript;
      lastProcessedText.current = currentText;

      handleVoiceInput(currentText);
    }
  }, [transcript]);

  // ★ API通信と遅延表示のロジック
  const handleVoiceInput = async (text: string) => {
    // 1. まず自分の発言をチャットに表示 (白文字)
    addMessage("Me", text, "text-white");

    try {
      // 2. Gemini APIへ送信
      const res = await fetch("/api/chat", {
        method: "POST",
        body: JSON.stringify({ message: text }),
      });
      const data = await res.json();

      // 3. 返ってきた5件のコメントを、ランダムな遅延で順番に表示
      if (data.comments && Array.isArray(data.comments)) {
        data.comments.forEach((comment: any, index: number) => {
          // 500ms 〜 2500ms の間でずらして表示
          const delay = 500 + (index * 400) + (Math.random() * 500);
          
          setTimeout(() => {
            addMessage(comment.user, comment.text, comment.color);
          }, delay);
        });
      }
    } catch (e) {
      console.error("AI Error:", e);
    }
  };

  return (
    <StudioLayout
      rightPanel={
        // メッセージ状態を渡す
        <ChatOverlay messages={messages} />
      }
      // マイクの操作をControlBarに渡す（必要ならControlBar側でpropsを受け取る形に修正）
      controlBar={<ControlBar />} 
    >
      <VideoPreview />
      
      {/* デバッグ用：マイク状態を強制的にオンにするボタン（ControlBar連携前用） */}
      <div className="absolute bottom-4 left-4 z-50">
        <button 
          onClick={isListening ? stopListening : startListening}
          className={`px-4 py-2 rounded-full font-bold shadow-lg ${
             isListening ? "bg-red-500 text-white animate-pulse" : "bg-blue-600 text-white"
          }`}
        >
          {isListening ? "Listening... (話しかけてね)" : "Click to Start Mic"}
        </button>
        {/* 話している最中の文字（interim）を画面中央下に字幕風に出す演出 */}
        {isListening && interim && (
          <div className="absolute -top-20 left-0 w-full text-center">
            <span className="bg-black/60 text-yellow-300 px-4 py-2 rounded text-xl font-bold backdrop-blur-md">
              {interim}
            </span>
          </div>
        )}
      </div>

    </StudioLayout>
  );
}