use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{BlobEvent, MediaRecorder, MediaRecorderOptions, WebSocket, MessageEvent};
use serde::{Deserialize, Serialize};
use log::{error, info}; // infoを追加

#[derive(Serialize, Deserialize, Debug)]
struct DeepgramResponse {
    channel: Channel,
    is_final: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct Channel {
    alternatives: Vec<Alternative>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Alternative {
    transcript: String,
}

#[component]
pub fn Mic() -> impl IntoView {
    let (is_recording, set_recording) = signal(false);
    let (transcript, set_transcript) = signal("".to_string());
    
    let recorder_ref = StoredValue::new_local(None::<MediaRecorder>);
    let ws_ref = StoredValue::new_local(None::<WebSocket>);

    let cleanup = move || {
        ws_ref.update_value(|ws| {
            if let Some(socket) = ws {
                let _ = socket.close();
            }
            *ws = None;
        });
        recorder_ref.update_value(|rec| {
            if let Some(r) = rec {
                if r.state() != web_sys::RecordingState::Inactive {
                    let _ = r.stop();
                }
            }
            *rec = None;
        });
        set_recording.set(false);
    };

    let toggle_recording = move |_| {
        if is_recording.get() {
            cleanup();
        } else {
            set_transcript.set("準備中...".to_string());
            set_recording.set(true);

            spawn_local(async move {
                let window = web_sys::window().unwrap();
                let navigator = window.navigator();
                let media_devices = navigator.media_devices().expect("MediaDevices not found");

                // 1. まずマイクの権限を取得（ここが先！）
                let constraints = web_sys::MediaStreamConstraints::new();
                constraints.set_audio(&JsValue::from(true));

                match media_devices.get_user_media_with_constraints(&constraints) {
                    Ok(promise) => {
                        let stream_js = wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();
                        let media_stream = stream_js.dyn_into::<web_sys::MediaStream>().unwrap();
                        
                        // マイクOKなら、レコーダー作成
                        let options = MediaRecorderOptions::new();
                        // options.set_mime_type("audio/webm"); // 自動判定
                        let recorder = match MediaRecorder::new_with_media_stream_and_media_recorder_options(&media_stream, &options) {
                            Ok(r) => r,
                            Err(e) => {
                                error!("Recorder creation failed: {:?}", e);
                                set_transcript.set("レコーダーエラー".to_string());
                                set_recording.set(false);
                                return;
                            }
                        };

                        // 2. 次にWebSocketに接続
                        let protocol = if window.location().protocol().unwrap() == "https:" { "wss:" } else { "ws:" };
                        let host = window.location().host().unwrap();
                        let ws_url = format!("{}//{}/api/transcribe/live", protocol, host);
                        
                        web_sys::console::log_1(&"Connecting to WebSocket...".into());

                        let ws = match WebSocket::new(&ws_url) {
                            Ok(ws) => ws,
                            Err(e) => {
                                error!("WebSocket connection failed: {:?}", e);
                                set_transcript.set("接続エラー".to_string());
                                set_recording.set(false);
                                return;
                            }
                        };
                        ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

                        // メッセージ受信（文字起こし結果）
                        let on_message = Closure::wrap(Box::new(move |e: MessageEvent| {
                            if let Ok(text) = e.data().dyn_into::<js_sys::JsString>() {
                                let text_string: String = text.into();
                                if let Ok(response) = serde_json::from_str::<DeepgramResponse>(&text_string) {
                                    if let Some(alt) = response.channel.alternatives.first() {
                                        let content = &alt.transcript;
                                        if !content.is_empty() {
                                            set_transcript.set(content.clone());
                                        }
                                    }
                                }
                            }
                        }) as Box<dyn FnMut(MessageEvent)>);
                        ws.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
                        on_message.forget();

                        // 音声データ送信
                        let ws_clone = ws.clone();
                        let on_data = Closure::wrap(Box::new(move |e: BlobEvent| {
                            if let Some(blob) = e.data() {
                                if blob.size() > 0.0 && ws_clone.ready_state() == WebSocket::OPEN {
                                    let _ = ws_clone.send_with_blob(&blob);
                                }
                            }
                        }) as Box<dyn FnMut(BlobEvent)>);
                        recorder.set_ondataavailable(Some(on_data.as_ref().unchecked_ref()));
                        on_data.forget();

                        // 3. 接続完了イベント（ここで確実にstartする）
                        let recorder_clone = recorder.clone();
                        let on_open = Closure::wrap(Box::new(move || {
                            web_sys::console::log_1(&"WebSocket Open! Starting Recorder...".into());
                            set_transcript.set("聞いています...".to_string());
                            // 250msごとにスライスして送信
                            recorder_clone.start_with_time_slice(250).unwrap();
                        }) as Box<dyn FnMut()>);
                        ws.set_onopen(Some(on_open.as_ref().unchecked_ref()));
                        on_open.forget();

                        ws_ref.set_value(Some(ws));
                        recorder_ref.set_value(Some(recorder));
                    },
                    Err(_) => {
                        set_transcript.set("マイク許可エラー".to_string());
                        set_recording.set(false);
                    }
                }
            });
        }
    };

    view! {
        <div style="position: fixed; bottom: 20px; left: 20px; z-index: 9999; font-family: sans-serif;">
            <button 
                on:click=toggle_recording
                style="background: #ff4444; color: white; border: none; padding: 12px 24px; border-radius: 30px; font-weight: bold; cursor: pointer; box-shadow: 0 4px 6px rgba(0,0,0,0.3); font-size: 16px;">
                {move || if is_recording.get() { "■ リアルタイム停止" } else { "🎤 リアルタイム入力" }}
            </button>
            <div style="margin-top: 10px; background: rgba(0,0,0,0.8); color: white; padding: 10px; border-radius: 8px; max-width: 300px; min-height: 20px;">
                "認識: " <span style="color: #44ff44; font-weight: bold;">{transcript}</span>
            </div>
        </div>
    }
}