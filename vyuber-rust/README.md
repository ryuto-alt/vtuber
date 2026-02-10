# VYuber Rust

VTuber配信アプリケーションのRust実装版です。

## 技術スタック

- **フロントエンド**: Leptos (Rust SPA, WASM)
- **バックエンド**: Axum (非同期Webフレームワーク)
- **RTMP**: sheave + FFmpeg
- **音声認識**: Web Speech API (wasm-bindgen)
- **AI**: Google Generative AI (Gemini)

## プロジェクト構成

```
vyuber-rust/
├── crates/
│   ├── vyuber-backend/    # Axumサーバー
│   ├── vyuber-frontend/   # Leptosフロントエンド
│   └── vyuber-shared/     # 共通型定義
└── scripts/               # ビルド・開発スクリプト
```

## 開発環境セットアップ

### 必要なツール

```bash
# Rustツールチェーン
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# WASM target
rustup target add wasm32-unknown-unknown

# Trunk (Leptosビルドツール)
cargo install trunk

# cargo-watch (ホットリロード)
cargo install cargo-watch
```

### GPU加速 (オプション)

**NVIDIA GPU (GTX 1060以上) で10-30倍高速化が可能です。**

#### 必要な環境
1. **NVIDIA GPU**: GTX 1060 / RTX 2060 以上推奨
2. **CUDA Toolkit**: 11.x または 12.x
   - ダウンロード: https://developer.nvidia.com/cuda-downloads
3. **cuDNN**: CUDA Toolkitに対応するバージョン
   - ダウンロード: https://developer.nvidia.com/cudnn

#### セットアップ手順
1. CUDA Toolkitをインストール
2. システム環境変数に追加:
   ```
   CUDA_PATH=C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v12.x
   ```
3. Whisperモデルをダウンロード:
   ```bash
   download-model.bat
   # → [3] small または [4] medium を選択
   ```
4. ビルド・実行:
   ```bash
   cargo build --release
   npm run dev
   ```

GPU検出は自動で行われ、利用可能な場合はログに表示されます：
```
✓ Whisper model loaded on GPU: models/ggml-medium.bin
```

### ビルド

```bash
# バックエンドのみ
cargo build --package vyuber-backend

# フロントエンド
cd crates/vyuber-frontend
trunk build

# 全体
cargo build
```

### 実行

#### npm scripts使用（最も簡単）

```bash
cd vyuber-rust

# 開発サーバー起動
npm run dev

# ホットリロード有効（ファイル変更時に自動再起動）
npm run dev:watch

# ビルド
npm run build
```

#### 開発スクリプト使用

```bash
# Windows PowerShell
.\scripts\dev.ps1

# Linux/Mac/Git Bash
./scripts/dev.sh

# ホットリロード有効（ファイル変更時に自動再起動）
# Windows
.\scripts\dev-watch.ps1

# Linux/Mac
./scripts/dev-watch.sh
```

#### 直接実行

```bash
# 開発環境（Infisical使用）
cd vyuber-rust
npx @infisical/cli run -- cargo run --release

# 本番
./target/release/vyuber-backend
```

## 環境変数

`.env.example` をコピーして `.env` ファイルを作成し、以下の環境変数を設定します：

```env
# Whisper音声認識モデル（download-model.batでダウンロード）
# WHISPER_MODEL_PATH=models/ggml-small.bin  # CPU環境向け
WHISPER_MODEL_PATH=models/ggml-medium.bin  # 推奨（GPU環境）

GEMINI_API_KEY=your_api_key_here
RTMP_PORT=1935
HTTP_FLV_PORT=8888
LOG_DIR=logs
```

### Whisperモデルの選択

| モデル | サイズ | 精度 | 速度 (CPU) | 速度 (GPU) | 推奨環境 |
|--------|-------|------|-----------|-----------|---------|
| tiny   | 75MB  | 低   | 0.3秒/10秒 | 0.03秒/10秒 | 非推奨 |
| base   | 142MB | 中   | 0.8秒/10秒 | 0.06秒/10秒 | CPU専用 |
| small | 466MB | 高 | 1.2秒/10秒 | 0.08秒/10秒 | CPU環境 |
| **medium** | **1.5GB** | **最高** | **3秒/10秒** | **0.15秒/10秒** | **推奨（GPU）** |

**推奨設定：**
- **CPU環境**: `small` または `base`
- **GPU環境 (GTX 1060以上)**: `medium`（推奨）

## 実装状況

- [x] プロジェクト構造作成
- [x] 共通型定義
- [x] バックエンド基盤（Axum）
- [x] ストリームキーAPI
- [x] **Whisper音声認識 (CPU/GPU対応)**
- [x] **言い間違え・噛み自動修正**
- [x] **リアルタイム文字起こし (WebSocket)**
- [x] チャット機能
- [ ] Gemini API連携
- [ ] RTMP/動画配信
- [ ] フロントエンド拡張
- [ ] 動画プレビュー

## 音声認識機能の特徴

### 🎯 高精度な文字起こし
- **Whisper (OpenAI)** ベースのローカル音声認識
- **VTuber配信特化**: 専門用語（スパチャ、メンシ、コラボ等）に対応
- **GPU加速**: NVIDIA GPU使用時は10-30倍高速化

### 🔧 言い間違え・噛み自動修正
1. **フィラー除去**: 「あー」「えー」「んー」「えっと」等を自動削除
2. **繰り返し統合**: 「そうそう、そう」→「そう」
3. **言い直し検出**: 「今日はじゃなくて明日は」→「明日は」
4. **未知語補完**: 形態素解析で音韻的に類似した語を補完

### ⚡ リアルタイム性
- **遅延**: 2-3秒 (CPU) / 0.9-1.5秒 (GPU)
- **沈黙検出**: 2.5秒の沈黙で自動確定
- **ストリーミング**: WebSocketで逐次送信

## ライセンス

Private
