//
// Copyright (c) 2025 Nathan Fiedler
//
import assert from 'node:assert';
import fs from 'node:fs/promises';
import path from 'node:path';
import { Database } from 'bun:sqlite';
import { Asset } from 'tanuki/server/domain/entities/asset.ts';
import { AssetMetadata } from 'tanuki/server/domain/entities/asset-metadata.ts';
import { AttributeCount } from 'tanuki/server/domain/entities/attributes.ts';
import { Location } from 'tanuki/server/domain/entities/location.ts';
import { SearchResult } from 'tanuki/server/domain/entities/search.ts';
import { type RecordRepository } from 'tanuki/server/domain/repositories/record-repository.ts';
import { type SettingsRepository } from 'tanuki/server/domain/repositories/settings-repository.ts';
import { runMigrations } from './sqlite-migrations.ts';

/**
 * Repository for entity records stored in a SQLite database.
 */
class SqliteRecordRepository implements RecordRepository {
  dbpath: string;
  database: Database | null;

  constructor({
    settingsRepository
  }: {
    settingsRepository: SettingsRepository;
  }) {
    const basepath = settingsRepository.get('SQLITE_DBPATH');
    assert.ok(basepath, 'missing SQLITE_DBPATH environment variable');
    this.dbpath = path.join(basepath, 'tanuki.sqlite');
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
    if (this.database) {
      this.database.close(true);
    }
    try {
      await fs.unlink(this.dbpath);
    } catch (error: any) {
      if (error.code !== 'ENOENT') {
        throw error;
      }
    }
    await this.initialize();
  }

  /**
   * Create the database if it is missing. This must be called to connect to
   * the database and authenticate as a valid user before proceeding.
   */
  async initialize(): Promise<void> {
    await fs.mkdir(path.dirname(this.dbpath), { recursive: true });
    this.database = new Database(this.dbpath, { create: true });
    this.database.run('PRAGMA foreign_keys = ON;');
    this.database.run('PRAGMA journal_mode = WAL;');

    // For consistency with other data source implementations, treat certain
    // textual values case insensitively.
    //
    // The tags cell consists of the asset tags separated by a tab (0x09).
    this.database.run(
      `CREATE TABLE IF NOT EXISTS assets (
        key TEXT NOT NULL PRIMARY KEY,
        hash TEXT NOT NULL COLLATE NOCASE,
        filename TEXT NOT NULL,
        filesize INTEGER NOT NULL,
        mimetype TEXT NOT NULL COLLATE NOCASE,
        caption TEXT,
        tags TEXT,
        loc_label TEXT,
        loc_city TEXT,
        loc_region TEXT,
        imported INTEGER NOT NULL,
        user_date INTEGER,
        orig_date INTEGER,
        year TEXT AS (STRFTIME('%Y', DATETIME(COALESCE(user_date, orig_date, imported), 'unixepoch'))) STORED
      ) STRICT`
    );

    // create indices for frequent queries on certain columns
    this.database.run(
      'CREATE UNIQUE INDEX IF NOT EXISTS hashes ON assets (hash)'
    );
    this.database.run(
      'CREATE INDEX IF NOT EXISTS dates ON assets (coalesce(user_date, orig_date, imported))'
    );

    // create view for finding all tags
    this.database.run(
      `CREATE VIEW IF NOT EXISTS tags_view AS
        WITH RECURSIVE split(tag, rest) AS (
            -- Anchor member: begin recursion, set up the initial row with trailing delimiter
            SELECT '', LOWER(tags) || CHAR(9) FROM assets
            UNION ALL
            -- Recursive member: Extracts one tag at a time
            SELECT
                substr(rest, 1, instr(rest, CHAR(9)) - 1),
                substr(rest, instr(rest, CHAR(9)) + 1)
            FROM
                split
            -- Termination condition: stop when there are no more delimiters
            WHERE
                instr(rest, CHAR(9)) > 0
        )
        -- Final SELECT to retrieve the results
        SELECT tag FROM split WHERE tag != '';`
    );

    // create view for finding all location parts
    this.database.run(
      `CREATE VIEW IF NOT EXISTS locations_view AS
        SELECT LOWER(loc_label) AS value FROM assets WHERE loc_label IS NOT NULL AND loc_label != ''
        UNION ALL
        SELECT LOWER(loc_city) FROM assets WHERE loc_city IS NOT NULL AND loc_city != ''
        UNION ALL
        SELECT LOWER(loc_region) FROM assets WHERE loc_region IS NOT NULL AND loc_region != '';`
    );

    runMigrations(this.database);
  }

