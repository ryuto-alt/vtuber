import { NextResponse } from 'next/server';
import { startRTMPServer, isServerRunning } from '@/lib/rtmp-server';

export async function POST() {
  try {
    if (isServerRunning()) {
      return NextResponse.json(
        { message: 'RTMP server is already running', status: 'running' },
        { status: 200 }
      );
    }

    const config = {
      rtmpPort: parseInt(process.env.RTMP_PORT || '1935'),
      httpPort: parseInt(process.env.HTTP_FLV_PORT || '8000')
    };

    startRTMPServer(config);

    return NextResponse.json(
      {
        message: 'RTMP server started successfully',
        status: 'running',
        rtmpPort: config.rtmpPort,
        httpPort: config.httpPort
      },
      { status: 200 }
    );
  } catch (error) {
    console.error('[API] Failed to start RTMP server:', error);
    return NextResponse.json(
      { message: 'Failed to start RTMP server', error: String(error) },
      { status: 500 }
    );
  }
}

export async function GET() {
  try {
    const running = isServerRunning();
    return NextResponse.json({
      status: running ? 'running' : 'stopped',
      rtmpPort: parseInt(process.env.RTMP_PORT || '1935'),
      httpPort: parseInt(process.env.HTTP_FLV_PORT || '8000')
    });
  } catch (error) {
    return NextResponse.json(
      { message: 'Failed to get server status', error: String(error) },
      { status: 500 }
    );
  }
}
