import { NextResponse } from "next/server";

export async function GET() {
  const apiKey = process.env.GEMINI_API_KEY;

  return NextResponse.json({
    hasKey: !!apiKey,
    keyLength: apiKey?.length || 0,
    keyPrefix: apiKey?.substring(0, 10) || 'NOT SET',
    allEnvKeys: Object.keys(process.env).filter(k => k.includes('GEMINI') || k.includes('API'))
  });
}
