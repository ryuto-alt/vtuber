require('dotenv').config({ path: '.env.local' });
const { GoogleGenerativeAI } = require("@google/generative-ai");

async function listModels() {
  const genAI = new GoogleGenerativeAI(process.env.GEMINI_API_KEY);
  
  try {
    // APIからモデル一覧を取得
    // ※SDKのバージョンによっては直接リスト取得が難しい場合があるため、
    // ここでは単純に接続テストも兼ねて実行します
    console.log("Checking API Key:", process.env.GEMINI_API_KEY ? "OK" : "MISSING");

    // curlコマンドと同等のリクエストを送って確認
    const url = `https://generativelanguage.googleapis.com/v1beta/models?key=${process.env.GEMINI_API_KEY}`;
    const response = await fetch(url);
    const data = await response.json();

    if (data.models) {
      console.log("\n=== 利用可能なモデル一覧 ===");
      data.models.forEach(model => {
        // generateContent（チャット生成）に対応しているモデルだけ表示
        if (model.supportedGenerationMethods.includes("generateContent")) {
          console.log(`- ${model.name.replace('models/', '')}`);
        }
      });
      console.log("==========================\n");
    } else {
      console.error("エラー:", data);
    }

  } catch (error) {
    console.error("接続エラー:", error);
  }
}

listModels();