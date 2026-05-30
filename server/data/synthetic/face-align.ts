//
// Copyright (c) 2026 Nathan Fiedler
//

/**
 * Canonical ArcFace 5-point reference landmarks for a 112×112 aligned crop
 * (left eye, right eye, nose, left mouth corner, right mouth corner). Detected
 * landmarks are warped onto this template so every embedding sees a face in the
 * same pose, which is what makes embeddings comparable across images.
 */
export const ARCFACE_TEMPLATE: number[] = [
  38.2946, 51.6963, 73.5318, 51.5014, 56.0252, 71.7366, 41.5493, 92.3655,
  70.7299, 92.2041
];

/** Side length of the aligned face crop (the ArcFace input size). */
export const FACE_CROP = 112;

/**
 * The face-embedding model version tagged onto every face row. Both the local
 * and Namazu detectors ship the same MobileFaceNet model and MUST report the
 * identical identifier, since the matcher only compares embeddings within one
 * version and backfill's staleness check keys on it — so it lives here, shared,
 * rather than duplicated per detector.
 */
export const FACE_MODEL_VERSION = 'mobilefacenet-v1';

/** A 2×3 affine transform `[[a, b, c], [d, e, f]]`. */
export type Affine = [[number, number, number], [number, number, number]];

/**
 * Least-squares 2D similarity transform (uniform scale + rotation +
 * translation, no shear) mapping `src` points onto `dst` points — the
 * closed-form Procrustes solution that `cv2.estimateAffinePartial2D` and
 * skimage's `SimilarityTransform` compute. Points are flat `[x0, y0, x1, y1,
 * …]` arrays of equal length.
 *
 * @param src - source landmarks (detected, in image pixels).
 * @param dst - destination landmarks (the reference template).
 * @returns the 2×3 affine taking src into dst.
 */
export function estimateSimilarityTransform(
  src: number[],
  dst: number[]
): Affine {
  const n = src.length / 2;
  let meanSx = 0;
  let meanSy = 0;
  let meanDx = 0;
  let meanDy = 0;
  for (let i = 0; i < n; i++) {
    meanSx += src[i * 2]!;
    meanSy += src[i * 2 + 1]!;
    meanDx += dst[i * 2]!;
    meanDy += dst[i * 2 + 1]!;
  }
  meanSx /= n;
  meanSy /= n;
  meanDx /= n;
  meanDy /= n;

  // Accumulate over demeaned coordinates so the translation drops out and only
  // the scaled-rotation part (p, q) remains to be solved.
  let sxx = 0; // sum of |src_demean|^2
  let num1 = 0; // sum(Dx*sx + Dy*sy)
  let num2 = 0; // sum(Dy*sx - Dx*sy)
  for (let i = 0; i < n; i++) {
    const sx = src[i * 2]! - meanSx;
    const sy = src[i * 2 + 1]! - meanSy;
    const dx = dst[i * 2]! - meanDx;
    const dy = dst[i * 2 + 1]! - meanDy;
    sxx += sx * sx + sy * sy;
    num1 += dx * sx + dy * sy;
    num2 += dy * sx - dx * sy;
  }
  const safe = sxx === 0 ? 1 : sxx;
  const p = num1 / safe;
  const q = num2 / safe;

  // Linear part [[p, -q], [q, p]] (scaled rotation); translation aligns means.
  const tx = meanDx - (p * meanSx - q * meanSy);
  const ty = meanDy - (q * meanSx + p * meanSy);
  return [
    [p, -q, tx],
    [q, p, ty]
  ];
}

/** Invert a 2×3 affine transform. Throws if the linear part is singular. */
export function invertAffine(m: Affine): Affine {
  const [[a, b, c], [d, e, f]] = m;
  const det = a * e - b * d;
  if (Math.abs(det) < 1e-12) {
    throw new Error('cannot invert a degenerate affine transform');
  }
  const ia = e / det;
  const ib = -b / det;
  const id = -d / det;
  const ie = a / det;
  return [
    [ia, ib, -(ia * c + ib * f)],
    [id, ie, -(id * c + ie * f)]
  ];
}

