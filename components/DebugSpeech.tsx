"use client";

import { useWebSpeech } from "@/hooks/useWebSpeech";
import { useEffect, useState } from "react";

export const DebugSpeech = () => {
  // さっき作ったフック(シェフ)を呼び出して、機能を使えるようにする
  const { isListening, startListening, stopListening, transcript, interim } = useWebSpeech();
  
  return (
    <div className="fixed bottom-10 left-10 z-50 p-4 bg-black/90 text-white rounded-xl border border-gray-700 w-80 shadow-2xl">
      <div className="flex justify-between items-center mb-4">
        <h3 className="text-sm font-bold text-gray-400">音声認識テスト</h3>
        {/* マイク状態のインジケーター */}
        <div className={`w-3 h-3 rounded-full ${isListening ? "bg-red-500 animate-pulse" : "bg-gray-600"}`} />
      </div>
      
      {/* 操作ボタン */}
      <button
        onClick={isListening ? stopListening : startListening}
        className={`w-full py-3 rounded-lg font-bold transition-all ${
          isListening 
            ? "bg-red-500/20 text-red-400 border border-red-500 hover:bg-red-500/30" 
            : "bg-blue-500 text-white hover:bg-blue-600"
        }`}
      >
        {isListening ? "⏹️ 停止する" : "🎙️ マイクON"}
      </button>

      {/* 結果表示エリア */}
      <div className="mt-4 space-y-2">
        <div className="text-xs text-gray-500">認識結果:</div>
        <div className="bg-gray-800 p-3 rounded-lg min-h-[60px] text-sm leading-relaxed border border-gray-700">
          {/* 確定した文字（白） */}
          <span className="text-white">{transcript}</span>
          {/* 話している途中の文字（黄色） */}
          <span className="text-yellow-400 opacity-80 ml-1">{interim}</span>
          
          {!transcript && !interim && (
            <span className="text-gray-600 italic">ここに文字が出ます...</span>
          )}
        </div>
      </div>
    </div>
  );
};