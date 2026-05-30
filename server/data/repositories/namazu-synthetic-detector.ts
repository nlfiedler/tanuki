//
// Copyright (c) 2026 Nathan Fiedler
//
import assert from 'node:assert';
import { type Asset } from 'tanuki/server/domain/entities/asset.ts';
import { DetectedFace } from 'tanuki/server/domain/entities/face.ts';
import { type SettingsRepository } from 'tanuki/server/domain/repositories/settings-repository.ts';
import { type SyntheticDetector } from 'tanuki/server/domain/services/synthetic-detector.ts';
import { MAX_LABELS } from 'tanuki/server/data/synthetic/label-curation.ts';
import {
  FACE_MODEL_VERSION,
  l2normalize
} from 'tanuki/server/data/synthetic/face-align.ts';

/** One label entry in a Namazu `/synthetic` response (already curated). */
interface NamazuLabel {
  name: string;
  score: number;
}

/** One face entry in a Namazu `/synthetic` response. */
interface NamazuFace {
  bbox: [number, number, number, number];
  embedding: string; // base64 little-endian Float32, 512 floats
  thumbnail: string; // base64 JPEG, ~128px
  score: number;
  model_version: string;
}

interface NamazuSyntheticResponse {
  labels?: NamazuLabel[];
  faces?: NamazuFace[];
  model_versions?: { labels?: string; faces?: string };
  truncated?: boolean;
}

/**
 * Synthetic-data detector that pushes inference to a Namazu blob store, which
 * has byte-level access to the asset and runs the same ONNX models (MobileNetV2
 * for labels, SCRFD-2.5g + MobileFaceNet for faces) as the local detector.
 * One HTTP round-trip per call against `POST /synthetic/<assetId>`; the
 * response carries both labels (already curated) and faces, and we extract the
 * slice the caller asked for.
 *
 * Selected by the container when `NAMAZU_URL` is set, in place of
 * {@link import('./local-synthetic-detector.ts').LocalSyntheticDetector}; no
 * local ONNX runtime or model files are exercised in that configuration.
 */
class NamazuSyntheticDetector implements SyntheticDetector {
  private baseUrl: string;
  private modelVersion: string;

  constructor({
    settingsRepository
  }: {
    settingsRepository: SettingsRepository;
  }) {
    this.baseUrl = settingsRepository.get('NAMAZU_URL').replace(/\/+$/, '');
    assert.ok(this.baseUrl, 'missing NAMAZU_URL environment variable');
    // Defaults to the shared identifier; overridable via `FACE_MODEL_VERSION`
    // if a Namazu deployment advances its model ahead of this build.
    this.modelVersion =
      settingsRepository.get('FACE_MODEL_VERSION') || FACE_MODEL_VERSION;
  }

  /** @inheritDoc */
  async detectLabels(asset: Asset): Promise<string[]> {
    const data = await this.fetchSynthetic(asset);
    if (data === null) return [];
    const labels = data.labels ?? [];
    // The names are already curated; de-duplicate keeping the highest score,
    // sort by score descending, and cap to match the local detector's output.
    const best = new Map<string, number>();
    for (const { name, score } of labels) {
      if (!best.has(name) || score > best.get(name)!) best.set(name, score);
    }
    return [...best.entries()]
      .sort((a, b) => b[1] - a[1])
      .slice(0, MAX_LABELS)
      .map(([name]) => name);
  }

  /** @inheritDoc */
  async detectFaces(asset: Asset): Promise<DetectedFace[]> {
    const data = await this.fetchSynthetic(asset);
    if (data === null) return [];
    return (data.faces ?? []).map(
      (face) =>
        new DetectedFace(
          face.bbox,
          // re-normalize defensively; cosine clustering assumes unit vectors
          l2normalize(decodeEmbedding(face.embedding)),
          decodeBase64(face.thumbnail),
          face.score,
          face.model_version || this.modelVersion
        )
    );
  }

  /** @inheritDoc */
  faceModelVersion(): string {
    return this.modelVersion;
  }

  /**
   * POST the asset to Namazu's `/synthetic` endpoint and return the parsed
   * response, or null for a non-image (204) or non-image-typed asset. Throws on
   * any other non-2xx so the worker pool retries and ultimately records FAILED.
   */
  private async fetchSynthetic(
    asset: Asset
  ): Promise<NamazuSyntheticResponse | null> {
    if (!asset.mediaType.startsWith('image/') || asset.byteLength <= 0) {
      return null;
    }
    const url = `${this.baseUrl}/synthetic/${asset.key}`;
    const response = await fetch(url, { method: 'POST' });
    if (response.status === 204) return null;
    if (!response.ok) {
      const body = await response.text().catch(() => '');
      throw new Error(
        `namazu /synthetic returned ${response.status}: ${body.slice(0, 200)}`
      );
    }
    return (await response.json()) as NamazuSyntheticResponse;
  }
}

/** Decode a base64 little-endian Float32 buffer into a Float32Array. */
function decodeEmbedding(base64: string): Float32Array {
  const buf = Buffer.from(base64, 'base64');
  const view = new DataView(buf.buffer, buf.byteOffset, buf.byteLength);
  const floats = new Float32Array(Math.floor(buf.byteLength / 4));
  for (let i = 0; i < floats.length; i++) {
    floats[i] = view.getFloat32(i * 4, true);
  }
  return floats;
}

/** Decode base64 into a standalone Uint8Array (e.g. a JPEG thumbnail). */
function decodeBase64(base64: string): Uint8Array {
  const buf = Buffer.from(base64, 'base64');
  return new Uint8Array(buf.buffer, buf.byteOffset, buf.byteLength);
}

export { NamazuSyntheticDetector };
