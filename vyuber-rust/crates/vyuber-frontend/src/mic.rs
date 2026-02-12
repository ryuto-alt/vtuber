use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{AudioContext, AudioProcessingEvent, GainNode, ScriptProcessorNode, WebSocket, MessageEvent};
use serde::{Deserialize, Serialize};
use log::error;
use crate::state::{GlobalState, ChatUser};

#[derive(Serialize, Deserialize, Debug)]
struct WhisperResponse {
    text: String,
    is_final: bool,
}

/// PCMデータを指定のサンプルレートにダウンサンプリング（線形補間）
fn downsample(buffer: &[f32], from_rate: f32, to_rate: f32) -> Vec<f32> {
    if (from_rate - to_rate).abs() < 1.0 {
        return buffer.to_vec();
    }
    let ratio = from_rate / to_rate;
    let new_length = (buffer.len() as f32 / ratio).floor() as usize;
    let mut result = Vec::with_capacity(new_length);
    for i in 0..new_length {
        let src_idx = i as f32 * ratio;
        let idx = src_idx as usize;
        let frac = src_idx - idx as f32;
        if idx + 1 < buffer.len() {
            // 線形補間: 隣接サンプル間を補間して音質向上
            result.push(buffer[idx] * (1.0 - frac) + buffer[idx + 1] * frac);
        } else if idx < buffer.len() {
            result.push(buffer[idx]);
        }
    }
    result
}

/// f32スライスをリトルエンディアンのバイト列に変換
fn f32_to_le_bytes(data: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(data.len() * 4);
    for sample in data {
        bytes.extend_from_slice(&sample.to_le_bytes());
    }
    bytes
}

