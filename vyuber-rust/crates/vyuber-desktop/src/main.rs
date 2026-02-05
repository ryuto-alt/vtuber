#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

fn main() {
    // ログをファイルとコンソール両方に出力
    let log_dir = std::env::var("LOG_DIR").unwrap_or_else(|_| "logs".to_string());
    std::fs::create_dir_all(&log_dir).ok();
    let file_appender = tracing_appender::rolling::daily(&log_dir, "vyuber.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(fmt::layer().with_ansi(true))
        .with(fmt::layer().with_ansi(false).with_writer(non_blocking))
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|_app| {
            // ポート3000が既に使用中かチェック（開発モードでは既に起動済み）
            let port_in_use = std::net::TcpStream::connect("127.0.0.1:3000").is_ok();

            if port_in_use {
                tracing::info!("Backend server already running on port 3000");
            } else {
                // Axumバックエンドサーバーをバックグラウンドタスクとして起動
                tracing::info!("Starting backend server...");
                tauri::async_runtime::spawn(async {
                    if let Err(e) = vyuber_backend::start_server().await {
                        tracing::error!("Backend server error: {}", e);
                    }
                });

                // サーバーのポートバインドを待つ
                std::thread::sleep(std::time::Duration::from_millis(500));
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
