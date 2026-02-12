use std::path::PathBuf;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tracing::{info, warn, error, debug};

/// MediaMTXプロセスを起動・管理する
pub struct MediaMtxProcess {
    child: Option<Child>,
    exe_path: PathBuf,
    config_path: PathBuf,
}

impl MediaMtxProcess {
    /// MediaMTXの実行ファイルと設定ファイルのパスを探索
    pub fn new() -> Self {
        // 実行ファイルの場所を探す
        let exe_path = find_mediamtx_exe();
        let config_path = find_mediamtx_config();

        info!("[MediaMTX] exe: {}", exe_path.display());
        info!("[MediaMTX] config: {}", config_path.display());

        Self {
            child: None,
            exe_path,
            config_path,
        }
    }

    /// MediaMTXを起動
    pub async fn start(&mut self) -> Result<(), String> {
        if self.child.is_some() {
            warn!("[MediaMTX] Already running, stopping first...");
            self.stop().await;
        }

        if !self.exe_path.exists() {
            return Err(format!("MediaMTX executable not found: {}", self.exe_path.display()));
        }

        info!("[MediaMTX] Starting process...");

        let mut child = Command::new(&self.exe_path)
            .arg(&self.config_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| format!("Failed to start MediaMTX: {}", e))?;

        info!("[MediaMTX] Process started (pid: {:?})", child.id());

        // stdout/stderrをログに転送
        if let Some(stdout) = child.stdout.take() {
            tokio::spawn(async move {
                let mut reader = BufReader::new(stdout).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    if line.contains("ERR") {
                        error!("[MediaMTX] {}", line);
                    } else if line.contains("WAR") {
                        warn!("[MediaMTX] {}", line);
                    } else if line.contains("DBG") || line.contains("DEB") {
                        debug!("[MediaMTX] {}", line);
                    } else {
                        info!("[MediaMTX] {}", line);
                    }
                }
            });
        }
        if let Some(stderr) = child.stderr.take() {
            tokio::spawn(async move {
                let mut reader = BufReader::new(stderr).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    error!("[MediaMTX-stderr] {}", line);
                }
            });
        }

        self.child = Some(child);

        // MediaMTXの起動を少し待つ
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        Ok(())
    }

    /// MediaMTXを停止
    pub async fn stop(&mut self) {
        if let Some(mut child) = self.child.take() {
            info!("[MediaMTX] Stopping process...");
            match child.kill().await {
                Ok(_) => info!("[MediaMTX] Process stopped"),
                Err(e) => warn!("[MediaMTX] Failed to stop: {}", e),
            }
        }
    }

    /// MediaMTXが動作中か確認
    pub fn is_running(&mut self) -> bool {
        if let Some(ref mut child) = self.child {
            match child.try_wait() {
                Ok(None) => true,  // まだ動作中
                Ok(Some(status)) => {
                    warn!("[MediaMTX] Process exited with: {}", status);
                    false
                }
                Err(e) => {
                    error!("[MediaMTX] Failed to check status: {}", e);
                    false
                }
            }
        } else {
            false
        }
    }
}

impl Drop for MediaMtxProcess {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            // 同期的にkillを試みる（best effort）
            let _ = child.start_kill();
        }
    }
}

/// MediaMTXバックグラウンドタスクを起動
pub async fn start_mediamtx() -> Result<(), String> {
    let mut process = MediaMtxProcess::new();
    process.start().await?;

    // バックグラウンドでMediaMTXの死活監視
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            if !process.is_running() {
                warn!("[MediaMTX] Process died, restarting...");
                match process.start().await {
                    Ok(_) => info!("[MediaMTX] Restarted successfully"),
                    Err(e) => error!("[MediaMTX] Failed to restart: {}", e),
                }
            }
        }
    });

    Ok(())
}

/// MediaMTX実行ファイルを探す
fn find_mediamtx_exe() -> PathBuf {
    // 環境変数で指定
    if let Ok(path) = std::env::var("MEDIAMTX_PATH") {
        return PathBuf::from(path);
    }

    // プロジェクトルートからの相対パス
    let candidates = [
        "mediamtx/mediamtx.exe",
        "mediamtx/mediamtx",
        "../mediamtx/mediamtx.exe",
        "../mediamtx/mediamtx",
    ];

    for candidate in &candidates {
        let p = PathBuf::from(candidate);
        if p.exists() {
            return p;
        }
    }

    // 現在のexeと同じディレクトリを探す
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let p = dir.join("mediamtx").join("mediamtx.exe");
            if p.exists() {
                return p;
            }
        }
    }

    // デフォルト
    PathBuf::from("mediamtx/mediamtx.exe")
}

/// MediaMTX設定ファイルを探す
fn find_mediamtx_config() -> PathBuf {
    if let Ok(path) = std::env::var("MEDIAMTX_CONFIG") {
        return PathBuf::from(path);
    }

    let candidates = [
        "mediamtx/vyuber-mediamtx.yml",
        "../mediamtx/vyuber-mediamtx.yml",
        "mediamtx/mediamtx.yml",
        "../mediamtx/mediamtx.yml",
    ];

    for candidate in &candidates {
        let p = PathBuf::from(candidate);
        if p.exists() {
            return p;
        }
    }

    PathBuf::from("mediamtx/vyuber-mediamtx.yml")
}
