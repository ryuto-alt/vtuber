use leptos::prelude::*;
use vyuber_shared::chat::ChatMessage;
use vyuber_shared::stream::StreamKeyResponse;
use wasm_bindgen_futures::spawn_local;
use wasm_bindgen::prelude::*;

mod services;

// JS bindings
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = window, js_name = initMpegtsPlayer)]
    fn init_mpegts_player(video_element_id: &str, stream_url: &str);

    #[wasm_bindgen(js_namespace = window, js_name = destroyMpegtsPlayer)]
    fn destroy_mpegts_player();
}

// JS helper for fullscreen
#[wasm_bindgen(inline_js = r#"
export function request_fullscreen(id) {
    const el = document.getElementById(id);
    if (!el) return;
    if (el.requestFullscreen) el.requestFullscreen();
    else if (el.webkitRequestFullscreen) el.webkitRequestFullscreen();
}
export function toggle_video_mute(id) {
    const el = document.getElementById(id);
    if (el) el.muted = !el.muted;
    return el ? el.muted : true;
}
export function toggle_video_play(id) {
    const el = document.getElementById(id);
    if (!el) return true;
    if (el.paused) { el.play(); return false; }
    else { el.pause(); return true; }
}
export function set_video_volume(id, vol) {
    const el = document.getElementById(id);
    if (el) { el.volume = vol; el.muted = false; }
}
export function copy_to_clipboard(text) {
    navigator.clipboard.writeText(text);
}
"#)]
extern "C" {
    fn request_fullscreen(id: &str);
    fn toggle_video_mute(id: &str) -> bool;
    fn toggle_video_play(id: &str) -> bool;
    fn set_video_volume(id: &str, vol: f64);
    fn copy_to_clipboard(text: &str);
}

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).expect("error initializing logger");
    log::info!("Mounting Leptos app...");
    mount_to_body(App);
}

