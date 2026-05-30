//
// Copyright (c) 2026 Nathan Fiedler
//
import * as ort from 'onnxruntime-node';
import sharp from 'sharp';
import { type Asset } from 'tanuki/server/domain/entities/asset.ts';
import { DetectedFace } from 'tanuki/server/domain/entities/face.ts';
import { type BlobRepository } from 'tanuki/server/domain/repositories/blob-repository.ts';
import { type SettingsRepository } from 'tanuki/server/domain/repositories/settings-repository.ts';
import { type SyntheticDetector } from 'tanuki/server/domain/services/synthetic-detector.ts';
import { curateLogits } from 'tanuki/server/data/synthetic/label-curation.ts';
import {
  CROP,
  preprocessImage
} from 'tanuki/server/data/synthetic/mobilenet-preprocess.ts';
import {
  decodeScrfd,
  nms,
  type RawDetection
} from 'tanuki/server/data/synthetic/scrfd-decode.ts';
import {
  ARCFACE_TEMPLATE,
  FACE_CROP,
  FACE_MODEL_VERSION,
  arcfacePreprocess,
  clamp,
  estimateSimilarityTransform,
  l2normalize,
  warpAffineBilinear
} from 'tanuki/server/data/synthetic/face-align.ts';

/** Default locations of the bundled ONNX models. */
const DEFAULT_LABEL_MODEL = 'models/mobilenet_v2.onnx';
const DEFAULT_DETECT_MODEL = 'models/scrfd_2.5g.onnx';
const DEFAULT_EMBED_MODEL = 'models/mobilefacenet.onnx';

/** SCRFD detector input is a fixed square; images are letterboxed into it. */
const DETECT_SIZE = 640;
/** InsightFace defaults: discard faces below this score, suppress above this IoU. */
const SCORE_THRESHOLD = 0.5;
const NMS_IOU = 0.4;
/** Cap faces per asset, matching the Namazu inference contract. */
const MAX_FACES = 20;

/**
 * In-process detector running MobileNetV2 (labels), SCRFD-2.5g (face
 * detection), and MobileFaceNet (face embedding) via `onnxruntime-node`. Bytes
 * are pulled from whichever blob store is configured through the shared
 * `fetchRange` contract, so this one implementation serves every deployment;
 * the Namazu push-down optimization can be layered on later.
 *
 * Each ONNX session loads lazily on first use and is cached for the process
 * lifetime; a failed load clears its cache slot so the next call retries.
 */
class LocalSyntheticDetector implements SyntheticDetector {
  private blobRepository: BlobRepository;
  private labelModelPath: string;
  private detectModelPath: string;
  private embedModelPath: string;
  private sessions = new Map<string, Promise<ort.InferenceSession>>();

  constructor({
    blobRepository,
    settingsRepository
  }: {
    blobRepository: BlobRepository;
    settingsRepository: SettingsRepository;
  }) {
    this.blobRepository = blobRepository;
    this.labelModelPath =
      settingsRepository.get('SYNTHETIC_MODEL_PATH') || DEFAULT_LABEL_MODEL;
    this.detectModelPath =
      settingsRepository.get('FACE_DETECT_MODEL_PATH') || DEFAULT_DETECT_MODEL;
    this.embedModelPath =
      settingsRepository.get('FACE_EMBED_MODEL_PATH') || DEFAULT_EMBED_MODEL;
  }

  /** Lazily create (and cache) an ONNX session for the given model path. */
  private session(path: string): Promise<ort.InferenceSession> {
    const existing = this.sessions.get(path);
    if (existing) return existing;
    const p = ort.InferenceSession.create(path);
    // Clear the cache on rejection so a transient load failure (missing/locked
    // model file) doesn't permanently poison every future call.
    p.catch(() => {
      if (this.sessions.get(path) === p) this.sessions.delete(path);
    });
    this.sessions.set(path, p);
    return p;
  }

  /** Read an image asset's bytes, or null if it is not a usable image. */
  private async imageBytes(asset: Asset): Promise<Buffer | null> {
    if (!asset.mediaType.startsWith('image/') || asset.byteLength <= 0) {
      return null;
    }
    const bytes = await this.blobRepository.fetchRange(
      asset.key,
      0,
      asset.byteLength - 1
    );
    return bytes.length === 0 ? null : bytes;
  }

  /** @inheritDoc */
  async detectLabels(asset: Asset): Promise<string[]> {
    const bytes = await this.imageBytes(asset);
    if (bytes === null) return [];

    const input = await preprocessImage(bytes);
    const session = await this.session(this.labelModelPath);
    const tensor = new ort.Tensor('float32', input, [1, 3, CROP, CROP]);
    const results = await session.run({ [session.inputNames[0]!]: tensor });
    const logits = results[session.outputNames[0]!]!.data as Float32Array;
    return curateLogits(logits);
  }

  /** @inheritDoc */
  faceModelVersion(): string {
    return FACE_MODEL_VERSION;
  }

  /** @inheritDoc */
  async detectFaces(asset: Asset): Promise<DetectedFace[]> {
    const bytes = await this.imageBytes(asset);
    if (bytes === null) return [];

    // Decode once in displayed orientation (apply EXIF rotation) so bounding
    // boxes and crops are in the same frame as metadata.displayWidth/Height.
    const base = sharp(bytes, { failOn: 'none' })
      .rotate()
      .removeAlpha()
      .toColourspace('srgb');
    const { data: fullBuf, info } = await base
      .raw()
      .toBuffer({ resolveWithObject: true });
    const full = new Uint8Array(
      fullBuf.buffer,
      fullBuf.byteOffset,
      fullBuf.byteLength
    );
    const width = info.width;
    const height = info.height;
    if (width === 0 || height === 0) return [];

    const detections = await this.runDetection(full, width, height);
    if (detections.length === 0) return [];

    const embedSession = await this.session(this.embedModelPath);
    const faces: DetectedFace[] = [];
    // Highest-confidence faces first, capped per the inference contract.
    detections.sort((a, b) => b.score - a.score);
    for (const det of detections.slice(0, MAX_FACES)) {
      const face = await this.embedFace(
        det,
        full,
        width,
        height,
        embedSession
      );
      faces.push(face);
    }
    return faces;
  }