  /** @inheritDoc */
  async countAssets(): Promise<number> {
    const query = this.database!.query('SELECT COUNT(*) AS count FROM assets');
    const row = query.get() as { count: number } | undefined;
    return row!.count;
  }

  /** @inheritDoc */
  async getAssetById(assetId: string): Promise<Asset | null> {
    const query = this.database!.query('SELECT * FROM assets WHERE key = ?;');
    const row = query.get(assetId);
    if (!row) return null;
    const asset = assetFromRow(row);
    asset.metadata = this.loadMetadata(assetId);
    return asset;
  }

  /** @inheritDoc */
  async getAssetByDigest(digest: string): Promise<Asset | null> {
    const query = this.database!.query('SELECT * FROM assets WHERE hash = ?;');
    const row = query.get(digest);
    if (!row) return null;
    const asset = assetFromRow(row);
    asset.metadata = this.loadMetadata(asset.key);
    return asset;
  }

  private loadMetadata(assetId: string): AssetMetadata | null {
    const row = this.database!
      .query('SELECT * FROM metadata WHERE asset_id = ?')
      .get(assetId);
    return row ? metadataFromRow(row) : null;
  }

  /** @inheritDoc */
  async allTags(): Promise<AttributeCount[]> {
    const query = this.database!.query(
      'SELECT tag AS label, COUNT(*) AS count FROM tags_view GROUP BY tag;'
    ).as(AttributeCount);
    return query.all();
  }

  /** @inheritDoc */
  async allLocations(): Promise<AttributeCount[]> {
    const query = this.database!.query(
      'SELECT value AS label, COUNT(*) AS count FROM locations_view GROUP BY value;'
    ).as(AttributeCount);
    return query.all();
  }

  /** @inheritDoc */
  async rawLocations(): Promise<Location[]> {
    const query = this.database!.query(
      `SELECT DISTINCT CONCAT(loc_label, ';', loc_city, ',', loc_region) AS location FROM assets;`
    );
    const results: Location[] = [];
    for (const row of query) {
      const location = Location.parse((row as any).location);
      if (location) {
        results.push(location);
      }
    }
    return results;
  }

  /** @inheritDoc */
  async allYears(): Promise<AttributeCount[]> {
    const query = this.database!.query(
      'SELECT year AS label, COUNT(*) AS count FROM assets GROUP BY year;'
    ).as(AttributeCount);
    return query.all();
  }

  /** @inheritDoc */
  async allMediaTypes(): Promise<AttributeCount[]> {
    const query = this.database!.query(
      'SELECT mimetype AS label, COUNT(*) AS count FROM assets GROUP BY mimetype;'
    ).as(AttributeCount);
    return query.all();
  }

  /** @inheritDoc */
  async putAsset(asset: Asset): Promise<void> {
    // attempt to insert a new row, but on conflict update only certain
    // fields which can be changed by the update usecase
    const insert = this.database!.query(
      `INSERT INTO assets (key, hash, filename, filesize, mimetype, caption,
        tags, loc_label, loc_city, loc_region, imported, user_date, orig_date)
       VALUES ($key, $hash, $filename, $filesize, $mimetype, $caption,
        $tags, $loc_label, $loc_city, $loc_region, $imported, $user_date, $orig_date)
       ON CONFLICT DO UPDATE SET filename = $filename, mimetype = $mimetype,
        caption = $caption, tags = $tags, loc_label = $loc_label,
        loc_city = $loc_city, loc_region = $loc_region, user_date = $user_date;`
    );
    const user_date = asset.userDate
      ? Math.trunc(asset.userDate.getTime() / 1000)
      : null;
    const orig_date = asset.originalDate
      ? Math.trunc(asset.originalDate.getTime() / 1000)
      : null;
    // Wrap the asset upsert and metadata upsert in a single transaction so a
    // failure in either rolls back both writes.
    this.database!.transaction(() => {
      insert.run({
        $key: asset.key,
        $hash: asset.checksum,
        $filename: asset.filename,
        $filesize: asset.byteLength,
        $mimetype: asset.mediaType,
        $caption: asset.caption || null,
        $tags: asset.tags.join('\t') || null,
        $loc_label: asset.location?.label || null,
        $loc_city: asset.location?.city || null,
        $loc_region: asset.location?.region || null,
        $imported: Math.trunc(asset.importDate.getTime() / 1000),
        $user_date: user_date,
        $orig_date: orig_date
      });
      this.upsertMetadata(asset.key, asset.metadata);
    })();
  }