#[component]
pub fn App() -> impl IntoView {
    let (messages, set_messages) = signal(Vec::<ChatMessage>::new());
    let (is_listening, set_is_listening) = signal(false);
    let (stream_key_info, set_stream_key_info) = signal(None::<StreamKeyResponse>);
    let (show_key_modal, set_show_key_modal) = signal(false);
    let (chat_input, set_chat_input) = signal(String::new());
    let (is_muted, set_is_muted) = signal(true);
    let (is_paused, set_is_paused) = signal(false);
    let (cam_on, set_cam_on) = signal(true);
    let (volume, set_volume) = signal(0.66f64);
    let (msg_count, set_msg_count) = signal(0usize);

    let start_listening = move || {
        set_is_listening.set(true);
    };
    let stop_listening = move || {
        set_is_listening.set(false);
    };

    let send_chat = move |text: String| {
        if text.trim().is_empty() { return; }
        set_messages.update(|msgs| {
            msgs.push(ChatMessage {
                id: js_sys::Date::now() as i64,
                user: "You".to_string(),
                text: text.clone(),
                color: "text-white".to_string(),
            });
        });
        set_msg_count.update(|c| *c += 1);

        Effect::new(move |_| {
            let text = text.clone();
            let set_messages = set_messages.clone();
            spawn_local(async move {
                match services::chat_api::send_message(&text).await {
                    Ok(comments) => {
                        for (i, comment) in comments.into_iter().enumerate() {
                            gloo_timers::future::TimeoutFuture::new((500 + i * 400) as u32).await;
                            set_msg_count.update(|c| *c += 1);
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
        <Header is_listening=is_listening/>
        <div class="flex flex-1 overflow-hidden min-h-0">
            <MainPanel
                is_listening=is_listening
                start_listening=start_listening
                stop_listening=stop_listening
                set_stream_key_info=set_stream_key_info
                set_show_key_modal=set_show_key_modal
                is_muted=is_muted
                set_is_muted=set_is_muted
                is_paused=is_paused
                set_is_paused=set_is_paused
                cam_on=cam_on
                set_cam_on=set_cam_on
                volume=volume
                set_volume=set_volume
                msg_count=msg_count
            />
            <ChatPanel
                messages=messages
                chat_input=chat_input
                set_chat_input=set_chat_input
                send_chat=send_chat
            />
        </div>
        <StreamKeyModal
            show_key_modal=show_key_modal
            set_show_key_modal=set_show_key_modal
            stream_key_info=stream_key_info
            set_stream_key_info=set_stream_key_info
        />
    }
}

// ─── Header ─────────────────────────────────────────────────────────────────

#[component]
fn Header(is_listening: ReadSignal<bool>) -> impl IntoView {
    view! {
        <header class="h-16 flex items-center justify-between px-6 bg-surface-dark border-b border-border-dark flex-shrink-0 z-20">
            <div class="flex items-center gap-6">
                <div class="flex items-center gap-3">
                    <div class="relative w-8 h-8 flex items-center justify-center">
                        <div class="absolute inset-0 bg-primary/20 rounded-lg transform rotate-45"></div>
                        <div class="absolute inset-0 border border-primary/40 rounded-lg transform rotate-45"></div>
                        <span class="material-symbols-outlined text-primary relative z-10 text-[20px]">"hub"</span>
                    </div>
                    <h1 class="font-bold text-xl tracking-tight text-white">
                        "AIVID"<span class="text-primary">"."</span>
                    </h1>
                </div>
                <div class="h-6 w-px bg-border-dark mx-2"></div>
                <div class="flex items-center gap-2">
                    {move || {
                        if is_listening.get() {
                            view! {
                                <span class="relative flex h-2 w-2">
                                    <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-red-400 opacity-75"></span>
                                    <span class="relative inline-flex rounded-full h-2 w-2 bg-red-500"></span>
                                </span>
                                <span class="text-xs font-medium text-red-400 uppercase tracking-wider">"ライブ配信中"</span>
                            }.into_any()
                        } else {
                            view! {
                                <span class="flex h-2 w-2 rounded-full bg-slate-600"></span>
                                <span class="text-xs font-medium text-slate-400 uppercase tracking-wider">"オフライン"</span>
                            }.into_any()
                        }
                    }}
                </div>
            </div>
            <div class="flex items-center gap-4">
                <button class="p-2 rounded-lg hover:bg-surface-darker text-slate-400 hover:text-white transition-colors" title="設定">
                    <span class="material-symbols-outlined">"settings"</span>
                </button>
                <div class="relative">
                    <button class="p-2 rounded-lg hover:bg-surface-darker text-slate-400 hover:text-white transition-colors" title="通知">
                        <span class="material-symbols-outlined">"notifications"</span>
                    </button>
                    <span class="absolute top-2 right-2 w-2 h-2 bg-primary rounded-full ring-2 ring-surface-dark"></span>
                </div>
                <div class="h-8 w-px bg-border-dark mx-1"></div>
                <button class="flex items-center gap-2 pl-2 hover:opacity-80 transition-opacity">
                    <div class="w-8 h-8 rounded-full bg-gradient-to-tr from-slate-700 to-slate-600 flex items-center justify-center text-white border border-border-dark">
                        <span class="text-xs font-bold">"JD"</span>
                    </div>
                </button>
            </div>
        </header>
    }
}

// ─── Main Panel ─────────────────────────────────────────────────────────────

#[component]
fn MainPanel(
    is_listening: ReadSignal<bool>,
    start_listening: impl Fn() + 'static + Copy,
    stop_listening: impl Fn() + 'static + Copy,
    set_stream_key_info: WriteSignal<Option<StreamKeyResponse>>,
    set_show_key_modal: WriteSignal<bool>,
    is_muted: ReadSignal<bool>,
    set_is_muted: WriteSignal<bool>,
    is_paused: ReadSignal<bool>,
    set_is_paused: WriteSignal<bool>,
    cam_on: ReadSignal<bool>,
    set_cam_on: WriteSignal<bool>,
    volume: ReadSignal<f64>,
    set_volume: WriteSignal<f64>,
    msg_count: ReadSignal<usize>,
) -> impl IntoView {
    view! {
        <main class="flex-1 flex flex-col p-6 overflow-y-auto min-w-0 bg-background-dark">
            <VideoPreview
                is_listening=is_listening
                is_muted=is_muted set_is_muted=set_is_muted
                is_paused=is_paused set_is_paused=set_is_paused
                volume=volume set_volume=set_volume
            />
            <ControlToolbar
                is_listening=is_listening
                start_listening=start_listening
                stop_listening=stop_listening
                set_stream_key_info=set_stream_key_info
                set_show_key_modal=set_show_key_modal
                cam_on=cam_on set_cam_on=set_cam_on
            />
            <StatsGrid msg_count=msg_count/>
        </main>
    }
}

// ─── Video Preview ──────────────────────────────────────────────────────────

#[component]
fn VideoPreview(
    #[allow(unused)]
    is_listening: ReadSignal<bool>,
    is_muted: ReadSignal<bool>,
    set_is_muted: WriteSignal<bool>,
    is_paused: ReadSignal<bool>,
    set_is_paused: WriteSignal<bool>,
    volume: ReadSignal<f64>,
    set_volume: WriteSignal<f64>,
) -> impl IntoView {
    // ページ読み込み時にプレイヤーを初期化（常時接続、OBSからのデータが来たら自動再生）
    Effect::new(move |_| {
        spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(500).await;
            init_mpegts_player("video-preview", "/api/live/stream");
        });
    });

    let on_play_pause = move |_: web_sys::MouseEvent| {
        let paused = toggle_video_play("video-preview");
        set_is_paused.set(paused);
    };

    let on_mute = move |_: web_sys::MouseEvent| {
        let muted = toggle_video_mute("video-preview");
        set_is_muted.set(muted);
    };

    let on_fullscreen = move |_: web_sys::MouseEvent| {
        request_fullscreen("video-container");
    };

    let on_volume_input = move |e: web_sys::Event| {
        let val: f64 = event_target_value(&e).parse().unwrap_or(0.66);
        set_volume.set(val);
        set_video_volume("video-preview", val);
        set_is_muted.set(false);
    };

    view! {
        <div
            id="video-container"
            class="relative bg-black rounded-xl overflow-hidden border border-border-dark group shadow-2xl mx-auto w-full"
            style="aspect-ratio: 16/9; max-height: calc(100vh - 320px); max-width: calc((100vh - 320px) * 16 / 9);"
        >
            <video
                id="video-preview"
                class="absolute inset-0 w-full h-full object-contain"
                autoplay=true
                muted=true
            ></video>

            // Top badges
            <div class="absolute top-4 left-4 flex gap-2 z-10">
                <span class="px-2 py-1 text-[10px] font-mono font-medium bg-black/80 text-primary rounded border border-white/5 backdrop-blur-md">"1080p60"</span>
                <span class="px-2 py-1 text-[10px] font-mono font-medium bg-black/80 text-slate-300 rounded border border-white/5 backdrop-blur-md">"6000 Kbps"</span>
            </div>

            // No signal placeholder (JS hides this when video starts playing)
            <div id="video-placeholder" class="absolute inset-0 w-full h-full flex flex-col items-center justify-center bg-[#050608] z-[5]">
                <div class="relative">
                    <div class="absolute -inset-4 bg-primary/5 rounded-full blur-xl animate-pulse"></div>
                    <div class="w-24 h-24 rounded-full bg-surface-darker border border-border-dark flex items-center justify-center mb-6 relative z-10">
                        <span class="material-symbols-outlined text-5xl text-slate-700">"videocam_off"</span>
                    </div>
                </div>
                <p class="text-lg font-medium tracking-wide text-slate-500 uppercase">"信号がありません"</p>
                <p class="text-sm text-slate-600 mt-2 font-mono">"ビデオソースを待機中"</p>
            </div>

            // Bottom controls (hover)
            <div class="absolute bottom-0 left-0 right-0 px-4 pb-3 pt-10 flex items-end justify-between bg-gradient-to-t from-black/80 via-transparent to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-300 pointer-events-none z-10">
                <div class="flex items-center gap-3 pointer-events-auto">
                    <button on:click=on_play_pause class="text-white hover:text-primary transition-colors">
                        <span class="material-symbols-outlined text-[22px]">
                            {move || if is_paused.get() { "play_arrow" } else { "pause" }}
                        </span>
                    </button>
                    <button on:click=on_mute class="text-white hover:text-primary transition-colors">
                        <span class="material-symbols-outlined text-[22px]">
                            {move || if is_muted.get() { "volume_off" } else { "volume_up" }}
                        </span>
                    </button>
                    <input
                        type="range" min="0" max="1" step="0.01"
                        prop:value=move || volume.get().to_string()
                        on:input=on_volume_input
                        class="w-20 h-1 accent-primary cursor-pointer"
                        style="appearance: auto;"
                    />
                </div>
                <div class="flex items-center gap-3 pointer-events-auto">
                    <button on:click=on_fullscreen class="text-white hover:text-primary transition-colors">
                        <span class="material-symbols-outlined text-[22px]">"fullscreen"</span>
                    </button>
                </div>
            </div>
        </div>
    }
}

// ─── Control Toolbar ────────────────────────────────────────────────────────

#[component]
fn ControlToolbar(
    is_listening: ReadSignal<bool>,
    start_listening: impl Fn() + 'static + Copy,
    stop_listening: impl Fn() + 'static + Copy,
    set_stream_key_info: WriteSignal<Option<StreamKeyResponse>>,
    set_show_key_modal: WriteSignal<bool>,
    cam_on: ReadSignal<bool>,
    set_cam_on: WriteSignal<bool>,
) -> impl IntoView {
    let on_generate_key = move |_: web_sys::MouseEvent| {
        spawn_local(async move {
            match services::stream_api::generate_stream_key().await {
                Ok(resp) => {
                    set_stream_key_info.set(Some(resp));
                    set_show_key_modal.set(true);
                }
                Err(e) => log::error!("Failed to generate stream key: {}", e),
            }
        });
    };

    let btn = "w-10 h-10 flex items-center justify-center rounded-lg hover:bg-surface-darker text-slate-400 hover:text-white transition-colors border border-transparent hover:border-border-dark";

    view! {
        <div class="mt-4 bg-surface-dark rounded-xl p-3 flex items-center justify-between border border-border-dark shadow-lg">
            <div class="flex items-center gap-2">
                // Mic
                <button
                    on:click=move |_| { if is_listening.get() { stop_listening(); } else { start_listening(); } }
                    class=move || if is_listening.get() {
                        "w-10 h-10 flex items-center justify-center rounded-lg bg-red-500/20 text-red-400 border border-red-500/30 transition-colors"
                    } else { btn }
                    title="マイクミュート"
                >
                    <span class="material-symbols-outlined text-[20px]">
                        {move || if is_listening.get() { "mic" } else { "mic_off" }}
                    </span>
                </button>
                // Camera
                <button
                    on:click=move |_| set_cam_on.update(|v| *v = !*v)
                    class=btn
                    title="カメラ"
                >
                    <span class="material-symbols-outlined text-[20px]">
                        {move || if cam_on.get() { "videocam" } else { "videocam_off" }}
                    </span>
                </button>
                <div class="w-px h-6 bg-border-dark mx-2"></div>
                // Screen share
                <button class=btn title="画面共有">
                    <span class="material-symbols-outlined text-[20px]">"screen_share"</span>
                </button>
                // Stream key
                <button on:click=on_generate_key class=btn title="ストリームキー">
                    <span class="material-symbols-outlined text-[20px]">"key"</span>
                </button>
                // Settings
                <button class=btn title="設定">
                    <span class="material-symbols-outlined text-[20px]">"tune"</span>
                </button>
            </div>
            <div class="flex items-center gap-4">
                // Timer
                <div class="flex items-center gap-3 px-4 py-2 bg-surface-darker rounded-lg border border-border-dark">
                    <span class="material-symbols-outlined text-slate-500 text-[18px]">"timer"</span>
                    <span class="font-mono text-slate-300 font-medium tracking-wide">"00:00:00"</span>
                </div>
                // Start/Stop button
                <button
                    on:click=move |_| { if is_listening.get() { stop_listening(); } else { start_listening(); } }
                    class=move || if is_listening.get() {
                        "bg-red-500 hover:bg-red-600 text-white px-6 py-2.5 rounded-lg font-bold flex items-center gap-2 transition-all shadow-[0_0_20px_rgba(239,68,68,0.15)] hover:shadow-[0_0_25px_rgba(239,68,68,0.3)] transform hover:-translate-y-0.5"
                    } else {
                        "bg-primary hover:bg-primary-hover text-black px-6 py-2.5 rounded-lg font-bold flex items-center gap-2 transition-all shadow-glow hover:shadow-[0_0_25px_rgba(16,185,129,0.3)] transform hover:-translate-y-0.5"
                    }
                >
                    <span class="material-symbols-outlined icon-fill">
                        {move || if is_listening.get() { "stop" } else { "play_arrow" }}
                    </span>
                    <span>{move || if is_listening.get() { "配信終了" } else { "配信開始" }}</span>
                </button>
            </div>
        </div>
    }
}

// ─── Stats Grid ─────────────────────────────────────────────────────────────

#[component]
fn StatsGrid(msg_count: ReadSignal<usize>) -> impl IntoView {
    view! {
        <div class="grid grid-cols-1 md:grid-cols-3 gap-6 mt-6">
            // AI Viewers
            <div class="bg-surface-dark border border-border-dark rounded-xl p-5 relative overflow-hidden group hover:border-border-dark/80 transition-colors">
                <div class="absolute top-0 right-0 p-4 opacity-50">
                    <div class="p-2 bg-accent-blue/10 rounded-lg">
                        <span class="material-symbols-outlined text-accent-blue text-xl">"smart_toy"</span>
                    </div>
                </div>
                <div class="flex flex-col h-full justify-between relative z-10">
                    <div>
                        <h3 class="text-xs font-semibold text-slate-500 uppercase tracking-wider mb-1">"AI視聴者"</h3>
                        <div class="flex items-baseline gap-2">
                            <span class="text-3xl font-bold text-white font-mono">"0"</span>
                            <span class="text-xs text-slate-500">"稼働中のボット"</span>
                        </div>
                    </div>
                    <div class="mt-4">
                        <div class="w-full bg-surface-darker rounded-full h-1.5 overflow-hidden">
                            <div class="bg-accent-blue h-1.5 rounded-full" style="width: 5%"></div>
                        </div>
                    </div>
                </div>
            </div>

            // Engagement
            <div class="bg-surface-dark border border-border-dark rounded-xl p-5 relative overflow-hidden group hover:border-border-dark/80 transition-colors">
                <div class="absolute top-0 right-0 p-4 opacity-50">
                    <div class="p-2 bg-accent-purple/10 rounded-lg">
                        <span class="material-symbols-outlined text-accent-purple text-xl">"forum"</span>
                    </div>
                </div>
                <div class="flex flex-col h-full justify-between relative z-10">
                    <div>
                        <h3 class="text-xs font-semibold text-slate-500 uppercase tracking-wider mb-1">"エンゲージメント"</h3>
                        <div class="flex items-baseline gap-2">
                            <span class="text-3xl font-bold text-white font-mono">{move || msg_count.get().to_string()}</span>
                            <span class="text-xs text-slate-500">"総アクション数"</span>
                        </div>
                    </div>
                    <div class="mt-4">
                        <div class="w-full bg-surface-darker rounded-full h-1.5 overflow-hidden">
                            <div class="bg-accent-purple h-1.5 rounded-full" style="width: 0%"></div>
                        </div>
                    </div>
                </div>
            </div>

            // Sentiment
            <div class="bg-surface-dark border border-border-dark rounded-xl p-5 relative overflow-hidden group hover:border-border-dark/80 transition-colors">
                <div class="absolute top-0 right-0 p-4 opacity-50">
                    <div class="p-2 bg-accent-pink/10 rounded-lg">
                        <span class="material-symbols-outlined text-accent-pink text-xl">"favorite"</span>
                    </div>
                </div>
                <div class="flex flex-col h-full justify-between relative z-10">
                    <div>
                        <h3 class="text-xs font-semibold text-slate-500 uppercase tracking-wider mb-1">"感情分析"</h3>
                        <div class="flex items-baseline gap-2">
                            <span class="text-3xl font-bold text-white font-mono">"--"</span>
                            <span class="text-xs text-slate-500">"普通"</span>
                        </div>
                    </div>
                    <div class="mt-5 flex items-center gap-3">
                        <span class="text-[10px] text-slate-500 font-mono">"ネガティブ"</span>
                        <div class="flex-1 h-1.5 bg-surface-darker rounded-full overflow-hidden flex">
                            <div class="h-full bg-accent-pink w-0"></div>
                            <div class="h-full bg-slate-700 w-full opacity-20"></div>
                            <div class="h-full bg-primary w-0"></div>
                        </div>
                        <span class="text-[10px] text-slate-500 font-mono">"ポジティブ"</span>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ─── Chat Panel ─────────────────────────────────────────────────────────────

#[component]
fn ChatPanel(
    messages: ReadSignal<Vec<ChatMessage>>,
    chat_input: ReadSignal<String>,
    set_chat_input: WriteSignal<String>,
    send_chat: impl Fn(String) + 'static + Copy,
) -> impl IntoView {
    let do_send = move || {
        let text = chat_input.get();
        if !text.trim().is_empty() {
            send_chat(text);
            set_chat_input.set(String::new());
        }
    };

    let on_send = move |_: web_sys::MouseEvent| { do_send(); };

    let on_keydown = move |e: web_sys::KeyboardEvent| {
        if e.key() == "Enter" && !e.shift_key() {
            e.prevent_default();
            do_send();
        }
    };

    view! {
        <aside class="w-80 border-l border-border-dark bg-surface-dark flex flex-col flex-shrink-0 z-10 shadow-xl">
            // Header
            <div class="h-14 border-b border-border-dark flex items-center justify-between px-4 bg-surface-dark">
                <div class="flex items-center gap-2">
                    <h2 class="font-bold text-sm text-white tracking-wide">"ライブチャット"</h2>
                    <span class="bg-primary/10 text-primary text-[10px] px-1.5 py-0.5 rounded font-mono border border-primary/20">"AI管理済み"</span>
                </div>
                <button class="text-slate-500 hover:text-white transition-colors">
                    <span class="material-symbols-outlined text-lg">"more_horiz"</span>
                </button>
            </div>

            // Messages
            <div class="flex-1 overflow-y-auto p-4 min-h-0">
                {move || {
                    let msgs = messages.get();
                    if msgs.is_empty() {
                        view! {
                            <div class="flex flex-col items-center justify-center h-full text-center">
                                <div class="w-16 h-16 bg-surface-darker rounded-full flex items-center justify-center mb-4 border border-border-dark">
                                    <span class="material-symbols-outlined text-3xl text-slate-600">"chat_bubble_outline"</span>
                                </div>
                                <p class="text-sm text-slate-400 font-medium mb-1">"まだメッセージはありません"</p>
                                <p class="text-xs text-slate-600 max-w-[200px]">"視聴者のメッセージとAI分析がここに表示されます。"</p>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div class="space-y-3">
                                {msgs.iter().enumerate().map(|(i, msg)| {
                                    let user = msg.user.clone();
                                    let text = msg.text.clone();
                                    let is_self = user == "You" || user == "Me";
                                    let palettes = [
                                        ("from-blue-500 to-cyan-400", "text-blue-400", "smart_toy", "shadow-blue-500/20"),
                                        ("from-purple-500 to-pink-500", "text-purple-400", "psychology", "shadow-purple-500/20"),
                                        ("from-orange-500 to-yellow-500", "text-orange-400", "rocket_launch", "shadow-orange-500/20"),
                                        ("from-emerald-500 to-teal-500", "text-emerald-400", "memory", "shadow-emerald-500/20"),
                                        ("from-red-500 to-pink-600", "text-red-400", "search", "shadow-red-500/20"),
                                        ("from-indigo-500 to-blue-600", "text-indigo-400", "school", "shadow-indigo-500/20"),
                                    ];
                                    let idx = if is_self { 0 } else { i % palettes.len() };
                                    let (grad, name_color, icon, shadow) = palettes[idx];

                                    let avatar_cls = format!(
                                        "size-7 rounded-md bg-gradient-to-br {} flex items-center justify-center shrink-0 shadow-sm {}",
                                        grad, shadow
                                    );
                                    let name_cls = format!("{} font-semibold text-xs", if is_self { "text-white" } else { name_color });

                                    view! {
                                        <div class="flex gap-2.5">
                                            <div class=avatar_cls>
                                                <span class="material-symbols-outlined text-white text-[14px]">
                                                    {if is_self { "person" } else { icon }}
                                                </span>
                                            </div>
                                            <div class="flex flex-col min-w-0">
                                                <span class=name_cls>{user}</span>
                                                <p class="text-slate-300 text-xs leading-relaxed mt-0.5 break-words">{text}</p>
                                            </div>
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                        }.into_any()
                    }
                }}
            </div>

            // Input
            <div class="p-4 border-t border-border-dark bg-surface-dark">
                <div class="relative group">
                    <textarea
                        class="w-full bg-surface-darker text-slate-200 text-sm rounded-xl border border-border-dark focus:border-primary focus:ring-1 focus:ring-primary py-3 pl-4 pr-12 resize-none overflow-hidden placeholder-slate-600 transition-all"
                        placeholder="視聴者に返信..."
                        rows="1"
                        prop:value=move || chat_input.get()
                        on:input=move |e| set_chat_input.set(event_target_value(&e))
                        on:keydown=on_keydown
                    ></textarea>
                    <button
                        on:click=on_send
                        class="absolute bottom-1.5 right-1.5 p-1.5 bg-primary hover:bg-primary-hover text-black rounded-lg transition-colors shadow-sm"
                    >
                        <span class="material-symbols-outlined text-lg block transform rotate-[-45deg] relative left-[1px] top-[1px]">"send"</span>
                    </button>
                </div>
                <div class="flex justify-between items-center mt-3">
                    <div class="flex gap-1">
                        <button class="p-1.5 rounded text-slate-500 hover:text-primary hover:bg-surface-darker transition-colors" title="絵文字">
                            <span class="material-symbols-outlined text-[18px]">"sentiment_satisfied"</span>
                        </button>
                        <button class="p-1.5 rounded text-slate-500 hover:text-primary hover:bg-surface-darker transition-colors" title="メンション">
                            <span class="material-symbols-outlined text-[18px]">"alternate_email"</span>
                        </button>
                    </div>
                    <span class="text-[10px] text-slate-600 font-mono">"ENTERで送信"</span>
                </div>
            </div>
        </aside>
    }
}

// ─── Stream Key Modal ───────────────────────────────────────────────────────

#[component]
fn StreamKeyModal(
    show_key_modal: ReadSignal<bool>,
    set_show_key_modal: WriteSignal<bool>,
    stream_key_info: ReadSignal<Option<StreamKeyResponse>>,
    set_stream_key_info: WriteSignal<Option<StreamKeyResponse>>,
) -> impl IntoView {
    let (server_copied, set_server_copied) = signal(false);
    let (key_copied, set_key_copied) = signal(false);
    let (is_generating, set_is_generating) = signal(false);

    // Auto-generate key when modal opens with no key
    Effect::new(move |_| {
        if show_key_modal.get() {
            // Fetch existing key first
            spawn_local(async move {
                match services::stream_api::get_stream_key().await {
                    Ok(resp) => {
                        if resp.stream_key.is_some() {
                            set_stream_key_info.set(Some(resp));
                        }
                    }
                    Err(e) => log::warn!("Could not fetch existing key: {}", e),
                }
            });
        }
        // Reset copied states when modal closes
        if !show_key_modal.get() {
            set_server_copied.set(false);
            set_key_copied.set(false);
        }
    });

    let on_generate = move |_: web_sys::MouseEvent| {
        set_is_generating.set(true);
        spawn_local(async move {
            match services::stream_api::generate_stream_key().await {
                Ok(resp) => {
                    set_stream_key_info.set(Some(resp));
                    set_key_copied.set(false);
                    set_server_copied.set(false);
                }
                Err(e) => log::error!("Failed to generate stream key: {}", e),
            }
            set_is_generating.set(false);
        });
    };

    let copy_with_feedback = move |text: String, set_copied: WriteSignal<bool>| {
        copy_to_clipboard(&text);
        set_copied.set(true);
        spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(2000).await;
            set_copied.set(false);
        });
    };

    move || {
        if !show_key_modal.get() {
            return view! { <div class="hidden"></div> }.into_any();
        }
        let info = stream_key_info.get();
        view! {
            <div class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50"
                 on:click=move |_| set_show_key_modal.set(false)>
                <div class="bg-surface-dark rounded-2xl p-6 w-[500px] border border-border-dark shadow-2xl"
                     on:click=move |e: web_sys::MouseEvent| e.stop_propagation()>
                    <div class="flex items-center justify-between mb-5">
                        <div class="flex items-center gap-2.5">
                            <div class="p-2 bg-primary/10 rounded-lg">
                                <span class="material-symbols-outlined text-primary text-xl">"key"</span>
                            </div>
                            <h2 class="text-lg font-bold text-white">"ストリームキー設定"</h2>
                        </div>
                        <button
                            on:click=move |_| set_show_key_modal.set(false)
                            class="p-1.5 rounded-lg hover:bg-surface-darker text-slate-500 hover:text-white transition-colors"
                        >
                            <span class="material-symbols-outlined text-[20px]">"close"</span>
                        </button>
                    </div>

                    {match info {
                        Some(ref resp) => {
                            let key = resp.stream_key.clone().unwrap_or_default();
                            let server = resp.server_url.clone();
                            let server_for_copy = server.clone();
                            let key_for_copy = key.clone();
                            view! {
                                <div class="space-y-4">
                                    // Server URL
                                    <div>
                                        <label class="text-xs font-medium text-slate-400 block mb-1.5">"サーバーURL"</label>
                                        <div class="flex gap-2">
                                            <div class="flex-1 bg-surface-darker p-3 rounded-lg font-mono text-xs break-all border border-border-dark text-slate-300 select-all">
                                                {server}
                                            </div>
                                            <button
                                                on:click=move |_| copy_with_feedback(server_for_copy.clone(), set_server_copied)
                                                class="px-3 rounded-lg border border-border-dark hover:border-primary/50 bg-surface-darker hover:bg-primary/10 text-slate-400 hover:text-primary transition-all flex items-center gap-1.5 shrink-0"
                                            >
                                                <span class="material-symbols-outlined text-[16px]">
                                                    {move || if server_copied.get() { "check" } else { "content_copy" }}
                                                </span>
                                                <span class="text-xs font-medium">
                                                    {move || if server_copied.get() { "コピー済み" } else { "コピー" }}
                                                </span>
                                            </button>
                                        </div>
                                    </div>

                                    // Stream Key
                                    <div>
                                        <label class="text-xs font-medium text-slate-400 block mb-1.5">"ストリームキー"</label>
                                        <div class="flex gap-2">
                                            <div class="flex-1 bg-surface-darker p-3 rounded-lg font-mono text-xs break-all border border-border-dark text-slate-300 select-all">
                                                {key}
                                            </div>
                                            <button
                                                on:click=move |_| copy_with_feedback(key_for_copy.clone(), set_key_copied)
                                                class="px-3 rounded-lg border border-border-dark hover:border-primary/50 bg-surface-darker hover:bg-primary/10 text-slate-400 hover:text-primary transition-all flex items-center gap-1.5 shrink-0"
                                            >
                                                <span class="material-symbols-outlined text-[16px]">
                                                    {move || if key_copied.get() { "check" } else { "content_copy" }}
                                                </span>
                                                <span class="text-xs font-medium">
                                                    {move || if key_copied.get() { "コピー済み" } else { "コピー" }}
                                                </span>
                                            </button>
                                        </div>
                                    </div>

                                    <div class="bg-surface-darker/50 rounded-lg p-3 border border-border-dark/50">
                                        <p class="text-[11px] text-slate-500 leading-relaxed flex items-start gap-2">
                                            <span class="material-symbols-outlined text-[14px] text-slate-600 mt-0.5 shrink-0">"info"</span>
                                            "OBSの設定 → 配信 → サービス「カスタム」を選択し、上記のサーバーURLとストリームキーを入力してください。"
                                        </p>
                                    </div>
                                </div>
                            }.into_any()
                        }
                        None => view! {
                            <div class="flex flex-col items-center py-8">
                                <div class="w-16 h-16 bg-surface-darker rounded-full flex items-center justify-center mb-4 border border-border-dark">
                                    <span class="material-symbols-outlined text-3xl text-slate-600">"vpn_key_off"</span>
                                </div>
                                <p class="text-sm text-slate-400 font-medium mb-1">"ストリームキーが未生成です"</p>
                                <p class="text-xs text-slate-600">"配信を開始するにはキーを生成してください"</p>
                            </div>
                        }.into_any(),
                    }}

                    <div class="mt-5 flex justify-between items-center">
                        <button
                            on:click=on_generate
                            disabled=move || is_generating.get()
                            class="px-4 py-2 rounded-lg bg-primary hover:bg-primary-hover text-black text-xs font-bold transition-colors flex items-center gap-1.5 disabled:opacity-50"
                        >
                            <span class="material-symbols-outlined text-[16px]">"refresh"</span>
                            {move || if is_generating.get() { "生成中..." } else if stream_key_info.get().is_some() { "再生成" } else { "キーを生成" }}
                        </button>
                        <button
                            on:click=move |_| set_show_key_modal.set(false)
                            class="px-5 py-2 rounded-lg bg-surface-darker hover:bg-border-dark text-slate-400 text-xs font-bold border border-border-dark transition-colors"
                        >"閉じる"</button>
                    </div>
                </div>
            </div>
        }.into_any()
    }
}
