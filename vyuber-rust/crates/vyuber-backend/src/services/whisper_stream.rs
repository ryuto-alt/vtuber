use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt};
use std::sync::Arc;

use super::whisper::WhisperService;

const SAMPLE_RATE: usize = 16000;

/// エネルギー計算に使うウィンドウサイズ（0.3秒 = 4800サンプル）
const ENERGY_WINDOW: usize = SAMPLE_RATE * 3 / 10;

/// 音声エネルギーの閾値（これ以下は無音とみなす）
/// ※ 0.003: 小声やマイクが遠い環境でも検出できるよう低めに設定
const ENERGY_THRESHOLD: f32 = 0.003;

/// 喋り終わり判定: この秒数の沈黙が続いたら文字起こしを確定する
const SILENCE_TIMEOUT_SECS: f32 = 2.0;
/// 沈黙タイムアウトのサンプル数
const SILENCE_TIMEOUT_SAMPLES: usize = (SAMPLE_RATE as f32 * SILENCE_TIMEOUT_SECS) as usize;

/// 最大バッファ長（メモリ制限: 30秒）
const MAX_BUFFER_SAMPLES: usize = SAMPLE_RATE * 30;

/// 最小発話長（これより短い音声は無視: 0.5秒）
const MIN_SPEECH_SAMPLES: usize = SAMPLE_RATE / 2;

/// 音声バッファのRMSエネルギーを計算
fn rms_energy(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum: f32 = samples.iter().map(|s| s * s).sum();
    (sum / samples.len() as f32).sqrt()
}

/// PCMデータをピーク正規化（小声でもWhisperが正確に認識できるよう音量を揃える）
fn normalize_audio(samples: &mut [f32]) {
    let peak = samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    // ピークが小さすぎる場合（ほぼ無音）は増幅しない、既に十分大きければ何もしない
    if peak > 1e-4 && peak < 0.9 {
        let gain = 0.9 / peak;
        for sample in samples.iter_mut() {
            *sample *= gain;
        }
        tracing::debug!("Audio normalized: peak {:.4} → 0.9 (gain: {:.1}x)", peak, gain);
    }
}

/// 発話状態の管理
#[derive(Debug, PartialEq)]
enum SpeechState {
    /// 無音状態（発話待ち）
    Idle,
    /// 発話中（音声を蓄積中）
    Speaking,
    /// 発話後の沈黙（喋り終わりか一時停止かを判定中）
    TrailingSilence,
}

pub async fn handler(
    ws: WebSocketUpgrade,
    State(whisper): State<Arc<WhisperService>>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, whisper))
}

