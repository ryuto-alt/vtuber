import NodeMediaServer from 'node-media-server';

let nms: NodeMediaServer | null = null;
const validStreamKeys = new Set<string>();

export interface RTMPConfig {
  rtmpPort: number;
  httpPort: number;
}

export function initRTMPServer(config: RTMPConfig) {
  if (nms) {
    return nms;
  }

  const serverConfig = {
    rtmp: {
      port: config.rtmpPort,
      chunk_size: 60000,
      gop_cache: true,
      ping: 30,
      ping_timeout: 60
    },
    http: {
      port: config.httpPort,
      allow_origin: '*',
      mediaroot: './media'
    },
    trans: {
      ffmpeg: process.env.FFMPEG_PATH || 'ffmpeg',
      tasks: [
        {
          app: 'live',
          hls: true,
          hlsFlags: '[hls_time=2:hls_list_size=3:hls_flags=delete_segments]',
          dash: true,
          dashFlags: '[f=dash:window_size=3:extra_window_size=5]'
        }
      ]
    }
  };

  nms = new NodeMediaServer(serverConfig);

  // Authentication
  nms.on('prePublish', (id: string, StreamPath: string, args: any) => {
    const streamKey = StreamPath.split('/').pop();
    if (!streamKey || !validStreamKeys.has(streamKey)) {
      const session = nms!.getSession(id);
      session.reject();
      console.log('[RTMP] Invalid stream key rejected:', streamKey);
    } else {
      console.log('[RTMP] Stream started:', streamKey);
    }
  });

  nms.on('donePublish', (id: string, StreamPath: string, args: any) => {
    console.log('[RTMP] Stream ended:', StreamPath);
  });

  return nms;
}

export function startRTMPServer(config: RTMPConfig) {
  const server = initRTMPServer(config);
  if (server && !isServerRunning()) {
    server.run();
    console.log(`[RTMP] Server started on port ${config.rtmpPort}`);
    console.log(`[HTTP-FLV] Server started on port ${config.httpPort}`);
  }
  return server;
}

export function addStreamKey(key: string) {
  validStreamKeys.add(key);
  console.log('[RTMP] Stream key added:', key);
}

export function removeStreamKey(key: string) {
  validStreamKeys.delete(key);
  console.log('[RTMP] Stream key removed:', key);
}

export function isValidStreamKey(key: string): boolean {
  return validStreamKeys.has(key);
}

export function isServerRunning(): boolean {
  return nms !== null;
}

export function getServerInstance(): NodeMediaServer | null {
  return nms;
}
