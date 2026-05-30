//
// Copyright (c) 2026 Nathan Fiedler
//
import assert from 'node:assert';
import PouchDB from 'pouchdb';
import { Asset } from 'tanuki/server/domain/entities/asset.ts';
import { AssetMetadata } from 'tanuki/server/domain/entities/asset-metadata.ts';
import { AttributeCount } from 'tanuki/server/domain/entities/attributes.ts';
import { Location } from 'tanuki/server/domain/entities/location.ts';
import { SearchResult } from 'tanuki/server/domain/entities/search.ts';
import {
  SyntheticData,
  SyntheticStatus
} from 'tanuki/server/domain/entities/synthetic-data.ts';
import { type RecordRepository } from 'tanuki/server/domain/repositories/record-repository.ts';
import { type SettingsRepository } from 'tanuki/server/domain/repositories/settings-repository.ts';
import {
  syntheticFromDocument,
  syntheticToDocument
} from './synthetic-data-codec.ts';
import {
  metadataFromDocument,
  metadataToDocument
} from './asset-metadata-codec.ts';
import * as views from './couchdb-views.js';

/**
 * Repository for entity records stored in a PouchDB database.
 */
class PouchDBRecordRepository implements RecordRepository {
  dbpath: string;
  database: any;

  constructor({
    settingsRepository
  }: {
    settingsRepository: SettingsRepository;
  }) {
    const basepath = settingsRepository.get('POUCHDB_PATH');
    assert.ok(basepath, 'missing POUCHDB_PATH environment variable');
    this.dbpath = basepath;
    this.database = new PouchDB(basepath);
  }

  /**
   * Destroy and create the database from scratch.
   *
   * @throws if `NODE_ENV` is set to 'production'.
   */
  async destroyAndCreate(): Promise<void> {
    assert.notStrictEqual(
      process.env['NODE_ENV'],
      'production',
      'destroy() called in production!'
    );
    try {
      await this.database.destroy();
    } catch {
      // ignored
    }
    this.database = new PouchDB(this.dbpath);
    await this.initialize();
  }

  /**
   * Create the database if it is missing. This must be called to connect to
   * the database and authenticate as a valid user before proceeding.
   */
  async initialize(): Promise<void> {
    await this.createIndices(assetsDefinition);
    await this.createIndices(newbornsDefinition);
  }

  /**
   * Add or update the design document.
   *
   * @param index - complete PouchDB document to be inserted or updated.
   * @returns true if the indices were created, false if updated.
   */
  async createIndices(index: any): Promise<void> {
    try {
      const oldDoc = await this.database.get(index._id);
      if (oldDoc.version === undefined || oldDoc.version < index.version) {
        // See commit 7f7a57d src/backend.js createIndices() for the old data
        // migration logic that could be useful again some day; on the other
        // hand, simply dumping and loading works well enough in most cases.
        await this.database.put({ ...index, _rev: oldDoc._rev });
        // clean up any stale indices from previous versions
        await this.database.viewCleanup();
      }
    } catch (error: any) {
      if (error.status === 404) {
        await this.database.post(index);
      } else {
        throw error;
      }
    }
  }

  /** @inheritDoc */
  async countAssets(): Promise<number> {
    // list() returns 'id', 'key', and 'value' which is an object with 'rev'
    const allDocs = await this.database.allDocs();
    // Count those documents that have id starting with "_design/" then subtract
    // that from the total_rows to find the true asset count.
    const designCount = allDocs.rows.reduce((acc: number, row: any) => {
      return row.id.startsWith('_design/') ? acc + 1 : acc;
    }, 0);
    return allDocs.total_rows - designCount;
  }

  /** @inheritDoc */
  async getAssetById(assetId: string): Promise<Asset | null> {
    try {
      const asset = await this.database.get(assetId);
      asset.key = assetId;
      return assetFromDocument(asset);
    } catch (error: any) {
      if (error.status === 404) {
        return null;
      }
      throw error;
    }
  }

  /** @inheritDoc */
  async getAssetByDigest(digest: string): Promise<Asset | null> {
    // should only be 1 result, but limit to 1 anyway
    const res = await this.database.query('assets/by_checksum', {
      key: digest.toLowerCase(),
      limit: 1,
      include_docs: true
    });
    if (res.rows.length > 0) {
      return assetFromDocument({ key: res.rows[0].id, ...res.rows[0].doc });
    }
    return null;
  }