#[component]
pub fn Mic() -> impl IntoView {
    let state = use_context::<GlobalState>().expect("GlobalState not found");

    let (is_recording, set_recording) = signal(false);
    let (send_to_chat, set_send_to_chat) = signal(true);
    let (mic_gain, set_mic_gain) = signal(2.0f64);

    let audio_ctx_ref = StoredValue::new_local(None::<AudioContext>);
    let processor_ref = StoredValue::new_local(None::<ScriptProcessorNode>);
    let gain_node_ref = StoredValue::new_local(None::<GainNode>);
    let ws_ref = StoredValue::new_local(None::<WebSocket>);

    let cleanup = move || {
        ws_ref.update_value(|ws| {
            if let Some(socket) = ws {
                let _ = socket.close();
            }
            *ws = None;
        });
        processor_ref.update_value(|proc| {
            if let Some(p) = proc {
                p.set_onaudioprocess(None);
                let _ = p.disconnect();
            }
            *proc = None;
        });
        gain_node_ref.update_value(|g| {
            if let Some(node) = g {
                let _ = node.disconnect();
            }
            *g = None;
        });
        audio_ctx_ref.update_value(|ctx| {
            if let Some(c) = ctx {
                let _ = c.close();
            }
            *ctx = None;
        });
        set_recording.set(false);
    };

    let toggle_recording = move |_| {
        if is_recording.get() {
            cleanup();
        } else {
            set_recording.set(true);

            spawn_local(async move {
                let window = web_sys::window().unwrap();
                let navigator = window.navigator();
                let media_devices = navigator.media_devices().expect("MediaDevices not found");

                // 1. マイク権限（AGC・ノイズ抑制で小声でも検出しやすくする）
                let constraints = web_sys::MediaStreamConstraints::new();
                let audio_settings = js_sys::Object::new();
                let _ = js_sys::Reflect::set(&audio_settings, &"autoGainControl".into(), &true.into());
                let _ = js_sys::Reflect::set(&audio_settings, &"noiseSuppression".into(), &true.into());
                let _ = js_sys::Reflect::set(&audio_settings, &"echoCancellation".into(), &false.into());
                constraints.set_audio(&audio_settings);

                match media_devices.get_user_media_with_constraints(&constraints) {
                    Ok(promise) => {
                        let stream_js = wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();
                        let media_stream = stream_js.dyn_into::<web_sys::MediaStream>().unwrap();

                        // 2. AudioContextを作成
                        let audio_ctx = match AudioContext::new() {
                            Ok(ctx) => ctx,
                            Err(e) => {
                                error!("AudioContext error: {:?}", e);
                                set_recording.set(false);
                                return;
                            }
                        };

                        let sample_rate = audio_ctx.sample_rate();

                        // 3. マイクストリームをAudioContextに接続
                        let source = audio_ctx
                            .create_media_stream_source(&media_stream)
                            .expect("Failed to create media stream source");

                        // 4. ScriptProcessorNode (バッファサイズ4096, 入力1ch, 出力1ch)
                        let processor = audio_ctx
                            .create_script_processor_with_buffer_size_and_number_of_input_channels_and_number_of_output_channels(
                                4096, 1, 1,
                            )
                            .expect("Failed to create script processor");

                        // 5. GainNode（マイクブースト: 小声でも検出しやすくする）
                        let gain_node = audio_ctx
                            .create_gain()
                            .expect("Failed to create gain node");
                        gain_node.gain().set_value(mic_gain.get_untracked() as f32);

                        // 6. WebSocket接続
                        let protocol = if window.location().protocol().unwrap() == "https:" { "wss:" } else { "ws:" };
                        let host = window.location().host().unwrap();
                        let ws_url = format!("{}//{}/api/transcribe/live", protocol, host);

                        let ws = match WebSocket::new(&ws_url) {
                            Ok(ws) => ws,
                            Err(e) => {
                                error!("WebSocket error: {:?}", e);
                                set_recording.set(false);
                                return;
                            }
                        };
                        ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

                        // 6. WebSocket受信: Whisperの認識結果を処理
                        let on_message = Closure::wrap(Box::new(move |e: MessageEvent| {
                            if let Ok(text) = e.data().dyn_into::<js_sys::JsString>() {
                                let text_string: String = text.into();
                                if let Ok(response) = serde_json::from_str::<WhisperResponse>(&text_string) {
                                    if response.is_final && !response.text.is_empty() {
                                        let content = response.text.trim();
                                        if send_to_chat.get() && !content.is_empty() {
                                            state.add_message(ChatUser::Me, content.to_string());
                                        }
                                    }
                                }
                            }
                        }) as Box<dyn FnMut(MessageEvent)>);
                        ws.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
                        on_message.forget();

                        // 7. 音声処理コールバック: PCMデータをダウンサンプリングして送信
                        let ws_clone = ws.clone();
                        let on_audio_process = Closure::wrap(Box::new(move |e: AudioProcessingEvent| {
                            if ws_clone.ready_state() != WebSocket::OPEN {
                                return;
                            }
                            if let Ok(input_buffer) = e.input_buffer() {
                                if let Ok(channel_data) = input_buffer.get_channel_data(0) {
                                    // ダウンサンプリング (ブラウザのレート → 16kHz)
                                    let downsampled = downsample(&channel_data, sample_rate, 16000.0);
                                    let bytes = f32_to_le_bytes(&downsampled);

                                    // バイナリで送信
                                    let array = js_sys::Uint8Array::from(&bytes[..]);
                                    let _ = ws_clone.send_with_array_buffer(&array.buffer());
                                }
                            }
                        }) as Box<dyn FnMut(AudioProcessingEvent)>);
                        processor.set_onaudioprocess(Some(on_audio_process.as_ref().unchecked_ref()));
                        on_audio_process.forget();

                        // 9. オーディオグラフを接続: source → GainNode → processor → destination
                        let _ = source.connect_with_audio_node(&gain_node);
                        let _ = gain_node.connect_with_audio_node(&processor);
                        let _ = processor.connect_with_audio_node(&audio_ctx.destination());

                        audio_ctx_ref.set_value(Some(audio_ctx));
                        processor_ref.set_value(Some(processor));
                        gain_node_ref.set_value(Some(gain_node));
                        ws_ref.set_value(Some(ws));
                    }
                    Err(_) => {
                        set_recording.set(false);
                    }
                }
            });
        }
    };

    view! {
        <div style="position: fixed; bottom: 20px; left: 100px; z-index: 9999; display: flex; align_items: center; gap: 10px;">
            <button
                on:click=toggle_recording
                style="background: #ff4444; color: white; border: none; padding: 12px 24px; border-radius: 30px; font-weight: bold; cursor: pointer; box-shadow: 0 4px 6px rgba(0,0,0,0.3); font-size: 16px;">
                {move || if is_recording.get() { "■ 停止" } else { "🎤 音声入力" }}
            </button>

            {move || is_recording.get().then(|| view! {
                <div style="background: rgba(0,0,0,0.7); padding: 8px 16px; border-radius: 20px; color: white; display: flex; align-items: center; gap: 8px;">
                    <label for="chat-toggle" style="font-size: 14px; cursor: pointer;">チャット反映</label>
                    <input
                        type="checkbox"
                        id="chat-toggle"
                        prop:checked=send_to_chat
                        on:change=move |e| set_send_to_chat.set(event_target_checked(&e))
                        style="cursor: pointer;"
                    />
                </div>
            })}

            // マイク感度スライダー（録音中のみ表示）
            {move || is_recording.get().then(|| view! {
                <div style="background: rgba(0,0,0,0.7); padding: 8px 16px; border-radius: 20px; color: white; display: flex; align-items: center; gap: 8px;">
                    <label style="font-size: 14px; white-space: nowrap;">感度</label>
                    <input
                        type="range"
                        min="1.0"
                        max="5.0"
                        step="0.5"
                        prop:value=move || mic_gain.get().to_string()
                        on:input=move |e| {
                            let value: f64 = event_target_value(&e).parse().unwrap_or(2.0);
                            set_mic_gain.set(value);
                            gain_node_ref.with_value(|g| {
                                if let Some(node) = g {
                                    node.gain().set_value(value as f32);
                                }
                            });
                        }
                        style="width: 80px; cursor: pointer;"
                    />
                    <span style="font-size: 12px; min-width: 30px;">{move || format!("x{:.1}", mic_gain.get())}</span>
                </div>
            })}
        </div>
    }
}
