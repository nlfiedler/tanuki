//
// Copyright (c) 2026 Nathan Fiedler
//
import * as ort from 'onnxruntime-node';
import { type Asset } from 'tanuki/server/domain/entities/asset.ts';
import { type BlobRepository } from 'tanuki/server/domain/repositories/blob-repository.ts';
import { type SettingsRepository } from 'tanuki/server/domain/repositories/settings-repository.ts';
import { type SyntheticDetector } from 'tanuki/server/domain/services/synthetic-detector.ts';
import { curateLogits } from 'tanuki/server/data/synthetic/label-curation.ts';
import {
  CROP,
  preprocessImage
} from 'tanuki/server/data/synthetic/mobilenet-preprocess.ts';

/** Default location of the bundled MobileNetV2 ONNX model. */
const DEFAULT_MODEL_PATH = 'models/mobilenet_v2.onnx';

/**
 * In-process label detector running MobileNetV2 via `onnxruntime-node`. Bytes
 * are pulled from whichever blob store is configured (local or Namazu) through
 * the shared `fetchRange` contract, so this one implementation serves every
 * deployment; the Namazu push-down optimization can be layered on later.
 *
 * The ONNX session loads lazily on first use and is cached for the process
 * lifetime.
 */
class LocalSyntheticDetector implements SyntheticDetector {
  private blobRepository: BlobRepository;
  private modelPath: string;
  private sessionPromise: Promise<ort.InferenceSession> | null = null;

  constructor({
    blobRepository,
    settingsRepository
  }: {
    blobRepository: BlobRepository;
    settingsRepository: SettingsRepository;
  }) {
    this.blobRepository = blobRepository;
    this.modelPath =
      settingsRepository.get('SYNTHETIC_MODEL_PATH') || DEFAULT_MODEL_PATH;
  }

  private session(): Promise<ort.InferenceSession> {
    if (this.sessionPromise === null) {
      const p = ort.InferenceSession.create(this.modelPath);
      // Clear the cache on rejection so the next request retries the load.
      // Without this, a single transient failure (missing/locked model file)
      // would permanently poison every future detectLabels() call.
      p.catch(() => {
        if (this.sessionPromise === p) this.sessionPromise = null;
      });
      this.sessionPromise = p;
    }
    return this.sessionPromise;
  }

  /** @inheritDoc */
  async detectLabels(asset: Asset): Promise<string[]> {
    if (!asset.mediaType.startsWith('image/') || asset.byteLength <= 0) {
      return [];
    }
    const bytes = await this.blobRepository.fetchRange(
      asset.key,
      0,
      asset.byteLength - 1
    );
    if (bytes.length === 0) return [];

    const input = await preprocessImage(bytes);
    const session = await this.session();
    const tensor = new ort.Tensor('float32', input, [1, 3, CROP, CROP]);
    const results = await session.run({ [session.inputNames[0]!]: tensor });
    const logits = results[session.outputNames[0]!]!.data as Float32Array;
    return curateLogits(logits);
  }
}

export { LocalSyntheticDetector };