  /** @inheritDoc */
  async allTags(): Promise<AttributeCount[]> {
    const res = await this.database.query('assets/all_tags', {
      group_level: 1
    });
    return res.rows.map((row: { key: string; value: number }) => {
      return new AttributeCount(row.key, row.value);
    });
  }

  /** @inheritDoc */
  async allPrimaryLabels(): Promise<AttributeCount[]> {
    const res = await this.database.query('assets/all_primary_labels', {
      group_level: 1
    });
    return res.rows.map((row: { key: string; value: number }) => {
      return new AttributeCount(row.key, row.value);
    });
  }

  /** @inheritDoc */
  async allLocations(): Promise<AttributeCount[]> {
    const res = await this.database.query('assets/all_location_parts', {
      group_level: 1
    });
    return res.rows.map((row: { key: string; value: number }) => {
      return new AttributeCount(row.key, row.value);
    });
  }

  /** @inheritDoc */
  async rawLocations(): Promise<Location[]> {
    const res = await this.database.query('assets/all_location_records', {
      group_level: 1
    });
    return res.rows.map((row: { key: string; value: number }) => {
      const parts = row.key.split('\t');
      return Location.fromParts(parts[0] || '', parts[1] || '', parts[2] || '');
    });
  }

  /** @inheritDoc */
  async allYears(): Promise<AttributeCount[]> {
    const res = await this.database.query('assets/all_years', {
      group_level: 1
    });
    return res.rows.map((row: { key: string; value: number }) => {
      // the view function emits the years as numbers but everything else
      // expects them to be strings
      return new AttributeCount(row.key.toString(), row.value);
    });
  }

  /** @inheritDoc */
  async allMediaTypes(): Promise<AttributeCount[]> {
    const res = await this.database.query('assets/all_media_types', {
      group_level: 1
    });
    return res.rows.map((row: { key: string; value: number }) => {
      return new AttributeCount(row.key, row.value);
    });
  }

  /** @inheritDoc */
  async putAsset(asset: Asset): Promise<void> {
    // strip `key` (it becomes _id), `metadata`, `synthetic`, and
    // `syntheticStatus` (each encoded separately to avoid copying class
    // instances into the doc).
    const { key, metadata, synthetic, syntheticStatus, ...doc } = asset;
    convertDatesIn(doc);
    (doc as any).metadata = metadataToDocument(metadata);
    (doc as any).synthetic = syntheticToDocument(synthetic, syntheticStatus);
    try {
      const oldDoc = await this.database.get(key);
      await this.database.put({ ...doc, _id: key, _rev: oldDoc._rev });
    } catch (error: any) {
      if (error.status === 404) {
        await this.database.put({ _id: key, ...doc });
      } else {
        throw error;
      }
    }
  }

  /** @inheritDoc */
  async fetchMetadata(
    assetIds: string[]
  ): Promise<Map<string, AssetMetadata | null>> {
    const result = new Map<string, AssetMetadata | null>();
    // Dedupe the input: PouchDB's allDocs does not, and we want one entry per
    // requested id regardless of repetition.
    const unique = Array.from(new Set(assetIds));
    for (const id of assetIds) result.set(id, null);
    if (unique.length === 0) return result;
    const res = await this.database.allDocs({
      keys: unique,
      include_docs: true
    });
    for (const row of res.rows) {
      // allDocs returns deleted-stub rows (value.deleted=true) and not-found
      // rows (error='not_found'); skip both.
      if (row.error) continue;
      if (row.value && row.value.deleted) continue;
      if (row.doc) {
        const metadata =
          metadataFromDocument(row.doc.metadata) ?? new AssetMetadata();
        metadata.byteLength = row.doc.byteLength ?? null;
        result.set(row.id, metadata);
      }
    }
    return result;
  }

  /** @inheritDoc */
  async fetchSynthetic(
    assetIds: string[]
  ): Promise<Map<string, SyntheticData | null>> {
    const result = new Map<string, SyntheticData | null>();
    const unique = Array.from(new Set(assetIds));
    for (const id of assetIds) result.set(id, null);
    if (unique.length === 0) return result;
    const res = await this.database.allDocs({
      keys: unique,
      include_docs: true
    });
    for (const row of res.rows) {
      if (row.error) continue;
      if (row.value && row.value.deleted) continue;
      if (row.doc) {
        const { data } = syntheticFromDocument(row.doc.synthetic);
        result.set(row.id, data);
      }
    }
    return result;
  }

