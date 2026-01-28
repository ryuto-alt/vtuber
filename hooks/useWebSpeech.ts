"use client";

import { useState, useEffect, useCallback, useRef } from 'react';

export const useWebSpeech = () => {
  const [isListening, setIsListening] = useState(false);
  const [transcript, setTranscript] = useState(''); // 確定した文章
  const [interim, setInterim] = useState('');       // 話している最中の文章
  const recognitionRef = useRef<any>(null);

  useEffect(() => {
    // ブラウザ対応チェック
    const SpeechRecognition = window.SpeechRecognition || (window as any).webkitSpeechRecognition;
    if (!SpeechRecognition) return;

    const recognition = new SpeechRecognition();
    recognition.lang = 'ja-JP';
    recognition.interimResults = true; // ★ここが重要：リアルタイム表示用
    recognition.continuous = true;     // 連続認識モード

    recognition.onresult = (event: any) => {
      let finalTranscript = '';
      let interimTranscript = '';

      for (let i = event.resultIndex; i < event.results.length; ++i) {
        if (event.results[i].isFinal) {
          finalTranscript += event.results[i][0].transcript;
        } else {
          interimTranscript += event.results[i][0].transcript;
        }
      }

      if (finalTranscript) {
        setTranscript(finalTranscript); // 確定した文をセット
      }
      setInterim(interimTranscript);    // 途中経過をセット
    };

    recognition.onerror = (event: any) => {
      console.error('Speech recognition error', event.error);
      setIsListening(false);
    };

    recognitionRef.current = recognition;
  }, []);

  const startListening = useCallback(() => {
    if (recognitionRef.current && !isListening) {
      try {
        recognitionRef.current.start();
        setIsListening(true);
      } catch (e) {
        console.error(e);
      }
    }
  }, [isListening]);

  const stopListening = useCallback(() => {
    if (recognitionRef.current && isListening) {
      recognitionRef.current.stop();
      setIsListening(false);
    }
  }, [isListening]);

  return { 
    isListening, 
    startListening, 
    stopListening, 
    transcript, // 確定テキスト（これをGeminiに送る）
    interim     // 途中経過テキスト（画面表示用）
  };
};