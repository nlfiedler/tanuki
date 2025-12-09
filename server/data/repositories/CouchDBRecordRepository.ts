//
// Copyright (c) 2025 Nathan Fiedler
//
import assert from 'node:assert';
import nano from 'nano';
import { Asset } from 'tanuki/server/domain/entities/Asset.ts';
import { AttributeCount } from 'tanuki/server/domain/entities/AttributeCount.ts';
import { Location } from 'tanuki/server/domain/entities/Location.ts';
import { SearchResult } from 'tanuki/server/domain/entities/SearchResult.ts';
import { type RecordRepository } from 'tanuki/server/domain/repositories/RecordRepository.ts';
import { type SettingsRepository } from 'tanuki/server/domain/repositories/SettingsRepository.ts';
import * as views from './couchdb_views.js';

/**
 * Repository for entity records stored in a CouchDB database.
 */
class CouchDBRecordRepository implements RecordRepository {
  url: string;
  dbname: string;
  username: string;
  password: string;
  conn: any;
  database: any;
  heartbeat: number;

  constructor({
    settingsRepository
  }: {
    settingsRepository: SettingsRepository;
  }) {
    this.url = settingsRepository.get('DATABASE_URL');
    assert.ok(this.url, 'missing DATABASE_URL environment variable');
    this.dbname = settingsRepository.get('DATABASE_NAME');
    assert.ok(this.dbname, 'missing DATABASE_NAME environment variable');
    this.username = settingsRepository.get('DATABASE_USER');
    assert.ok(this.username, 'missing DATABASE_USER environment variable');
    this.password = settingsRepository.get('DATABASE_PASSWORD');
    assert.ok(this.password, 'missing DATABASE_PASSWORD environment variable');
    this.heartbeat = settingsRepository.getInt('DATABASE_HEARTBEAT_MS', 60000);
    this.conn = nano(this.url);
    this.database = null;
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
    await this.conn.auth(this.username, this.password);
    try {
      await this.conn.db.destroy(this.dbname);
    } catch {
      // ignored
    }
    await this.createIfMissing();
  }

  /**
   * Create the database if it is missing. This must be called to connect to
   * the database and authenticate as a valid user before proceeding.
   */
  async createIfMissing(): Promise<void> {
    try {
      await this.conn.auth(this.username, this.password);
      await this.conn.db.get(this.dbname);
      this.database = this.conn.db.use(this.dbname);
    } catch (err: any) {
      if (err.statusCode == 404) {
        await this.conn.db.create(this.dbname);
        this.database = this.conn.db.use(this.dbname);
      } else {
        throw err;
      }
    }
    await this.createIndices(assetsDefinition);
    this.stayAlive();
  }

  /**
   * Add or update the design document.
   *
   * @param index - complete CouchDB document to be inserted or updated.
   * @returns true if the indices were created, false if updated.
   */
  async createIndices(index: any): Promise<void> {
    try {
      const oldDoc = await this.database.get(index._id);
      if (oldDoc.version === undefined || oldDoc.version < index.version) {
        // See commit 7f7a57d src/backend.js createIndices() for the old data
        // migration logic that could be useful again some day; on the other
        // hand, simply dumping and loading works well enough in most cases.
        await this.database.insert({ ...index, _rev: oldDoc._rev });
        // clean up any stale indices from previous versions
        //
        // missing from API, see https://github.com/apache/couchdb-nano/issues/25
        //
        // await this.database.viewCleanup()
      }
    } catch (err: any) {
      if (err.statusCode === 404) {
        await this.database.insert(index);
      } else {
        throw err;
      }
    }
  }

  // Use the database "changes" feed to keep the connection alive indefinitely.
  stayAlive() {
    // CPU activity is negligible with this approach, while the 'continuous' mode
    // keeps the CPU busy on both this system and the host running CouchDB.
    setInterval(async () => {
      await this.database.changes(this.dbname, { since: 'now', limit: 1 });
    }, this.heartbeat);
  }

  /** @inheritDoc */
  async countAssets(): Promise<number> {
    // list() returns 'id', 'key', and 'value' which is an object with 'rev'
    const allDocs = await this.database.list();
    // Count those documents that have id starting with "_design/" then subtract
    // that from the total_rows to find the true asset count.
    const designCount = allDocs.rows.reduce((acc: number, row: any) => {
      return row.id.startsWith('_design/') ? acc + 1 : acc;
    }, 0);
    return allDocs.total_rows - designCount;
  }

  /** @inheritDoc */
  async getAssetById(assetId: string): Promise<Asset | null> {
    const asset = await this.database.get(assetId);
    if (asset !== null) {
      asset.key = assetId;
      return assetFromDocument(asset);
    }
    return null;
  }