  /** @inheritDoc */
  async fetchSyntheticStatus(
    assetIds: string[]
  ): Promise<Map<string, SyntheticStatus>> {
    const result = new Map<string, SyntheticStatus>();
    const unique = Array.from(new Set(assetIds));
    for (const id of assetIds) result.set(id, SyntheticStatus.PENDING);
    if (unique.length === 0) return result;
    const res = await this.database.allDocs({
      keys: unique,
      include_docs: true
    });
    for (const row of res.rows) {
      if (row.error) continue;
      if (row.value && row.value.deleted) continue;
      if (row.doc) {
        const { status } = syntheticFromDocument(row.doc.synthetic);
        result.set(row.id, status);
      }
    }
    return result;
  }

  /** @inheritDoc */
  async setSynthetic(
    assetId: string,
    data: SyntheticData | null,
    status: SyntheticStatus
  ): Promise<void> {
    // Conflict-retry loop: a concurrent updateAsset can bump _rev between the
    // .get() and .put(); without retry, every brief overlap surfaces as a
    // worker failure and burns a retry attempt. 404 is a no-op (asset was
    // deleted out from under us).
    const maxAttempts = 5;
    for (let attempt = 0; attempt < maxAttempts; attempt++) {
      let doc: any;
      try {
        doc = await this.database.get(assetId);
      } catch (error: any) {
        if (error?.status === 404 || error?.name === 'not_found') return;
        throw error;
      }
      doc.synthetic = syntheticToDocument(data, status);
      try {
        await this.database.put(doc);
        return;
      } catch (error: any) {
        if (
          (error?.status === 409 || error?.name === 'conflict') &&
          attempt < maxAttempts - 1
        ) {
          continue;
        }
        if (error?.status === 404 || error?.name === 'not_found') return;
        throw error;
      }
    }
  }

  /** @inheritDoc */
  async deleteAsset(assetId: string): Promise<void> {
    const asset = await this.database.get(assetId);
    await this.database.remove(asset._id, asset._rev);
  }

  /**
   * Find those search results that have _all_ of the given search keys.
   *
   * @param view name of the design document to search.
   * @param keys set of keys on which to query.
   * @returns those search results that contain all given keys.
   */
  async queryAllKeys(view: string, keys: string[]): Promise<SearchResult[]> {
    // find all documents that have any one of the given keys
    const queryResults = await this.database.query(`assets/${view}`, {
      keys: Array.from(keys)
        .map((e) => e.toLowerCase())
        .sort()
    });
    // reduce the documents to those that have all of the given keys
    const keyCounts = queryResults.rows.reduce((acc: any, row: any) => {
      const docId = row.id;
      const count = acc.has(docId) ? acc.get(docId) : 0;
      acc.set(docId, count + 1);
      return acc;
    }, new Map());
    const matchingRows = queryResults.rows.filter((row: any) => {
      return keyCounts.get(row.id) === keys.length;
    });
    // remove duplicate documents by sorting on the primary key
    const uniqueResults = matchingRows
      .sort((a: any, b: any) => {
        return a.id.localeCompare(b.id);
      })
      .filter(
        (row: any, idx: any, arr: any) =>
          idx === 0 || row.id !== arr[idx - 1].id
      );
    return uniqueResults.map((row: any) => convertViewResult(row));
  }

  /** @inheritDoc */
  async queryByTags(tags: string[]): Promise<SearchResult[]> {
    return this.queryAllKeys('by_tag', tags);
  }

  /** @inheritDoc */
  async queryByLabel(label: string): Promise<SearchResult[]> {
    const queryResults = await this.database.query('assets/by_primary_label', {
      key: label.toLowerCase()
    });
    return queryResults.rows.map((row: any) => convertViewResult(row));
  }

