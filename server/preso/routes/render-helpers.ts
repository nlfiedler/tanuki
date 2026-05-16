//
// Copyright (c) 2026 Nathan Fiedler
//
import path from 'node:path';
import sharp from 'sharp';

const VIDEO_EXTENSIONS = new Set([
  '.mp4',
  '.mov',
  '.qt',
  '.m4v',
  '.webm',
  '.mkv',
  '.avi',
  '.wmv',
  '.mpg',
  '.mpeg',
  '.3gp',
  '.ogv'
]);

function isVideoPath(filepath: string): boolean {
  return VIDEO_EXTENSIONS.has(path.extname(filepath).toLowerCase());
}

// Extract a representative frame from a video file using ffmpeg's `thumbnail`
// filter, returning an mjpeg buffer. Mirrors the namazu sidecar's strategy.
async function extractVideoFrame(filepath: string): Promise<Buffer> {
  const proc = Bun.spawn(
    [
      'ffmpeg',
      '-i',
      filepath,
      '-vf',
      'thumbnail',
      '-frames:v',
      '1',
      '-f',
      'image2pipe',
      '-vcodec',
      'mjpeg',
      '-loglevel',
      'error',
      'pipe:1'
    ],
    { stdout: 'pipe', stderr: 'pipe' }
  );
  const data = await new Response(proc.stdout).arrayBuffer();
  const exitCode = await proc.exited;
  if (exitCode !== 0) {
    const stderr = await new Response(proc.stderr).text();
    throw new Error(`ffmpeg failed (${exitCode}): ${stderr}`);
  }
  return Buffer.from(data);
}

async function renderResizedJpeg(
  filepath: string,
  resizeOptions: sharp.ResizeOptions
): Promise<Buffer> {
  const source: Buffer | string = isVideoPath(filepath)
    ? await extractVideoFrame(filepath)
    : filepath;
  return await sharp(source, { autoOrient: true })
    .resize(resizeOptions)
    .toFormat('jpeg')
    .toBuffer();
}

// Cap on rendered dimensions to bound memory/CPU use per request.
const MAX_RENDER_DIMENSION = 4096;

// Parse a positive integer query/path parameter, optionally clamped to a
// maximum. Returns null if the value is missing, non-numeric, non-integer,
// less than 1, or greater than `max`.
function parsePositiveInt(
  raw: string | undefined,
  max = MAX_RENDER_DIMENSION
): number | null {
  if (typeof raw !== 'string' || !/^\d+$/.test(raw)) {
    return null;
  }
  const n = Number.parseInt(raw, 10);
  if (!Number.isInteger(n) || n < 1 || n > max) {
    return null;
  }
  return n;
}

export {
  MAX_RENDER_DIMENSION,
  extractVideoFrame,
  isVideoPath,
  parsePositiveInt,
  renderResizedJpeg
};