async fn handle_socket(socket: WebSocket, whisper: Arc<WhisperService>) {
    tracing::info!("リアルタイム音声認識を開始 (Whisper)");

    let (mut sender, mut receiver) = socket.split();

    // 発話全体を蓄積するバッファ
    let mut audio_buffer: Vec<f32> = Vec::with_capacity(SAMPLE_RATE * 10);
    // 現在の状態
    let mut state = SpeechState::Idle;
    // 沈黙が続いているサンプル数
    let mut silence_samples: usize = 0;
    // 発話部分のサンプル数（無音を除く）
    let mut speech_samples: usize = 0;

    while let Some(Ok(msg)) = receiver.next().await {
        match msg {
            Message::Binary(data) => {
                let samples: Vec<f32> = data
                    .chunks_exact(4)
                    .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                    .collect();

                if samples.is_empty() {
                    continue;
                }

                // 直近のエネルギーを計算
                let energy = if samples.len() >= ENERGY_WINDOW {
                    rms_energy(&samples[samples.len() - ENERGY_WINDOW..])
                } else {
                    rms_energy(&samples)
                };

                let is_speech = energy > ENERGY_THRESHOLD;

                match state {
                    SpeechState::Idle => {
                        if is_speech {
                            // 発話開始
                            state = SpeechState::Speaking;
                            audio_buffer.clear();
                            audio_buffer.extend_from_slice(&samples);
                            speech_samples = samples.len();
                            silence_samples = 0;
                            tracing::debug!("発話開始 (energy: {:.4})", energy);
                        }
                        // 無音中は何もしない（バッファに貯めない）
                    }
                    SpeechState::Speaking => {
                        audio_buffer.extend_from_slice(&samples);

                        if is_speech {
                            speech_samples += samples.len();
                            silence_samples = 0;
                        } else {
                            // 沈黙が始まった
                            silence_samples = samples.len();
                            state = SpeechState::TrailingSilence;
                        }
                    }
                    SpeechState::TrailingSilence => {
                        audio_buffer.extend_from_slice(&samples);

                        if is_speech {
                            // 再び喋り始めた → Speaking に戻る
                            speech_samples += samples.len();
                            silence_samples = 0;
                            state = SpeechState::Speaking;
                            tracing::debug!("発話再開 (energy: {:.4})", energy);
                        } else {
                            silence_samples += samples.len();

                            // 十分な沈黙が続いた → 喋り終わりと判定
                            if silence_samples >= SILENCE_TIMEOUT_SAMPLES {
                                tracing::debug!(
                                    "発話終了検出 (silence: {:.1}s, speech: {:.1}s, total: {:.1}s)",
                                    silence_samples as f32 / SAMPLE_RATE as f32,
                                    speech_samples as f32 / SAMPLE_RATE as f32,
                                    audio_buffer.len() as f32 / SAMPLE_RATE as f32,
                                );

                                // 最小発話長チェック
                                if speech_samples >= MIN_SPEECH_SAMPLES {
                                    // 末尾の無音部分をトリムして送信
                                    let trim_end = audio_buffer
                                        .len()
                                        .saturating_sub(silence_samples.saturating_sub(SAMPLE_RATE / 4));
                                    let mut pcm_data = audio_buffer[..trim_end].to_vec();
                                    // 小声でもWhisperが正確に認識できるよう正規化
                                    normalize_audio(&mut pcm_data);

                                    let whisper_clone = whisper.clone();
                                    let result = tokio::task::spawn_blocking(move || {
                                        whisper_clone.transcribe(&pcm_data)
                                    })
                                    .await;

                                    let text = match result {
                                        Ok(Ok(text)) => text,
                                        Ok(Err(e)) => {
                                            tracing::error!("Whisper error: {}", e);
                                            String::new()
                                        }
                                        Err(e) => {
                                            tracing::error!("Task join error: {}", e);
                                            String::new()
                                        }
                                    };

                                    if !text.is_empty() {
                                        let response = serde_json::json!({
                                            "text": text,
                                            "is_final": true
                                        });

                                        if sender
                                            .send(Message::Text(response.to_string()))
                                            .await
                                            .is_err()
                                        {
                                            tracing::warn!("ブラウザへの送信エラー");
                                            break;
                                        }
                                    }
                                }

                                // リセット
                                audio_buffer.clear();
                                speech_samples = 0;
                                silence_samples = 0;
                                state = SpeechState::Idle;
                            }
                        }
                    }
                }

                // メモリ保護: バッファが長すぎる場合は強制的に推論
                if audio_buffer.len() >= MAX_BUFFER_SAMPLES && state != SpeechState::Idle {
                    tracing::warn!("バッファ上限到達 (30s)、強制推論");

                    let mut pcm_data = audio_buffer.clone();
                    normalize_audio(&mut pcm_data);
                    let whisper_clone = whisper.clone();
                    let result = tokio::task::spawn_blocking(move || {
                        whisper_clone.transcribe(&pcm_data)
                    })
                    .await;

                    let text = match result {
                        Ok(Ok(text)) => text,
                        Ok(Err(e)) => {
                            tracing::error!("Whisper error: {}", e);
                            String::new()
                        }
                        Err(e) => {
                            tracing::error!("Task join error: {}", e);
                            String::new()
                        }
                    };

                    if !text.is_empty() {
                        let response = serde_json::json!({
                            "text": text,
                            "is_final": true
                        });

                        if sender
                            .send(Message::Text(response.to_string()))
                            .await
                            .is_err()
                        {
                            tracing::warn!("ブラウザへの送信エラー");
                            break;
                        }
                    }

                    audio_buffer.clear();
                    speech_samples = 0;
                    silence_samples = 0;
                    state = SpeechState::Idle;
                }
            }
            Message::Close(_) => {
                tracing::info!("ブラウザが接続を切断しました");
                break;
            }
            _ => {}
        }
    }

    tracing::info!("リアルタイム音声認識を終了 (Whisper)");
}
