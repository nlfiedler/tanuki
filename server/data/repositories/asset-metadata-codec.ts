//
// Copyright (c) 2026 Nathan Fiedler
//
import { AssetMetadata } from 'tanuki/server/domain/entities/asset-metadata.ts';

/**
 * Convert an AssetMetadata entity to a plain object suitable for persistence
 * as JSON (CouchDB/PouchDB sub-document, or the SQLite `raw` column for
 * round-trip). Returns null if the metadata is null or carries no values.
 */
export function metadataToDocument(
  metadata: AssetMetadata | null
): object | null {
  if (metadata === null || !metadata.hasValues()) {
    return null;
  }
  return {
    cameraMake: metadata.cameraMake,
    cameraModel: metadata.cameraModel,
    lensMake: metadata.lensMake,
    lensModel: metadata.lensModel,
    exposureTime: metadata.exposureTime,
    fNumber: metadata.fNumber,
    iso: metadata.iso,
    focalLength35mm: metadata.focalLength35mm,
    originalDateOffset: metadata.originalDateOffset,
    gpsLatitude: metadata.gpsLatitude,
    gpsLongitude: metadata.gpsLongitude,
    displayWidth: metadata.displayWidth,
    displayHeight: metadata.displayHeight,
    duration: metadata.duration,
    frameRate: metadata.frameRate,
    videoCodec: metadata.videoCodec,
    raw: metadata.raw
  };
}

/**
 * Hydrate an AssetMetadata entity from a plain object retrieved from
 * persistence. Returns null if the document is null/undefined.
 */
export function metadataFromDocument(doc: any): AssetMetadata | null {
  if (doc === null || doc === undefined) {
    return null;
  }
  const m = new AssetMetadata();
  m.cameraMake = doc.cameraMake ?? null;
  m.cameraModel = doc.cameraModel ?? null;
  m.lensMake = doc.lensMake ?? null;
  m.lensModel = doc.lensModel ?? null;
  m.exposureTime = doc.exposureTime ?? null;
  m.fNumber = doc.fNumber ?? null;
  m.iso = doc.iso ?? null;
  m.focalLength35mm = doc.focalLength35mm ?? null;
  m.originalDateOffset = doc.originalDateOffset ?? null;
  m.gpsLatitude = doc.gpsLatitude ?? null;
  m.gpsLongitude = doc.gpsLongitude ?? null;
  m.displayWidth = doc.displayWidth ?? null;
  m.displayHeight = doc.displayHeight ?? null;
  m.duration = doc.duration ?? null;
  m.frameRate = doc.frameRate ?? null;
  m.videoCodec = doc.videoCodec ?? null;
  m.raw = doc.raw ?? null;
  return m;
}