  /** @inheritDoc */
  async getAssetByDigest(digest: string): Promise<Asset | null> {
    // should only be 1 result, but limit to 1 anyway
    const res = await this.database.view('assets', 'by_checksum', {
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
    const res = await this.database.view('assets', 'all_tags', {
      group_level: 1
    });
    return res.rows.map((row: { key: string; value: number }) => {
      return new AttributeCount(row.key, row.value);
    });
  }

  /** @inheritDoc */
  async allLocations(): Promise<AttributeCount[]> {
    const res = await this.database.view('assets', 'all_location_parts', {
      group_level: 1
    });
    return res.rows.map((row: { key: string; value: number }) => {
      return new AttributeCount(row.key, row.value);
    });
  }

  /** @inheritDoc */
  async rawLocations(): Promise<Location[]> {
    const res = await this.database.view('assets', 'all_location_records', {
      group_level: 1
    });
    return res.rows.map((row: { key: string; value: number }) => {
      const parts = row.key.split('\t');
      return Location.fromParts(parts[0] || '', parts[1] || '', parts[2] || '');
    });
  }

  /** @inheritDoc */
  async allYears(): Promise<AttributeCount[]> {
    const res = await this.database.view('assets', 'all_years', {
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
    const res = await this.database.view('assets', 'all_media_types', {
      group_level: 1
    });
    return res.rows.map((row: { key: string; value: number }) => {
      return new AttributeCount(row.key, row.value);
    });
  }

  /** @inheritDoc */
  async putAsset(asset: Asset): Promise<void> {
    // strip the `key` property since that will be the _id anyway
    const { key, ...doc } = asset;
    convertDatesIn(doc);
    try {
      const oldDoc = await this.database.get(key);
      await this.database.insert({ ...doc, _id: key, _rev: oldDoc._rev });
    } catch (err: any) {
      if (err.statusCode === 404) {
        await this.database.insert(doc, key);
      } else {
        throw err;
      }
    }
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
    const queryResults = await this.database.view('assets', view, {
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
  async queryByLocations(locations: string[]): Promise<SearchResult[]> {
    return this.queryAllKeys('by_location', locations);
  }

  /** @inheritDoc */
  async queryByMediaType(media_type: string): Promise<SearchResult[]> {
    const queryResults = await this.database.view('assets', 'by_mimetype', {
      key: media_type.toLowerCase()
    });
    return queryResults.rows.map((row: any) => convertViewResult(row));
  }

  /** @inheritDoc */
  async queryBeforeDate(before: Date): Promise<SearchResult[]> {
    const queryResults = await this.database.view('assets', 'by_date', {
      endkey: before.getTime() - 1
    });
    return queryResults.rows.map((row: any) => convertViewResult(row));
  }

  /** @inheritDoc */
  async queryAfterDate(after: Date): Promise<SearchResult[]> {
    const queryResults = await this.database.view('assets', 'by_date', {
      startkey: after.getTime()
    });
    return queryResults.rows.map((row: any) => convertViewResult(row));
  }

  /** @inheritDoc */
  async queryDateRange(after: Date, before: Date): Promise<SearchResult[]> {
    const queryResults = await this.database.view('assets', 'by_date', {
      startkey: after.getTime(),
      endkey: before.getTime() - 1
    });
    return queryResults.rows.map((row: any) => convertViewResult(row));
  }

  /** @inheritDoc */
  async queryNewborn(after: Date): Promise<SearchResult[]> {
    const queryResults = await this.database.view('assets', 'newborn', {
      startkey: after.getTime()
    });
    return queryResults.rows.map((row: any) => convertViewResult(row));
  }

  /** @inheritDoc */
  async fetchAssets(cursor: any, limit: number): Promise<[Asset[], any]> {
    // The cursor is either null, a document identifier, or 'done'. By using a
    // document identifier as the start key, CouchDB will begin retrieving
    // documents from that document. Thanks to CouchDB storing the records in a
    // B-tree, it will return the documents in natural order, making for easy
    // traversal of the entire collection.
    if (cursor === 'done') {
      return [[], cursor];
    }
    // add one to the limit to get the next key to return as the cursor
    limit++;
    // inexplicably, using undefined for startkey is literally passed as the
    // string 'undefined' to the CouchDB REST API
    const res = await this.database.list(
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
      if (row.id !== '_design/assets') {
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
 * CouchDB map/reduce view result.
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
const assetsDefinition = {
  _id: '_design/assets',
  // monotonically increasing version number for tracking schema changes
  version: 1,
  views: {
    by_checksum: {
      map: views.by_checksum.toString()
    },
    by_date: {
      map: views.insertBestDate(views.by_date)
    },
    by_filename: {
      map: views.insertBestDate(views.by_filename)
    },
    by_location: {
      map: views.insertBestDate(views.by_location)
    },
    by_mimetype: {
      map: views.insertBestDate(views.by_mimetype)
    },
    by_tag: {
      map: views.insertBestDate(views.by_tag)
    },
    newborn: {
      map: views.insertBestDate(views.newborn)
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
    }
  }
};

/**
 * Convert the date fields in the document (in-place) from Date to number.
 *
 * @param doc - Asset-like record to write to CouchDB, modified in-place.
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
 * Create an Asset entity from the given CouchDB document.
 *
 * @param doc - database record as read from CouchDB.
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

export { CouchDBRecordRepository };
