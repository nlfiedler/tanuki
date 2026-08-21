//
// Copyright (c) 2026 Nathan Fiedler
//
import {
  SyntheticData,
  SyntheticStatus
} from 'tanuki/server/domain/entities/synthetic-data.ts';

/**
 * Plain object representation of synthetic data persisted on the asset
 * document (CouchDB / PouchDB sub-document). The `status` field is included
 * so a single sub-document carries both the labels and their extraction state.
 */
export interface SyntheticDocument {
  labels: string[];
  primaryLabel: string | null;
  status: SyntheticStatus;
}

/**
 * Convert a SyntheticData entity plus status into a plain object suitable for
 * persistence. Returns null when the entity carries no labels AND the status
 * is the default PENDING; in that case the persistence layer can omit the
 * sub-document entirely (absence implies PENDING).
 */
export function syntheticToDocument(
  data: SyntheticData | null,
  status: SyntheticStatus
): SyntheticDocument | null {
  const hasData = data !== null && data.hasValues();
  if (!hasData && status === SyntheticStatus.PENDING) {
    return null;
  }
  return {
    labels: data?.labels ?? [],
    primaryLabel: data?.primaryLabel ?? null,
    status
  };
}

/**
 * Hydrate a SyntheticData entity (and its status) from a persisted plain
 * object. Returns `{ data: null, status: PENDING }` when the document is
 * absent.
 */
export function syntheticFromDocument(doc: any): {
  data: SyntheticData | null;
  status: SyntheticStatus;
} {
  if (doc === null || doc === undefined) {
    return { data: null, status: SyntheticStatus.PENDING };
  }
  const status = parseStatus(doc.status);
  const labels = Array.isArray(doc.labels) ? (doc.labels as string[]) : [];
  // Coalesce empty strings as well as null/undefined: an empty `primaryLabel`
  // is never meaningful and would otherwise survive `??` and look like a real
  // entry to downstream code.
  const primaryLabel = doc.primaryLabel || null;
  if (primaryLabel === null && labels.length === 0) {
    return { data: null, status };
  }
  const data = new SyntheticData();
  data.labels = labels;
  data.primaryLabel = primaryLabel;
  return { data, status };
}

/**
Parse a persisted status value, defaulting to PENDING on unrecognized input.
*/
export function parseStatus(value: unknown): SyntheticStatus {
  if ([SyntheticStatus.READY, 'READY', 'ready'].includes(value as any)) {
    return SyntheticStatus.READY;
  }
  if ([SyntheticStatus.FAILED, 'FAILED', 'failed'].includes(value as any)) {
    return SyntheticStatus.FAILED;
  }
  return SyntheticStatus.PENDING;
}
