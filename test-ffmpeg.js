const { spawn } = require('child_process');
require('dotenv').config({ path: '.env.local' });

const ffmpegPath = process.env.FFMPEG_PATH || 'ffmpeg';
console.log('Testing FFMPEG Path:', ffmpegPath);

const ffmpeg = spawn(ffmpegPath, ['-version']);

ffmpeg.stdout.on('data', (data) => {
    console.log(`stdout: ${data}`);
});

ffmpeg.stderr.on('data', (data) => {
    console.error(`stderr: ${data}`);
});

ffmpeg.on('close', (code) => {
    console.log(`child process exited with code ${code}`);
});

ffmpeg.on('error', (err) => {
    console.error('Failed to start ffmpeg:', err);
});
