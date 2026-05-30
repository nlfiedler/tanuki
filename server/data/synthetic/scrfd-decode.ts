//
// Copyright (c) 2026 Nathan Fiedler
//

/**
 * One decoded face candidate in detector-input pixel coordinates: an
 * axis-aligned box `[x1, y1, x2, y2]`, the detector confidence, and the five
 * facial landmarks as a flat `[x0, y0, x1, y1, …]` array (left eye, right eye,
 * nose, left mouth corner, right mouth corner).
 */
export interface RawDetection {
  score: number;
  box: [number, number, number, number];
  kps: number[];
}

/** SCRFD-2.5g FPN strides; the model emits one (score, bbox, kps) head per stride. */
const FEAT_STRIDES = [8, 16, 32];
/** Anchors per spatial cell; the two anchors share a center and are interleaved. */
const NUM_ANCHORS = 2;
/** Number of facial landmarks SCRFD regresses. */
const NUM_KPS = 5;

/**
 * Decode SCRFD-2.5g network outputs into face candidates, following the
 * InsightFace reference. The three score / bbox / kps arrays are ordered by
 * ascending stride (8, 16, 32) and laid out cell-major, anchor-minor. Box and
 * landmark predictions are point-to-edge distances in stride units; we recover
 * absolute coordinates from each anchor center.
 *
 * Coordinates are in the detector-input frame (e.g. 640×640); the caller maps
 * them back to original-image pixels by dividing by the letterbox scale.
 *
 * @param scores - per-stride confidence arrays, shape `[N, 1]` flattened.
 * @param bboxes - per-stride distance arrays, shape `[N, 4]` flattened.
 * @param kps - per-stride landmark arrays, shape `[N, 10]` flattened.
 * @param inputWidth - detector input width in pixels.
 * @param inputHeight - detector input height in pixels.
 * @param threshold - discard candidates scoring below this.
 */
export function decodeScrfd(
  scores: Float32Array[],
  bboxes: Float32Array[],
  kps: Float32Array[],
  inputWidth: number,
  inputHeight: number,
  threshold: number
): RawDetection[] {
  const detections: RawDetection[] = [];
  for (const [s, FEAT_STRIDE] of FEAT_STRIDES.entries()) {
    const stride = FEAT_STRIDE!;
    const score = scores[s]!;
    const bbox = bboxes[s]!;
    const kp = kps[s]!;
    const height = Math.round(inputHeight / stride);
    const width = Math.round(inputWidth / stride);
    for (let i = 0; i < height; i++) {
      for (let j = 0; j < width; j++) {
        for (let a = 0; a < NUM_ANCHORS; a++) {
          const idx = (i * width + j) * NUM_ANCHORS + a;
          if (score[idx]! < threshold) continue;
          const cx = j * stride;
          const cy = i * stride;
          const x1 = cx - bbox[idx * 4]! * stride;
          const y1 = cy - bbox[idx * 4 + 1]! * stride;
          const x2 = cx + bbox[idx * 4 + 2]! * stride;
          const y2 = cy + bbox[idx * 4 + 3]! * stride;
          const points: number[] = [];
          for (let p = 0; p < NUM_KPS; p++) {
            points.push(cx + kp[idx * 10 + p * 2]! * stride, cy + kp[idx * 10 + p * 2 + 1]! * stride);
          }
          detections.push({
            score: score[idx]!,
            box: [x1, y1, x2, y2],
            kps: points
          });
        }
      }
    }
  }
  return detections;
}

/** Intersection-over-union of two `[x1, y1, x2, y2]` boxes. */
function iou(
  a: [number, number, number, number],
  b: [number, number, number, number]
): number {
  const ix1 = Math.max(a[0], b[0]);
  const iy1 = Math.max(a[1], b[1]);
  const ix2 = Math.min(a[2], b[2]);
  const iy2 = Math.min(a[3], b[3]);
  const iw = Math.max(0, ix2 - ix1);
  const ih = Math.max(0, iy2 - iy1);
  const inter = iw * ih;
  const areaA = Math.max(0, a[2] - a[0]) * Math.max(0, a[3] - a[1]);
  const areaB = Math.max(0, b[2] - b[0]) * Math.max(0, b[3] - b[1]);
  const union = areaA + areaB - inter;
  return union <= 0 ? 0 : inter / union;
}

/**
 * Greedy non-maximum suppression: keep the highest-scoring boxes, discarding
 * any that overlap an already-kept box by more than `iouThreshold`. Returns the
 * survivors in descending score order.
 *
 * @param detections - candidates to filter (not mutated).
 * @param iouThreshold - overlap above which the lower-scoring box is dropped.
 */
export function nms(
  detections: RawDetection[],
  iouThreshold: number
): RawDetection[] {
  const order = [...detections].sort((x, y) => y.score - x.score);
  const kept: RawDetection[] = [];
  for (const candidate of order) {
    let suppressed = false;
    for (const winner of kept) {
      if (iou(candidate.box, winner.box) > iouThreshold) {
        suppressed = true;
        break;
      }
    }
    if (!suppressed) kept.push(candidate);
  }
  return kept;
}
