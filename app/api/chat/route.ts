import { GoogleGenerativeAI } from "@google/generative-ai";
import { NextResponse } from "next/server";

// APIキーの読み込み
const genAI = new GoogleGenerativeAI(process.env.GEMINI_API_KEY!);

export async function POST(req: Request) {
  try {
    const { message } = await req.json();

    // ★修正ポイント:
    // 特定のバージョン(2.0など)を指定せず、エイリアス "gemini-flash-latest" を使用します。
    // これにより、その時点で利用可能な最新・最適なFlashモデル（おそらくgemini-2.5-flash等）が自動選択されます。
    const model = genAI.getGenerativeModel({ 
      model: "gemini-flash-latest", 
      generationConfig: { responseMimeType: "application/json" } 
    });

    const prompt = `
      あなたはライブ配信の視聴者です。配信者の発言に対して、以下の5つの異なる人格になりきって、それぞれの反応を生成してください。
      
      ## 配信者の発言:
      "${message}"

      ## 生成する5つの人格:
      1. 全肯定ファン (青色系): とにかく褒める。語彙力低め。
      2. 初見さん (紫色系): 状況がわかっていない、または純粋な質問。
      3. 辛口コメント (オレンジ色系): 少し批判的、または技術的なツッコミ。
      4. スパム/ネタ勢 (ピンク色系): 絵文字多め、または文脈と関係ない勢いだけのコメント。
      5. 古参 (緑色系): "おっ" "いつもの" など、慣れている感。

      ## 出力形式 (JSON Array):
      [
        { "user": "名前", "text": "コメント内容", "color": "text-blue-400" },
        ...
      ]
      
      必ずValidなJSON配列のみを返してください。
    `;

    const result = await model.generateContent(prompt);
    const responseText = result.response.text();
    
    // JSONとしてパースして返す
    const jsonResponse = JSON.parse(responseText);

    return NextResponse.json({ comments: jsonResponse });
  } catch (error) {
    console.error("Gemini API Error:", error);
    return NextResponse.json({ error: "API Error" }, { status: 500 });
  }
}