# VYuber - VTuber配信スタジオ

**Rust + Leptos WASM** で構築されたVTuber配信スタジオアプリケーションです。OBSからのRTMPストリームを受信・表示できます。

## ✨ 特徴

- 🦀 **完全Rust実装** - バックエンド（Axum）とフロントエンド（Leptos WASM）
- ⚡ **高速・軽量** - WASMによる高パフォーマンス
- 🎥 RTMPサーバーによるストリーム受信
- 🔑 OBS連携（ストリームキー生成）
- 📹 リアルタイム映像プレビュー
- 💬 チャットオーバーレイ（AI連携準備完了）
- 🎤 音声認識UI

## 🚀 クイックスタート

### 1. 必要なツール

- [Rust](https://rustup.rs/) (最新版)
- [Trunk](https://trunkrs.dev/) - WASM ビルドツール
  ```bash
  cargo install trunk
  ```
- Node.js (npm scriptsとInfisical用)

### 2. 開発サーバーの起動

```bash
cd vyuber-rust
npm run dev
```

これにより以下が起動します:
- **Axumサーバー**: http://localhost:3000
- **RTMPサーバー**: rtmp://localhost:1935
- **Leptos UI**: http://localhost:3000 で自動配信

ブラウザで **http://localhost:3000** を開きます。

### 3. OBSからの配信設定

#### ステップ1: ストリームキーの生成

1. UIの「ストリームキー生成」ボタンをクリック
2. 生成されたストリームキーをコピー

#### ステップ2: OBSの設定

1. OBSを開く
2. 「設定」→「配信」を選択
3. サービス: **カスタム**
4. サーバー: `rtmp://localhost:1935/live`
5. ストリームキー: UIで生成したキーを貼り付け
6. 「OK」をクリック

**⚠️ 低遅延配信のための設定**

1. 「設定」→「出力」を開く
2. **出力モード**: 「詳細」に変更
3. **配信**タブで以下を設定:
   - **キーフレーム間隔**: `1` (秒)
   - これにより遅延が大幅に短縮されます

#### ステップ3: 配信開始

1. OBSで「配信開始」をクリック
2. UIのビデオプレビューに映像が表示されます
3. 配信が開始されます 🎉

## 🛠️ 開発

### ホットリロード

ファイル変更時に自動再起動:

```bash
npm run dev:watch
```

### ビルド

```bash
# フロントエンドのビルド
trunk build --release

# バックエンドのビルド
cargo build --release
```

### スクリプト一覧

- `npm run dev` - 開発サーバー起動
- `npm run dev:watch` - ホットリロード有効
- `npm run build` - リリースビルド
- `npm test` - テスト実行

## 🔧 環境変数

Infisicalで管理されます。必要な変数:

```env
GEMINI_API_KEY=your_api_key_here  # AI機能用
RTMP_PORT=1935                    # RTMPポート
HTTP_FLV_PORT=8888                # HTTP-FLVポート
```

## 🏗️ プロジェクト構造

```
vyuber-rust/
├── crates/
│   ├── vyuber-backend/     # Axumバックエンド
│   │   ├── src/
│   │   │   ├── api/        # APIエンドポイント
│   │   │   ├── rtmp/       # RTMPサーバー
│   │   │   └── services/   # ビジネスロジック
│   │   └── static/         # 静的ファイル（ビルド成果物）
│   ├── vyuber-frontend/    # Leptosフロントエンド（WASM）
│   │   └── src/
│   │       ├── lib.rs      # メインUI
│   │       └── services/   # API通信
│   └── vyuber-shared/      # 共通型定義
├── scripts/                # 開発スクリプト
└── package.json           # npm scripts
```

## 🦀 技術スタック

### バックエンド
- **Axum** - 非同期Webフレームワーク
- **Tokio** - 非同期ランタイム
- **Tower** - ミドルウェア
- **Serde** - シリアライゼーション

### フロントエンド
- **Leptos** - リアクティブUIフレームワーク
- **WASM** - WebAssembly
- **Tailwind CSS** - スタイリング
- **mpegts.js** - 動画プレーヤー

### 開発ツール
- **Trunk** - WASMビルドツール
- **Infisical** - 環境変数管理

## 📚 ドキュメント

詳細は `vyuber-rust/README.md` を参照してください。

## 📝 ライセンス

Private
