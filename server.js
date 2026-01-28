require('dotenv').config({ path: '.env.local' });
const NodeMediaServer = require('node-media-server');

const validStreamKeys = new Set();
const path = require('path');
const ffmpegPath = process.env.FFMPEG_PATH || 'ffmpeg';
console.log('[DEBUG] FFMPEG Path:', ffmpegPath);

// Add FFMPEG directory to PATH to ensure plugins/libs are found if needed
if (process.env.FFMPEG_PATH) {
  const ffmpegDir = path.dirname(process.env.FFMPEG_PATH);
  process.env.PATH = `${ffmpegDir}${path.delimiter}${process.env.PATH}`;
  console.log('[DEBUG] Added ffmpeg dir to PATH:', ffmpegDir);
}

const fs = require('fs');

const mediaRoot = path.join(__dirname, 'media');

// Ensure media directory exists
if (!fs.existsSync(mediaRoot)) {
  fs.mkdirSync(mediaRoot, { recursive: true });
  console.log('[DEBUG] Created media directory:', mediaRoot);
}

const config = {
  logType: 4,
  rtmp: {
    port: parseInt(process.env.RTMP_PORT) || 1935,
    chunk_size: 60000,
    gop_cache: true,
    ping: 30,
    ping_timeout: 60
  },
  http: {
    port: parseInt(process.env.HTTP_FLV_PORT) || 8000,
    allow_origin: '*',
    mediaroot: mediaRoot
  }
};

const nms = new NodeMediaServer(config);

nms.on('prePublish', (id, StreamPath, args) => {
  console.log('[DEBUG] prePublish:', id, StreamPath);
});

nms.on('postPublish', (id, StreamPath, args) => {
  console.log('[DEBUG] postPublish - Transcoder should start here:', id, StreamPath);
});

nms.run();

console.log('='.repeat(50));
console.log('RTMP Server Started');
console.log('='.repeat(50));
console.log(`RTMP Port: ${config.rtmp.port}`);
console.log(`HTTP-FLV/HLS Port: ${config.http.port}`);
console.log('='.repeat(50));
console.log('OBS Settings:');
console.log(`  Server: rtmp://localhost:${config.rtmp.port}/live`);
console.log(`  Stream Key: (any key for now)`);
console.log('='.repeat(50));
console.log('Stream URL will be:');
console.log(`  HLS: http://localhost:${config.http.port}/live/{stream_key}/index.m3u8`);
console.log(`  FLV: http://localhost:${config.http.port}/live/{stream_key}.flv`);
console.log('='.repeat(50));

// Handle graceful shutdown
process.on('SIGINT', () => {
  console.log('\nShutting down RTMP server...');
  process.exit(0);
});

process.on('SIGTERM', () => {
  console.log('\nShutting down RTMP server...');
  process.exit(0);
});