  private upsertMetadata(
    assetId: string,
    metadata: AssetMetadata | null
  ): void {
    if (metadata === null || !metadata.hasValues()) {
      this.database!
        .query('DELETE FROM metadata WHERE asset_id = ?')
        .run(assetId);
      return;
    }
    this.database!
      .query(
        `INSERT INTO metadata (
          asset_id, camera_make, camera_model, lens_make, lens_model,
          exposure_time, f_number, iso, focal_length_35mm, original_date_offset,
          gps_latitude, gps_longitude, display_width, display_height,
          duration, frame_rate, video_codec, raw
        ) VALUES (
          $asset_id, $camera_make, $camera_model, $lens_make, $lens_model,
          $exposure_time, $f_number, $iso, $focal_length_35mm, $original_date_offset,
          $gps_latitude, $gps_longitude, $display_width, $display_height,
          $duration, $frame_rate, $video_codec, $raw
        )
        ON CONFLICT(asset_id) DO UPDATE SET
          camera_make = $camera_make, camera_model = $camera_model,
          lens_make = $lens_make, lens_model = $lens_model,
          exposure_time = $exposure_time, f_number = $f_number, iso = $iso,
          focal_length_35mm = $focal_length_35mm,
          original_date_offset = $original_date_offset,
          gps_latitude = $gps_latitude, gps_longitude = $gps_longitude,
          display_width = $display_width, display_height = $display_height,
          duration = $duration, frame_rate = $frame_rate,
          video_codec = $video_codec, raw = $raw;`
      )
      .run({
        $asset_id: assetId,
        $camera_make: metadata.cameraMake,
        $camera_model: metadata.cameraModel,
        $lens_make: metadata.lensMake,
        $lens_model: metadata.lensModel,
        $exposure_time: metadata.exposureTime,
        $f_number: metadata.fNumber,
        $iso: metadata.iso,
        $focal_length_35mm: metadata.focalLength35mm,
        $original_date_offset: metadata.originalDateOffset,
        $gps_latitude: metadata.gpsLatitude,
        $gps_longitude: metadata.gpsLongitude,
        $display_width: metadata.displayWidth,
        $display_height: metadata.displayHeight,
        $duration: metadata.duration,
        $frame_rate: metadata.frameRate,
        $video_codec: metadata.videoCodec,
        $raw: metadata.raw ? JSON.stringify(metadata.raw) : null
      });
  }

  /** @inheritDoc */
  async deleteAsset(assetId: string): Promise<void> {
    const rm = this.database!.query('DELETE FROM assets WHERE key = ?');
    rm.run(assetId);
  }

  /** @inheritDoc */
  async queryByTags(tags: string[]): Promise<SearchResult[]> {
    const query = this.database!.query(
      `SELECT key, filename, mimetype, loc_label, loc_city, loc_region,
          coalesce(user_date, orig_date, imported) AS date, LOWER(tags) AS tags
        FROM assets WHERE tags IS NOT NULL;`
    );
    const expected = new Set(tags.map((t) => t.toLowerCase()));
    const results: SearchResult[] = [];
    for (const row of query) {
      const actual: string[] = (row as any).tags.split('\t');
      const matchCount = actual.reduce((acc, value) => {
        if (expected.has(value)) {
          return acc + 1;
        }
        return acc;
      }, 0);
      if (matchCount == tags.length) {
        results.push(searchResultFromRow(row));
      }
    }
    return results;
  }

  /** @inheritDoc */
  async queryByLocations(locations: string[]): Promise<SearchResult[]> {
    const query = this.database!.query(
      `SELECT key, filename, mimetype, loc_label, loc_city, loc_region,
          coalesce(user_date, orig_date, imported) AS date FROM assets;`
    );
    const expected = new Set(locations.map((l) => l.toLowerCase()));
    const results: SearchResult[] = [];
    for (const row of query) {
      const actual: string[] = [
        (row as any).loc_label,
        (row as any).loc_city,
        (row as any).loc_region
      ].map((v) => v?.toLowerCase());
      const matchCount = actual.reduce((acc, value) => {
        if (expected.has(value)) {
          return acc + 1;
        }
        return acc;
      }, 0);
      if (matchCount == locations.length) {
        results.push(searchResultFromRow(row));
      }
    }
    return results;
  }

