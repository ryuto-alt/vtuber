import { NextResponse } from 'next/server';

export async function POST() {
    try {
        const apiKey = process.env.OPEN_ROUTER_API_KEY;
        if (!apiKey) {
            return NextResponse.json({ error: 'API key not configured' }, { status: 500 });
        }

        const response = await fetch('https://openrouter.ai/api/v1/chat/completions', {
            method: 'POST',
            headers: {
                'Authorization': `Bearer ${apiKey}`,
                'Content-Type': 'application/json',
                'HTTP-Referer': 'https://vyuber.local', // Optional for OpenRouter
                'X-Title': 'Vyuber Local'
            },
            body: JSON.stringify({
                model: "openai/gpt-3.5-turbo", // or a cheap efficient model
                messages: [
                    {
                        role: "system",
                        content: `You are simulating a live stream chat viewer on a Japanese stream.
            
            GOAL: Generate ONE single short, NATURAL chat message (max 30 characters).
            Avoid forced or stereotypical slang (like "Icchi", "Ngo", "Mensu").
            
            TONE: Extremely casual, reactive, short, and authentic to modern Japanese live streams (YouTube/Twitch/Niconico).
            
            Vary personalities/styles:
            - Short Reactions (Most common): "草", "！？", "あ", "おお", "え？", "ま？", "それな"
            - The "Tsukkomi" (Sharp retort): "おいｗ", "何してんねん", "フラグ回収", "は？"
            - The "Praiser" (Simple): "うま", "888888", "天才か", "かわいい"
            - The "Casual" (Conversational): "今日何時まで？", "初見", "声いいな", "音ズレかも"
            - Laughing: "ｗｗｗ", "大草原", "草生える"

            Key Rules:
            - Keep it SHORT. Real viewers often type just 1-5 characters.
            - No polite Japanese (Desu/Masu) unless acting as a very polite new viewer.
            - No forced "NanJ" specific jargon unless it fits naturally as general slang (like "kusa").
            - Use occasional emojis but mostly text/symbols.

            Examples:
            - "草"
            - "え、これマジ？"
            - "ｗｗｗｗｗｗ"
            - "画面かくついてる"
            - "初見です"
            - "あｗ"
            - "うま！"
            - "何やってんのｗｗ"
            
            Authenticate as a JSON object with these fields:
            - user: A random Japanese nickname (often highly casual, e.g., "名無し", "tomato", "aa", "猫")
            - text: The message content in Natural Japanese Internet Slang
            - color: A tailwind text color class.
            
            Output ONLY the JSON object, no markdown code fence.`
                    }
                ],
                temperature: 0.9,
                max_tokens: 60
            })
        });

        if (!response.ok) {
            const errorText = await response.text();
            console.error('OpenRouter API Error:', errorText);
            return NextResponse.json({ error: 'Failed to fetch from AI' }, { status: response.status });
        }

        const data = await response.json();
        let content = data.choices[0]?.message?.content;

        // Parse JSON if it came as a string
        try {
            if (typeof content === 'string') {
                // Remove markdown code blocks if present
                content = content.replace(/```json/g, '').replace(/```/g, '').trim();
                content = JSON.parse(content);
            }
        } catch (e) {
            console.error('JSON Parse Error:', e);
            // Fallback
            content = { user: "System", text: "Error generating message", color: "text-red-500" };
        }

        return NextResponse.json(content);

    } catch (error) {
        console.error('Chat API Error:', error);
        return NextResponse.json({ error: 'Internal Server Error' }, { status: 500 });
    }
}