  /** @inheritDoc */
  async latestAssetByLabel(
    label: string
  ): Promise<{ assetId: string; primaryLabel: string } | null> {
    // One round-trip per label, bounded by docs-per-label. include_docs is
    // needed to recover the original-cased primary_label (the view emits a
    // lowercased key for case-insensitive matching).
    const res = await this.database.query('assets/by_primary_label', {
      key: label.toLowerCase(),
      include_docs: true
    });
    let bestId: string | null = null;
    let bestLabel: string | null = null;
    let bestDate = Number.NEGATIVE_INFINITY;
    for (const row of res.rows) {
      const bestdate = Array.isArray(row.value) ? Number(row.value[0]) : 0;
      const primary = row.doc?.synthetic?.primaryLabel;
      if (typeof primary !== 'string') continue;
      if (bestId === null || bestdate > bestDate) {
        bestId = row.id;
        bestLabel = primary;
        bestDate = bestdate;
      }
    }
    return bestId && bestLabel !== null
      ? { assetId: bestId, primaryLabel: bestLabel }
      : null;
  }

  /** @inheritDoc */
  async queryByLocations(locations: string[]): Promise<SearchResult[]> {
    return this.queryAllKeys('by_location', locations);
  }

  /** @inheritDoc */
  async queryByMediaType(media_type: string): Promise<SearchResult[]> {
    const queryResults = await this.database.query('assets/by_mimetype', {
      key: media_type.toLowerCase()
    });
    return queryResults.rows.map((row: any) => convertViewResult(row));
  }

  /** @inheritDoc */
  async queryBeforeDate(before: Date): Promise<SearchResult[]> {
    const queryResults = await this.database.query('assets/by_date', {
      endkey: before.getTime() - 1
    });
    return queryResults.rows.map((row: any) => convertViewResult(row));
  }

  /** @inheritDoc */
  async queryAfterDate(after: Date): Promise<SearchResult[]> {
    const queryResults = await this.database.query('assets/by_date', {
      startkey: after.getTime()
    });
    return queryResults.rows.map((row: any) => convertViewResult(row));
  }

  /** @inheritDoc */
  async queryDateRange(after: Date, before: Date): Promise<SearchResult[]> {
    const queryResults = await this.database.query('assets/by_date', {
      startkey: after.getTime(),
      endkey: before.getTime() - 1
    });
    return queryResults.rows.map((row: any) => convertViewResult(row));
  }

  /** @inheritDoc */
  async queryNewborn(after: Date): Promise<SearchResult[]> {
    const queryResults = await this.database.query('newborns/newborn', {
      startkey: after.getTime()
    });
    return queryResults.rows.map((row: any) => convertViewResult(row));
  }

  /** @inheritDoc */
  async fetchAssets(cursor: any, limit: number): Promise<[Asset[], any]> {
    // The cursor is either null, a document identifier, or 'done'. By using a
    // document identifier as the start key, PouchDB will begin retrieving
    // documents from that document. Thanks to PouchDB storing the records in a
    // log-structured merge tree, it will return the documents in natural order,
    // making for easy traversal of the entire collection.
    if (cursor === 'done') {
      return [[], cursor];
    }
    // add one to the limit to get the next key to return as the cursor
    limit++;
    // inexplicably, using undefined for startkey is literally passed as the
    // string 'undefined' to the PouchDB REST API
    const res = await this.database.allDocs(
      cursor
        ? {
            startkey: cursor,
            limit,
            include_docs: true
          }
        : { limit, include_docs: true }
    );
    if (res.rows.length === limit) {
      // there are more rows than requested, get the last one and use its
      // identifier as the cursor from which to start scanning next time
      cursor = res.rows.pop().id;
    } else {
      // there are fewer rows than requested, return a 'done' cursor
      cursor = 'done';
    }
    const assets = res.rows.reduce((acc: Array<Asset>, row: any) => {
      if (!row.id.startsWith('_design')) {
        acc.push(assetFromDocument({ key: row.id, ...row.doc }));
      }
      return acc;
    }, []);
    return [assets, cursor];
  }

  /** @inheritDoc */
  async storeAssets(incoming: Asset[]): Promise<void> {
    // db.bulk() requires both _id and _rev in order to update existing records
    for (const record of incoming) {
      await this.putAsset(record);
    }
  }
}

/**
 * PouchDB map/reduce view result.
 */
type ViewResult = {
  id: string;
  value: [
    // best date
    number,
    // filename
    string,
    // location
    Location | null,
    // mediaType
    string
  ];
};