  /** @inheritDoc */
  async queryByMediaType(media_type: string): Promise<SearchResult[]> {
    const query = this.database!.query(
      `SELECT key, filename, mimetype, loc_label, loc_city, loc_region,
          coalesce(user_date, orig_date, imported) AS date
        FROM assets WHERE mimetype = ?;`
    );
    const results: SearchResult[] = [];
    for (const row of query.iterate(media_type)) {
      results.push(searchResultFromRow(row));
    }
    return results;
  }

  /** @inheritDoc */
  async queryBeforeDate(before: Date): Promise<SearchResult[]> {
    //
    // SQLite "indexes on expressions" stipulates that the expression used
    // to query the table must match the one used to define the index, so be
    // sure the coalesce() invocation matches the index precisely.
    //
    const query = this.database!.query(
      `SELECT key, filename, mimetype, loc_label, loc_city, loc_region,
          coalesce(user_date, orig_date, imported) AS date
        FROM assets WHERE date < ?;`
    );
    const results: SearchResult[] = [];
    for (const row of query.iterate(Math.trunc(before.getTime() / 1000))) {
      results.push(searchResultFromRow(row));
    }
    return results;
  }

  /** @inheritDoc */
  async queryAfterDate(after: Date): Promise<SearchResult[]> {
    //
    // SQLite "indexes on expressions" stipulates that the expression used
    // to query the table must match the one used to define the index, so be
    // sure the coalesce() invocation matches the index precisely.
    //
    const query = this.database!.query(
      `SELECT key, filename, mimetype, loc_label, loc_city, loc_region,
          coalesce(user_date, orig_date, imported) AS date
        FROM assets WHERE date >= ?;`
    );
    const results: SearchResult[] = [];
    for (const row of query.iterate(Math.trunc(after.getTime() / 1000))) {
      results.push(searchResultFromRow(row));
    }
    return results;
  }

  /** @inheritDoc */
  async queryDateRange(after: Date, before: Date): Promise<SearchResult[]> {
    //
    // SQLite "indexes on expressions" stipulates that the expression used
    // to query the table must match the one used to define the index, so be
    // sure the coalesce() invocation matches the index precisely.
    //
    const query = this.database!.query(
      `SELECT key, filename, mimetype, loc_label, loc_city, loc_region,
          coalesce(user_date, orig_date, imported) AS date
        FROM assets WHERE date >= ?1 AND date < ?2;`
    );
    const results: SearchResult[] = [];
    for (const row of query.iterate(
      Math.trunc(after.getTime() / 1000),
      Math.trunc(before.getTime() / 1000)
    )) {
      results.push(searchResultFromRow(row));
    }
    return results;
  }

  /** @inheritDoc */
  async queryNewborn(after: Date): Promise<SearchResult[]> {
    const query = this.database!.query(
      `SELECT key, filename, mimetype, loc_label, loc_city, loc_region,
          coalesce(user_date, orig_date, imported) AS date
        FROM assets
        WHERE imported >= ? AND tags IS NULL AND caption IS NULL AND loc_label IS NULL;`
    );
    const results: SearchResult[] = [];
    for (const row of query.iterate(Math.trunc(after.getTime() / 1000))) {
      results.push(searchResultFromRow(row));
    }
    return results;
  }

  /** @inheritDoc */
  async fetchAssets(cursor: any, limit: number): Promise<[Asset[], any]> {
    // The cursor is either null, a document identifier, or 'done'.
    if (cursor === 'done') {
      return [[], cursor];
    }
    // Default the starting point with "0" as that will work despite looking
    // like a hack (asset identifiers always start with "M").
    const query = this.database!.query(
      `SELECT * FROM assets WHERE key > ?1 ORDER BY key LIMIT ?2;`
    );
    const results: Asset[] = [];
    for (const row of query.iterate(cursor ?? '0', limit)) {
      results.push(assetFromRow(row));
    }
    if (results.length > 0) {
      const metadataMap = await this.fetchMetadata(results.map((a) => a.key));
      for (const asset of results) {
        asset.metadata = metadataMap.get(asset.key) ?? null;
      }
    }
    cursor = results.at(-1)?.key ?? 'done';
    return [results, cursor];
  }

