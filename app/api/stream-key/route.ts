import { NextResponse } from 'next/server';
import { v4 as uuidv4 } from 'uuid';

// Simple in-memory storage for stream key
// Note: The RTMP server runs in a separate process (server.js)
// and currently accepts any stream key in development mode
let currentStreamKey: string | null = null;

export async function POST() {
  try {
    // Generate new stream key
    currentStreamKey = uuidv4().replace(/-/g, '');

    const rtmpPort = parseInt(process.env.RTMP_PORT || '1935');
    const serverUrl = `rtmp://localhost:${rtmpPort}/live`;

    return NextResponse.json({
      streamKey: currentStreamKey,
      serverUrl,
      fullUrl: `${serverUrl}/${currentStreamKey}`
    });
  } catch (error) {
    console.error('[API] Failed to generate stream key:', error);
    return NextResponse.json(
      { message: 'Failed to generate stream key', error: String(error) },
      { status: 500 }
    );
  }
}

export async function GET() {
  try {
    const rtmpPort = parseInt(process.env.RTMP_PORT || '1935');
    const serverUrl = `rtmp://localhost:${rtmpPort}/live`;

    if (!currentStreamKey) {
      return NextResponse.json({
        streamKey: null,
        serverUrl,
        fullUrl: null
      });
    }

    return NextResponse.json({
      streamKey: currentStreamKey,
      serverUrl,
      fullUrl: `${serverUrl}/${currentStreamKey}`
    });
  } catch (error) {
    return NextResponse.json(
      { message: 'Failed to get stream key', error: String(error) },
      { status: 500 }
    );
  }
}

export async function DELETE() {
  try {
    // Clear the current stream key
    currentStreamKey = null;

    return NextResponse.json({
      message: 'Stream key deleted successfully'
    });
  } catch (error) {
    return NextResponse.json(
      { message: 'Failed to delete stream key', error: String(error) },
      { status: 500 }
    );
  }
}