// Define the map/reduce query views, whose functions are defined separately to
// allow writing pure JavaScript without the linters complaining.
//
// Some other views are defined in a separate design document for performance.
const assetsDefinition = {
  _id: '_design/assets',
  // monotonically increasing version number for tracking schema changes
  version: 4,
  views: {
    by_checksum: {
      map: views.by_checksum.toString(),
      options: { collation: 'raw' }
    },
    by_date: {
      map: views.insertBestDate(views.by_date)
    },
    by_filename: {
      map: views.insertBestDate(views.by_filename),
      options: { collation: 'raw' }
    },
    by_location: {
      map: views.insertBestDate(views.by_location)
    },
    by_mimetype: {
      map: views.insertBestDate(views.by_mimetype),
      options: { collation: 'raw' }
    },
    by_tag: {
      map: views.insertBestDate(views.by_tag)
    },
    all_location_records: {
      map: views.all_location_records.toString(),
      reduce: '_count'
    },
    all_location_parts: {
      map: views.all_location_parts.toString(),
      reduce: '_count'
    },
    all_tags: {
      map: views.all_tags.toString(),
      reduce: '_count'
    },
    all_years: {
      map: views.all_years.toString(),
      reduce: '_count'
    },
    all_media_types: {
      map: views.all_media_types.toString(),
      reduce: '_count'
    },
    by_primary_label: {
      map: views.insertBestDate(views.by_primary_label)
    },
    all_primary_labels: {
      map: views.all_primary_labels.toString(),
      reduce: '_count'
    }
  }
};

// Define the view for finding pending assets (those without tags, caption, or
// location label) in a separate design document in the hopes that this will
// shorten the time it takes for the updated view to be available. This view has
// more conditional logic than the others and should be run in its own
// (lightweight) process.
const newbornsDefinition = {
  _id: '_design/newborns',
  // monotonically increasing version number for tracking schema changes
  version: 1,
  views: {
    newborn: {
      map: views.insertBestDate(views.newborn)
    }
  }
};

/**
 * Convert the date fields in the document (in-place) from Date to number.
 *
 * @param doc - Asset-like record to write to PouchDB, modified in-place.
 * @returns the object for convenience.
 */
function convertDatesIn(doc: any): any {
  doc.importDate = doc.importDate.getTime();
  if (doc.userDate !== null) {
    doc.userDate = doc.userDate.getTime();
  }
  if (doc.originalDate !== null) {
    doc.originalDate = doc.originalDate.getTime();
  }
  return doc;
}

/**
 * Create an Asset entity from the given PouchDB document.
 *
 * @param doc - database record as read from PouchDB.
 * @returns converted asset entity.
 */
function assetFromDocument(doc: any): any {
  const asset = new Asset(doc.key);
  asset.setChecksum(doc.checksum);
  asset.setFilename(doc.filename);
  asset.setByteLength(doc.byteLength);
  asset.setMediaType(doc.mediaType);
  if (doc.tags) {
    asset.setTags(doc.tags);
  }
  asset.setImportDate(new Date(doc.importDate));
  if (doc.caption) {
    asset.setCaption(doc.caption);
  }
  if (doc.location) {
    const location = Location.fromRaw(
      doc.location.label,
      doc.location.city,
      doc.location.region
    );
    asset.setLocation(location);
  }
  if (doc.userDate !== null) {
    asset.setUserDate(new Date(doc.userDate));
  }
  if (doc.originalDate !== null) {
    asset.setOriginalDate(new Date(doc.originalDate));
  }
  const metadata = metadataFromDocument(doc.metadata) ?? new AssetMetadata();
  metadata.byteLength = doc.byteLength ?? null;
  asset.setMetadata(metadata);
  const { data, status } = syntheticFromDocument(doc.synthetic);
  asset.setSynthetic(data);
  asset.setSyntheticStatus(status);
  return asset;
}

/**
 * Convert the map/reduce view result into a SearchResult.
 *
 * @param {Object} result - row from the results of a map/reduce query.
 * @return result as an object.
 */
function convertViewResult(result: ViewResult): SearchResult {
  const lo = result.value[2]; // 2: location
  const location = Location.fromRaw(
    lo?.label || null,
    lo?.city || null,
    lo?.region || null
  );
  return new SearchResult(
    result.id, // assetId
    result.value[1], // 1: filename
    result.value[3], // 3: mediaType
    location,
    new Date(result.value[0]) // 0: datetime
  );
}

export { PouchDBRecordRepository };