/**
 * Warp an interleaved-RGB source image into an `outW × outH` crop using the
 * forward affine `m` (source → destination). Bilinear sampling; out-of-bounds
 * samples clamp to the nearest edge pixel.
 *
 * @param src - interleaved RGB bytes, length `srcW * srcH * 3`.
 * @param srcW - source width in pixels.
 * @param srcH - source height in pixels.
 * @param m - forward (src → dst) 2×3 affine; inverted internally for sampling.
 * @param outW - output crop width.
 * @param outH - output crop height.
 * @returns interleaved RGB bytes, length `outW * outH * 3`.
 */
export function warpAffineBilinear(
  src: Uint8Array,
  srcW: number,
  srcH: number,
  m: Affine,
  outW: number,
  outH: number
): Uint8Array {
  const inv = invertAffine(m);
  const out = new Uint8Array(outW * outH * 3);
  for (let oy = 0; oy < outH; oy++) {
    for (let ox = 0; ox < outW; ox++) {
      // Map this destination pixel back to a source location.
      const sx = inv[0][0] * ox + inv[0][1] * oy + inv[0][2];
      const sy = inv[1][0] * ox + inv[1][1] * oy + inv[1][2];
      const x0 = Math.floor(sx);
      const y0 = Math.floor(sy);
      const fx = sx - x0;
      const fy = sy - y0;
      const cx0 = clamp(x0, 0, srcW - 1);
      const cx1 = clamp(x0 + 1, 0, srcW - 1);
      const cy0 = clamp(y0, 0, srcH - 1);
      const cy1 = clamp(y0 + 1, 0, srcH - 1);
      const o = (oy * outW + ox) * 3;
      for (let ch = 0; ch < 3; ch++) {
        const p00 = src[(cy0 * srcW + cx0) * 3 + ch]!;
        const p01 = src[(cy0 * srcW + cx1) * 3 + ch]!;
        const p10 = src[(cy1 * srcW + cx0) * 3 + ch]!;
        const p11 = src[(cy1 * srcW + cx1) * 3 + ch]!;
        const top = p00 + (p01 - p00) * fx;
        const bottom = p10 + (p11 - p10) * fx;
        out[o + ch] = Math.round(top + (bottom - top) * fy);
      }
    }
  }
  return out;
}

function clamp(v: number, lo: number, hi: number): number {
  return v < lo ? lo : (Math.min(v, hi));
}

/**
 * Convert a 112×112 interleaved-RGB aligned crop into the MobileFaceNet input
 * tensor: per-pixel `(value - 127.5) / 128`, planar NCHW, shape `[1, 3, 112,
 * 112]`.
 *
 * @param rgb - interleaved RGB bytes, length `112 * 112 * 3`.
 * @returns Float32Array of length `3 * 112 * 112`.
 */
export function arcfacePreprocess(rgb: Uint8Array): Float32Array {
  const plane = FACE_CROP * FACE_CROP;
  const tensor = new Float32Array(3 * plane);
  for (let i = 0; i < plane; i++) {
    tensor[i] = (rgb[i * 3]! - 127.5) / 128;
    tensor[plane + i] = (rgb[i * 3 + 1]! - 127.5) / 128;
    tensor[2 * plane + i] = (rgb[i * 3 + 2]! - 127.5) / 128;
  }
  return tensor;
}

/**
 * L2-normalize an embedding in place and return it, so cosine similarity
 * reduces to a dot product. A zero vector is left unchanged.
 */
export function l2normalize(vec: Float32Array): Float32Array {
  let norm = 0;
  for (const v of vec) norm += v * v;
  norm = Math.sqrt(norm);
  if (norm > 0) {
    for (let i = 0; i < vec.length; i++) vec[i]! /= norm;
  }
  return vec;
}