  /** @inheritDoc */
  async fetchMetadata(
    assetIds: string[]
  ): Promise<Map<string, AssetMetadata | null>> {
    const result = new Map<string, AssetMetadata | null>();
    for (const id of assetIds) result.set(id, null);
    if (assetIds.length === 0) return result;
    const placeholders = assetIds.map(() => '?').join(',');
    const query = this.database!.query(
      `SELECT * FROM metadata WHERE asset_id IN (${placeholders})`
    );
    for (const row of query.iterate(...assetIds)) {
      result.set((row as any).asset_id, metadataFromRow(row));
    }
    return result;
  }

  /** @inheritDoc */
  async storeAssets(incoming: Asset[]): Promise<void> {
    const insert = this.database!.query(
      `INSERT OR REPLACE INTO assets (key, hash, filename, filesize, mimetype, caption,
        tags, loc_label, loc_city, loc_region, imported, user_date, orig_date)
       VALUES ($key, $hash, $filename, $filesize, $mimetype, $caption,
        $tags, $loc_label, $loc_city, $loc_region, $imported, $user_date, $orig_date);`
    );
    this.database!.transaction(() => {
      for (const asset of incoming) {
        const user_date = asset.userDate
          ? Math.trunc(asset.userDate.getTime() / 1000)
          : null;
        const orig_date = asset.originalDate
          ? Math.trunc(asset.originalDate.getTime() / 1000)
          : null;
        insert.run({
          $key: asset.key,
          $hash: asset.checksum,
          $filename: asset.filename,
          $filesize: asset.byteLength,
          $mimetype: asset.mediaType,
          $caption: asset.caption || null,
          $tags: asset.tags.join('\t') || null,
          $loc_label: asset.location?.label || null,
          $loc_city: asset.location?.city || null,
          $loc_region: asset.location?.region || null,
          $imported: Math.trunc(asset.importDate.getTime() / 1000),
          $user_date: user_date,
          $orig_date: orig_date
        });
        this.upsertMetadata(asset.key, asset.metadata);
      }
    })();
  }
}

/**
 * Create an Asset entity from the given database row.
 *
 * @param row - database row as read from SQLite.
 * @returns converted asset entity.
 */
function assetFromRow(row: any): any {
  const asset = new Asset(row.key);
  asset.setChecksum(row.hash);
  asset.setFilename(row.filename);
  asset.setByteLength(row.filesize);
  asset.setMediaType(row.mimetype);
  if (row.tags) {
    asset.setTags(row.tags.split('\t'));
  }
  asset.setImportDate(new Date(row.imported * 1000));
  if (row.caption) {
    asset.setCaption(row.caption);
  }
  const location = Location.fromRaw(
    row.loc_label,
    row.loc_city,
    row.loc_region
  );
  if (location.hasValues()) {
    asset.setLocation(location);
  }
  if (row.user_date !== null) {
    asset.setUserDate(new Date(row.user_date * 1000));
  }
  if (row.orig_date !== null) {
    asset.setOriginalDate(new Date(row.orig_date * 1000));
  }
  return asset;
}

function metadataFromRow(row: any): AssetMetadata {
  const m = new AssetMetadata();
  m.cameraMake = row.camera_make ?? null;
  m.cameraModel = row.camera_model ?? null;
  m.lensMake = row.lens_make ?? null;
  m.lensModel = row.lens_model ?? null;
  m.exposureTime = row.exposure_time ?? null;
  m.fNumber = row.f_number ?? null;
  m.iso = row.iso ?? null;
  m.focalLength35mm = row.focal_length_35mm ?? null;
  m.originalDateOffset = row.original_date_offset ?? null;
  m.gpsLatitude = row.gps_latitude ?? null;
  m.gpsLongitude = row.gps_longitude ?? null;
  m.displayWidth = row.display_width ?? null;
  m.displayHeight = row.display_height ?? null;
  m.duration = row.duration ?? null;
  m.frameRate = row.frame_rate ?? null;
  m.videoCodec = row.video_codec ?? null;
  if (row.raw) {
    try {
      m.raw = JSON.parse(row.raw);
    } catch {
      m.raw = null;
    }
  }
  return m;
}

function searchResultFromRow(row: any): SearchResult {
  // SELECT key, filename, mimetype, loc_label, loc_city, loc_region,
  //     coalesce(user_date, orig_date, imported) AS date, LOWER(tags) AS tags
  const location = Location.fromRaw(
    row.loc_label,
    row.loc_city,
    row.loc_region
  );
  const date = new Date(row.date * 1000);
  return new SearchResult(row.key, row.filename, row.mimetype, location, date);
}

export { SqliteRecordRepository };
