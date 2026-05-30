//
// Copyright (c) 2026 Nathan Fiedler
//
import crypto from 'node:crypto';
import { type Asset } from 'tanuki/server/domain/entities/asset.ts';
import { type DetectedFace, Face, type SyntheticJob } from 'tanuki/server/domain/entities/face.ts';
import {
  SyntheticData,
  SyntheticStatus
} from 'tanuki/server/domain/entities/synthetic-data.ts';
import { type FaceStore } from 'tanuki/server/domain/repositories/face-store.ts';
import { type RecordRepository } from 'tanuki/server/domain/repositories/record-repository.ts';
import { type SettingsRepository } from 'tanuki/server/domain/repositories/settings-repository.ts';
import { type SyntheticDetector } from 'tanuki/server/domain/services/synthetic-detector.ts';
import { type SyntheticJobProcessor } from 'tanuki/server/domain/services/synthetic-job-processor.ts';
import logger from 'tanuki/server/logger.ts';

/**
 * Cosine-similarity threshold for online clustering: a detected face joins the
 * nearest existing person when their similarity is at least this, otherwise it
 * seeds a new person. 0.5 is well-characterized for MobileFaceNet on aligned
 * crops; overridable via `FACE_CLUSTER_THRESHOLD`.
 */
const DEFAULT_CLUSTER_THRESHOLD = 0.5;

/**
 * Worker-pool processor that runs the {@link SyntheticDetector} against a job's
 * asset and persists the result. Labels jobs classify the image and write the
 * labels (and READY status) to the asset record. Faces jobs detect + embed
 * faces, cluster each into a person, store the face rows in the face store, and
 * record the faces status there. Recoverable failures throw so the pool can
 * retry and ultimately record FAILED (kind-aware) for the asset.
 */
class DetectingSyntheticJobProcessor implements SyntheticJobProcessor {
  private recordRepository: RecordRepository;
  private faceStore: FaceStore;
  private detector: SyntheticDetector;
  private clusterThreshold: number;

  constructor({
    recordRepository,
    faceStore,
    syntheticDetector,
    settingsRepository
  }: {
    recordRepository: RecordRepository;
    faceStore: FaceStore;
    syntheticDetector: SyntheticDetector;
    settingsRepository: SettingsRepository;
  }) {
    this.recordRepository = recordRepository;
    this.faceStore = faceStore;
    this.detector = syntheticDetector;
    this.clusterThreshold = settingsRepository.getFloat(
      'FACE_CLUSTER_THRESHOLD',
      DEFAULT_CLUSTER_THRESHOLD
    );
  }

  /** @inheritDoc */
  async process(job: SyntheticJob): Promise<void> {
    const asset = await this.recordRepository.getAssetById(job.assetId);
    if (asset === null) {
      // asset was deleted before we got to it; nothing to do (don't retry)
      return;
    }
    if (job.kind === 'labels') {
      await this.processLabels(job.assetId, asset);
    } else if (job.kind === 'faces') {
      await this.processFaces(job.assetId, asset);
    } else {
      throw new Error(`unsupported synthetic job kind: ${job.kind}`);
    }
  }

  private async processLabels(assetId: string, asset: Asset): Promise<void> {
    const labels = await this.detector.detectLabels(asset);
    const data = new SyntheticData();
    data.labels = labels;
    data.primaryLabel = labels[0] ?? null;
    // Store null when nothing cleared the score floor, but still mark READY:
    // the asset was processed, it simply produced no labels.
    await this.recordRepository.setSynthetic(
      assetId,
      data.hasValues() ? data : null,
      SyntheticStatus.READY
    );
  }

  private async processFaces(assetId: string, asset: Asset): Promise<void> {
    const detected = await this.detector.detectFaces(asset);
    // Reprocessing an asset: clear any prior faces so a re-run (e.g. a model
    // upgrade backfill) replaces rather than duplicates. This also removes
    // now-empty person rows.
    await this.faceStore.deleteByAssetId(assetId);
    let persisted = 0;
    for (const face of detected) {
      // Persist faces one at a time so a single bad embedding doesn't lose the
      // others (partial-record persistence per the spec). Cluster online: each
      // stored face is visible to the next one's nearest-neighbor lookup, so
      // two faces of a new person in the same image land in one cluster.
      try {
        await this.persistFace(assetId, face);
        persisted++;
      } catch (error: any) {
        logger.warn(
          `faces job: failed to persist a face for asset ${assetId}:`,
          error
        );
      }
    }
    // If faces were detected but none could be stored, the whole batch failed
    // (e.g. a transient face-store error). Throw so the pool retries instead of
    // recording READY with zero faces — which, combined with the
    // deleteByAssetId above, would also have wiped any prior faces and never be
    // revisited by retry/backfill.
    if (detected.length > 0 && persisted === 0) {
      throw new Error(
        `failed to persist any of ${detected.length} detected faces for asset ${assetId}`
      );
    }
    await this.faceStore.setFacesStatus(assetId, SyntheticStatus.READY);
  }

  /** Cluster one detected face into a person and store it. */
  private async persistFace(
    assetId: string,
    detected: DetectedFace
  ): Promise<void> {
    const nearest = await this.faceStore.nearestPerson(
      detected.embedding,
      detected.modelVersion
    );
    let personId: string;
    if (nearest && nearest.score >= this.clusterThreshold) {
      personId = nearest.personId;
    } else {
      const person = await this.faceStore.createPerson();
      personId = person.id;
    }
    const face = new Face(
      crypto.randomUUID(),
      assetId,
      detected.bbox,
      detected.embedding,
      detected.thumbnail,
      detected.modelVersion,
      personId,
      detected.score
    );
    await this.faceStore.insertFace(face);
  }
}

export { DetectingSyntheticJobProcessor };
