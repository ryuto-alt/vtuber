use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{BlobEvent, MediaRecorder, MediaRecorderOptions, WebSocket, MessageEvent};
use serde::{Deserialize, Serialize};
use log::{error, info};
use crate::state::{GlobalState, ChatUser}; // stateを使う

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
    // ステートを取得
    let state = use_context::<GlobalState>().expect("GlobalState not found");

    let (is_recording, set_recording) = signal(false);
    let (send_to_chat, set_send_to_chat) = signal(true); // ★チャット送信ON/OFF
    
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
            set_recording.set(true);

            spawn_local(async move {
                let window = web_sys::window().unwrap();
                let navigator = window.navigator();
                let media_devices = navigator.media_devices().expect("MediaDevices not found");

                // 1. マイク権限
                let constraints = web_sys::MediaStreamConstraints::new();
                constraints.set_audio(&JsValue::from(true));

                match media_devices.get_user_media_with_constraints(&constraints) {
                    Ok(promise) => {
                        let stream_js = wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();
                        let media_stream = stream_js.dyn_into::<web_sys::MediaStream>().unwrap();
                        let options = MediaRecorderOptions::new();
                        let recorder = MediaRecorder::new_with_media_stream_and_media_recorder_options(&media_stream, &options).unwrap();

                        // 2. WebSocket
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

                        // メッセージ受信
                        let on_message = Closure::wrap(Box::new(move |e: MessageEvent| {
                            if let Ok(text) = e.data().dyn_into::<js_sys::JsString>() {
                                let text_string: String = text.into();
                                if let Ok(response) = serde_json::from_str::<DeepgramResponse>(&text_string) {
                                    // ★ is_final: true（文章が確定した）時だけ処理する
                                    if response.is_final {
                                        if let Some(alt) = response.channel.alternatives.first() {
                                            let content = alt.transcript.trim();
                                            // スイッチがONで、かつ空文字じゃなければ送信
                                            if send_to_chat.get() && !content.is_empty() {
                                                state.add_message(ChatUser::Me, content.to_string());
                                            }
                                        }
                                    }
                                }
                            }
                        }) as Box<dyn FnMut(MessageEvent)>);
                        ws.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
                        on_message.forget();

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

                        let recorder_clone = recorder.clone();
                        let on_open = Closure::wrap(Box::new(move || {
                            recorder_clone.start_with_time_slice(250).unwrap();
                        }) as Box<dyn FnMut()>);
                        ws.set_onopen(Some(on_open.as_ref().unchecked_ref()));
                        on_open.forget();

                        ws_ref.set_value(Some(ws));
                        recorder_ref.set_value(Some(recorder));
                    },
                    Err(_) => {
                        set_recording.set(false);
                    }
                }
            });
        }
    };

    // UI部分：左下の表示を削除し、ON/OFFトグルを追加
    view! {
        <div style="position: fixed; bottom: 20px; left: 20px; z-index: 9999; display: flex; align_items: center; gap: 10px;">
            // 録音ボタン
            <button 
                on:click=toggle_recording
                style="background: #ff4444; color: white; border: none; padding: 12px 24px; border-radius: 30px; font-weight: bold; cursor: pointer; box-shadow: 0 4px 6px rgba(0,0,0,0.3); font-size: 16px;">
                {move || if is_recording.get() { "■ 停止" } else { "🎤 音声入力" }}
            </button>

            // チャット送信スイッチ (録音中のみ表示)
            {move || is_recording.get().then(|| view! {
                <div style="background: rgba(0,0,0,0.7); padding: 8px 16px; border-radius: 20px; color: white; display: flex; align_items: center; gap: 8px;">
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
            
            // デバッグ用：AIコメント追加ボタン
            <button 
                on:click=move |_| state.add_demo_ai_comment()
                style="background: #4444ff; color: white; border: none; padding: 8px 16px; border-radius: 8px; font-size: 12px; cursor: pointer;">
                "🤖 AIコメント追加(Demo)"
            </button>
        </div>
    }
}