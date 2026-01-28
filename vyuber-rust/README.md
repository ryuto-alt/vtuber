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

`.env.local` ファイル（親ディレクトリ）から以下の環境変数を読み込みます：

```env
GEMINI_API_KEY=your_api_key_here
RTMP_PORT=1935
HTTP_FLV_PORT=8888
```

## 実装状況

- [x] プロジェクト構造作成
- [x] 共通型定義
- [x] バックエンド基盤（Axum）
- [x] ストリームキーAPI
- [ ] Gemini API連携
- [ ] RTMP/動画配信
- [ ] フロントエンド（Leptos）
- [ ] 音声認識
- [ ] チャット機能
- [ ] 動画プレビュー

## ライセンス

Private