  /**
   * Letterbox the full-resolution RGB into the SCRFD input square, run
   * detection, and return surviving faces with box + landmarks mapped back to
   * original-image pixel coordinates.
   */
  private async runDetection(
    full: Uint8Array,
    width: number,
    height: number
  ): Promise<RawDetection[]> {
    const scale = Math.min(DETECT_SIZE / width, DETECT_SIZE / height);
    const newW = Math.max(1, Math.round(width * scale));
    const newH = Math.max(1, Math.round(height * scale));
    // Resize proportionally, then pad to a full square at the top-left so the
    // mapping back to original pixels is a single divide by `scale`.
    const det = await sharp(full, {
      raw: { width, height, channels: 3 }
    })
      .resize(newW, newH, { fit: 'fill' })
      .extend({
        top: 0,
        left: 0,
        bottom: DETECT_SIZE - newH,
        right: DETECT_SIZE - newW,
        background: { r: 0, g: 0, b: 0 }
      })
      .raw()
      .toBuffer();

    // SCRFD preprocessing: (pixel - 127.5) / 128, RGB, planar NCHW.
    const plane = DETECT_SIZE * DETECT_SIZE;
    const input = new Float32Array(3 * plane);
    for (let i = 0; i < plane; i++) {
      input[i] = (det[i * 3]! - 127.5) / 128;
      input[plane + i] = (det[i * 3 + 1]! - 127.5) / 128;
      input[2 * plane + i] = (det[i * 3 + 2]! - 127.5) / 128;
    }

    const session = await this.session(this.detectModelPath);
    const tensor = new ort.Tensor('float32', input, [
      1,
      3,
      DETECT_SIZE,
      DETECT_SIZE
    ]);
    const results = await session.run({ [session.inputNames[0]!]: tensor });

    // The output names are opaque; classify each head by its last dimension
    // (1 = score, 4 = bbox, 10 = kps) and order each group by anchor count so
    // strides line up as [8, 16, 32] regardless of graph naming.
    const scores: { n: number; data: Float32Array }[] = [];
    const bboxes: { n: number; data: Float32Array }[] = [];
    const kpss: { n: number; data: Float32Array }[] = [];
    for (const name of session.outputNames) {
      const out = results[name]!;
      const data = out.data as Float32Array;
      const last = out.dims.at(-1)!;
      const n = out.dims[0]!;
      switch (last) {
        case 1: {
          scores.push({ n, data });
          break;
        }
        case 4: {
          bboxes.push({ n, data });
          break;
        }
        case 10: {
          kpss.push({ n, data });
          break;
        }
        // No default: any other shape is not part of the SCRFD head set.
      }
    }
    scores.sort(byAnchorsDesc);
    bboxes.sort(byAnchorsDesc);
    kpss.sort(byAnchorsDesc);

    const decoded = decodeScrfd(
      scores.map((x) => x.data),
      bboxes.map((x) => x.data),
      kpss.map((x) => x.data),
      DETECT_SIZE,
      DETECT_SIZE,
      SCORE_THRESHOLD
    );
    const kept = nms(decoded, NMS_IOU);

    // Map detector-frame coordinates back to original pixels and clamp boxes.
    for (const d of kept) {
      d.box = [
        clamp(d.box[0] / scale, 0, width),
        clamp(d.box[1] / scale, 0, height),
        clamp(d.box[2] / scale, 0, width),
        clamp(d.box[3] / scale, 0, height)
      ];
      d.kps = d.kps.map((v) => v / scale);
    }
    return kept;
  }

  /** Align, embed, and crop a single detected face into a {@link DetectedFace}. */
  private async embedFace(
    det: RawDetection,
    full: Uint8Array,
    width: number,
    height: number,
    embedSession: ort.InferenceSession
  ): Promise<DetectedFace> {
    // Warp the landmarks onto the ArcFace template to get an aligned crop.
    const transform = estimateSimilarityTransform(det.kps, ARCFACE_TEMPLATE);
    const crop = warpAffineBilinear(
      full,
      width,
      height,
      transform,
      FACE_CROP,
      FACE_CROP
    );

    const input = arcfacePreprocess(crop);
    const tensor = new ort.Tensor('float32', input, [1, 3, FACE_CROP, FACE_CROP]);
    const results = await embedSession.run({
      [embedSession.inputNames[0]!]: tensor
    });
    const raw = results[embedSession.outputNames[0]!]!.data as Float32Array;
    const embedding = l2normalize(Float32Array.from(raw));

    // Encode the aligned crop as the stored ~112px JPEG thumbnail.
    const thumbnail = await sharp(crop, {
      raw: { width: FACE_CROP, height: FACE_CROP, channels: 3 }
    })
      .jpeg({ quality: 85 })
      .toBuffer();

    const [x1, y1, x2, y2] = det.box;
    return new DetectedFace(
      [x1, y1, x2 - x1, y2 - y1],
      embedding,
      new Uint8Array(thumbnail),
      det.score,
      FACE_MODEL_VERSION
    );
  }
}

/** Order detector heads by descending anchor count, i.e. stride [8, 16, 32]. */
function byAnchorsDesc(a: { n: number }, b: { n: number }): number {
  return b.n - a.n;
}

export { LocalSyntheticDetector };
