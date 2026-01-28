# VYuber - VTuber配信スタジオ

Next.jsで構築されたVTuber配信スタジオアプリケーションです。OBSからのRTMPストリームを受信・表示できます。

## 機能

- RTMPサーバーによるストリーム受信
- OBS連携（ストリームキー生成）
- リアルタイム映像プレビュー
- チャットオーバーレイ
- 録画コントロール

## セットアップ

### 1. 依存関係のインストール

```bash
npm install
```

### 2. 開発サーバーの起動

次のコマンドで、RTMPサーバーとNext.jsアプリが同時に起動します:

```bash
npm run dev
```

これにより以下が起動します:
- RTMPサーバー: ポート1935
- HLS配信サーバー: ポート8000
- Next.jsアプリ: http://localhost:3000

ブラウザで [http://localhost:3000](http://localhost:3000) を開きます。

### 3. OBSからの配信設定

#### ステップ1: ストリームキーの生成

1. アプリ下部の「ストリームキー生成」ボタンをクリック
2. 「接続情報を表示」ボタンをクリックして、サーバーURLとストリームキーを確認

#### ステップ2: OBSの設定

1. OBSを開く
2. 「設定」→「配信」を選択
3. サービス: **カスタム**
4. サーバー: アプリで表示された **サーバーURL** をコピー（例: `rtmp://localhost:1935/live`）
5. ストリームキー: アプリで表示された **ストリームキー** をコピー
6. 「OK」をクリック

**⚠️ 重要: 低遅延配信のための設定**

OBSでプレビューの遅延を最小化するため、以下の設定を行ってください:

1. 「設定」→「出力」を開く
2. **出力モード**: 「詳細」に変更
3. **配信**タブで以下を設定:
   - **キーフレーム間隔**: `1` (秒) に設定
   - これにより遅延が5秒以内に短縮されます

#### ステップ3: 配信開始

1. OBSで「配信開始」をクリック
2. アプリのビデオプレビューに映像が表示されます
3. 「LIVE」インジケーターが表示されれば成功です

## 環境変数

`.env.local` ファイルで以下の設定を変更できます:

```env
RTMP_PORT=1935                  # RTMPサーバーのポート
HTTP_FLV_PORT=8000             # HLS配信のポート
NEXT_PUBLIC_HTTP_FLV_PORT=8000 # クライアント側のポート設定
```

## 技術スタック

- **フレームワーク**: Next.js 16 (App Router)
- **UI**: React 19, Tailwind CSS
- **ストリーミング**:
  - Node Media Server (RTMPサーバー)
  - HLS.js (ブラウザ再生)
- **アイコン**: Lucide React

## Learn More

To learn more about Next.js, take a look at the following resources:

- [Next.js Documentation](https://nextjs.org/docs) - learn about Next.js features and API.
- [Learn Next.js](https://nextjs.org/learn) - an interactive Next.js tutorial.

You can check out [the Next.js GitHub repository](https://github.com/vercel/next.js) - your feedback and contributions are welcome!

## Deploy on Vercel

The easiest way to deploy your Next.js app is to use the [Vercel Platform](https://vercel.com/new?utm_medium=default-template&filter=next.js&utm_source=create-next-app&utm_campaign=create-next-app-readme) from the creators of Next.js.

Check out our [Next.js deployment documentation](https://nextjs.org/docs/app/building-your-application/deploying) for more details.
