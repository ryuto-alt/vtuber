"use client";

import React, { useState, useEffect, useRef } from 'react';
import { StudioLayout } from "@/components/StudioLayout";
import { ChatOverlay, ChatMessage } from "@/components/ChatOverlay";
import { ControlBar } from "@/components/ControlBar";
import { VideoPreview } from "@/components/VideoPreview";
import { useWebSpeech } from "@/hooks/useWebSpeech"; // 前回のフック

export default function Home() {
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [isStreaming, setIsStreaming] = useState(false);

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
    console.log('[Frontend] transcript changed:', transcript);
    console.log('[Frontend] lastProcessedText:', lastProcessedText.current);

    // transcript（確定テキスト）が入っていて、かつ前回処理したものと違う場合のみ実行
    if (transcript && transcript !== lastProcessedText.current) {
      const currentText = transcript;
      console.log('[Frontend] New transcript detected, processing:', currentText);
      lastProcessedText.current = currentText;

      handleVoiceInput(currentText);
    }
  }, [transcript]);

  // 配信の開始/停止を切り替える関数
  const handleToggleStreaming = (streaming: boolean) => {
    setIsStreaming(streaming);
    // TODO: 実際の配信開始/停止処理をここに追加
    console.log(streaming ? "配信開始" : "配信停止");
  };

  // ★ API通信と遅延表示のロジック
  const handleVoiceInput = async (text: string) => {
    console.log('[Frontend] handleVoiceInput called with:', text);

    // 1. まず自分の発言をチャットに表示 (白文字)
    addMessage("Me", text, "text-white");

    try {
      console.log('[Frontend] Sending request to /api/chat...');

      // 2. Gemini APIへ送信
      const res = await fetch("/api/chat", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({ message: text }),
      });

      console.log('[Frontend] Response status:', res.status);
      const data = await res.json();
      console.log('[Frontend] Response data:', data);

      // エラーチェック
      if (data.error) {
        console.error('[Frontend] API returned error:', data.error, data.details);
        addMessage("System", `エラー: ${data.details || data.error}`, "text-red-500");
        return;
      }

      // 3. 返ってきた5件のコメントを、ランダムな遅延で順番に表示
      if (data.comments && Array.isArray(data.comments)) {
        console.log('[Frontend] Processing', data.comments.length, 'comments');
        data.comments.forEach((comment: any, index: number) => {
          // 500ms 〜 2500ms の間でずらして表示
          const delay = 500 + (index * 400) + (Math.random() * 500);

          console.log(`[Frontend] Scheduling comment ${index + 1} with delay ${delay}ms:`, comment);

          setTimeout(() => {
            console.log(`[Frontend] Adding comment ${index + 1}:`, comment);
            addMessage(comment.user, comment.text, comment.color);
          }, delay);
        });
      } else {
        console.warn('[Frontend] No comments in response or comments is not an array:', data);
      }
    } catch (e) {
      console.error("[Frontend] AI Error:", e);
      addMessage("System", "APIエラーが発生しました", "text-red-500");
    }
  };

  return (
    <StudioLayout
      rightPanel={
        // メッセージ状態を渡す
        <ChatOverlay messages={messages} />
      }
      // マイクの操作をControlBarに渡す（必要ならControlBar側でpropsを受け取る形に修正）
      controlBar={<ControlBar isStreaming={isStreaming} onToggleStreaming={handleToggleStreaming} />} 
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