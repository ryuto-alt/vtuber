use leptos::prelude::*;
use vyuber_shared::chat::ChatMessage;
use wasm_bindgen_futures::spawn_local;
use wasm_bindgen::prelude::*;

mod services;

// WASM entry point - automatically called when module loads
#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).expect("error initializing logger");

    log::info!("Mounting Leptos app...");
    mount_to_body(App);
    log::info!("Leptos app mounted successfully!");
}

#[component]
pub fn App() -> impl IntoView {
    let (messages, set_messages) = signal(Vec::<ChatMessage>::new());
    let (is_streaming, set_is_streaming) = signal(false);
    let (is_listening, set_is_listening) = signal(false);

    // 簡易的な音声認識開始/停止
    let start_listening = move || {
        set_is_listening.set(true);
        log::info!("Listening started");
    };

    let stop_listening = move || {
        set_is_listening.set(false);
        log::info!("Listening stopped");
    };

    // チャット送信ハンドラ
    let send_chat = move |text: String| {
        // 自分のメッセージを追加
        set_messages.update(|msgs| {
            msgs.push(ChatMessage {
                id: js_sys::Date::now() as i64,
                user: "Me".to_string(),
                text: text.clone(),
                color: "text-white".to_string(),
            });
        });

        // API呼び出し（非同期）
        Effect::new(move |_| {
            let text = text.clone();
            let set_messages = set_messages.clone();

            spawn_local(async move {
                match services::chat_api::send_message(&text).await {
                    Ok(comments) => {
                        for (i, comment) in comments.into_iter().enumerate() {
                            let delay = 500 + (i * 400);
                            gloo_timers::future::TimeoutFuture::new(delay as u32).await;

                            set_messages.update(|msgs| {
                                msgs.push(ChatMessage {
                                    id: js_sys::Date::now() as i64,
                                    user: comment.user,
                                    text: comment.text,
                                    color: comment.color,
                                });
                            });
                        }
                    }
                    Err(e) => {
                        log::error!("Chat API error: {}", e);
                        set_messages.update(|msgs| {
                            msgs.push(ChatMessage {
                                id: js_sys::Date::now() as i64,
                                user: "System".to_string(),
                                text: "APIエラーが発生しました".to_string(),
                                color: "text-red-500".to_string(),
                            });
                        });
                    }
                }
            });
        });
    };

    view! {
        <div class="min-h-screen bg-zinc-950 text-white">
            <StudioLayout
                messages=messages
                is_streaming=is_streaming
                set_is_streaming=set_is_streaming
                is_listening=is_listening
                start_listening=start_listening
                stop_listening=stop_listening
                send_chat=send_chat
            />
        </div>
    }
}

#[component]
fn StudioLayout(
    messages: ReadSignal<Vec<ChatMessage>>,
    is_streaming: ReadSignal<bool>,
    set_is_streaming: WriteSignal<bool>,
    is_listening: ReadSignal<bool>,
    start_listening: impl Fn() + 'static + Copy,
    stop_listening: impl Fn() + 'static + Copy,
    send_chat: impl Fn(String) + 'static + Copy,
) -> impl IntoView {
    view! {
        <div class="flex flex-col h-screen">
            <header class="bg-zinc-900 border-b border-zinc-800 px-6 py-3">
                <h1 class="text-xl font-bold">"VYUBER MVP (Rust)"</h1>
            </header>

            <div class="flex-1 flex overflow-hidden">
                <div class="flex-1 bg-zinc-900 flex items-center justify-center">
                    <VideoPreview/>
                </div>

                <div class="w-96 bg-zinc-950 border-l border-zinc-800 overflow-y-auto">
                    <ChatOverlay messages=messages/>
                </div>
            </div>

            <footer class="bg-zinc-900 border-t border-zinc-800 px-6 py-4">
                <ControlBar
                    is_streaming=is_streaming
                    set_is_streaming=set_is_streaming
                    is_listening=is_listening
                    start_listening=start_listening
                    stop_listening=stop_listening
                />
            </footer>

            <div class="absolute bottom-4 left-4 z-50">
                <button
                    on:click=move |_| {
                        if is_listening.get() {
                            stop_listening();
                        } else {
                            start_listening();
                        }
                    }
                    class=move || {
                        if is_listening.get() {
                            "px-4 py-2 rounded-full font-bold shadow-lg bg-red-500 text-white animate-pulse"
                        } else {
                            "px-4 py-2 rounded-full font-bold shadow-lg bg-blue-600 text-white"
                        }
                    }
                >
                    {move || if is_listening.get() {
                        "Listening..."
                    } else {
                        "Click to Start Mic"
                    }}
                </button>
                <button
                    on:click=move |_| {
                        send_chat("テストメッセージ".to_string());
                    }
                    class="ml-2 px-4 py-2 rounded-full font-bold shadow-lg bg-green-600 text-white"
                >
                    "テスト送信"
                </button>
            </div>
        </div>
    }
}

#[component]
fn VideoPreview() -> impl IntoView {
    view! {
        <div class="w-full h-full flex flex-col items-center justify-center relative">
            <video
                id="video-preview"
                class="w-full h-full object-contain hidden"
                autoplay=true
                muted=true
            ></video>

            <div class="absolute inset-0 flex flex-col items-center justify-center">
                <div class="absolute inset-0 bg-gradient-to-br from-zinc-800 to-zinc-900 opacity-50"></div>
                <div class="relative z-10 text-center space-y-4">
                    <div class="text-6xl">"📹"</div>
                    <div class="text-zinc-400 font-medium">"ストリーム待機中"</div>
                    <p class="text-zinc-600 text-sm">"OBSから配信を開始してください"</p>
                </div>
            </div>
        </div>
    }
}

#[component]
fn ChatOverlay(messages: ReadSignal<Vec<ChatMessage>>) -> impl IntoView {
    view! {
        <div class="p-4 space-y-3">
            <div class="text-lg font-semibold mb-4">"💬 チャット"</div>
            {move || {
                let msgs = messages.get();
                if msgs.is_empty() {
                    view! {
                        <div class="text-zinc-500 text-sm text-center py-8">
                            "音声入力を開始すると、AIコメントがここに表示されます"
                        </div>
                    }.into_any()
                } else {
                    msgs.iter().map(|msg| {
                        let user = msg.user.clone();
                        let text = msg.text.clone();
                        let color = msg.color.clone();
                        view! {
                            <div class="mb-2 p-2 rounded bg-zinc-900">
                                <div class=format!("text-xs {}", color)>
                                    {user}
                                </div>
                                <div class="text-sm">
                                    {text}
                                </div>
                            </div>
                        }
                    }).collect_view().into_any()
                }
            }}
        </div>
    }
}

#[component]
fn ControlBar(
    is_streaming: ReadSignal<bool>,
    set_is_streaming: WriteSignal<bool>,
    is_listening: ReadSignal<bool>,
    start_listening: impl Fn() + 'static + Copy,
    stop_listening: impl Fn() + 'static + Copy,
) -> impl IntoView {
    view! {
        <div class="flex items-center justify-between">
            <div class="flex items-center gap-3">
                <button
                    on:click=move |_| {
                        if is_listening.get() {
                            stop_listening();
                        } else {
                            start_listening();
                        }
                    }
                    class=move || {
                        if is_listening.get() {
                            "p-3 rounded-full bg-red-600 hover:bg-red-700 transition-colors"
                        } else {
                            "p-3 rounded-full bg-zinc-800 hover:bg-zinc-700 transition-colors"
                        }
                    }
                >
                    "🎤"
                </button>
                <button class="p-3 rounded-full bg-zinc-800 hover:bg-zinc-700 transition-colors">
                    "📹"
                </button>
            </div>

            <button
                on:click=move |_| {
                    set_is_streaming.update(|s| *s = !*s);
                }
                class=move || {
                    if is_streaming.get() {
                        "h-14 px-8 rounded-full flex items-center gap-3 font-semibold bg-red-600 text-white"
                    } else {
                        "h-14 px-8 rounded-full flex items-center gap-3 font-semibold bg-white text-zinc-900"
                    }
                }
            >
                <span class="text-xl">
                    {move || if is_streaming.get() { "⏹" } else { "▶" }}
                </span>
                <span>
                    {move || if is_streaming.get() { "配信停止" } else { "配信開始" }}
                </span>
            </button>

            <button class="px-4 py-2 rounded-lg bg-zinc-800 hover:bg-zinc-700 text-sm">
                "🔑 ストリームキー"
            </button>
        </div>
    }
}
