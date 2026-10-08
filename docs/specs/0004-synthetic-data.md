# Synthetic Data for Assets

Building on the previous specification concerning asset metadata, this document concerns the introduction of synthetic data that is derived from an image asset using machine learning models that provide facial recognition and image classification (label tagging). The changes are broken down into phases. Non-image assets are ignored.

The synthetic data of particular interest:

- a list of **people** (opaque person ids) detected in the image
- a list of detected **labels** — concept/scene/object tags produced by a multi-label image classifier (e.g. `beach`, `dog`, `wedding gown`)
- a designated **primary label**, the highest-confidence label, surfaced at the front of the labels list

For people, the goal is to detect faces, embed each face as a feature vector, and cluster vectors into people. The web interface shows each unique person with a representative thumbnail and lets the user assign a name to that person.

For labels, the goal mirrors PhotoPrism's behavior: run a CPU-friendly image classifier (**MobileNetV2**) trained on ImageNet, take the top-K outputs by score, and apply a curated mapping to turn the raw 1000-class ImageNet vocabulary into a smaller, human-friendly tag set. MobileNetV2 is roughly as accurate as PhotoPrism's NASNet-Mobile choice for our purposes, ships as an official ONNX from the ONNX Model Zoo (no conversion ceremony required), and unifies the runtime story with the face-recognition path (both via `onnxruntime-node` / `tract`).

## Phases

Work is split into two phases. Each phase ends with a working UI.

### Phase 1 — Labels

Deliverables:

- Image classification integrated into ingestion and backfill
- `SyntheticData` GraphQL type with `labels: [String!]!` and `primaryLabel: String`
- `syntheticStatus` field on `Asset`
- `synthetic_data` table (SQLite) and equivalent fields on CouchDB/PouchDB documents
- `synthetic_jobs` table, worker pool, retry mutation
- `backfillLabels` mutation
- Label curation map (see "Label vocabulary" below)
- `Labels` page on the client
- Search syntax extension: `label:<class>`
- `thumb-list.tsx` updated to show primary label as the first line

Acceptance: a freshly imported image gets a primary label within seconds (background queue), the Labels page lists every distinct primary label with a representative thumbnail, and `label:beach` works in the search bar.

### Phase 2 — Faces and People

Deliverables:

- Face detection + embedding integrated into ingestion and backfill
- `face` and `person` tables (SQLite) and equivalent CouchDB/PouchDB structure
- Online clustering during ingestion
- `people: [Person!]!` on `SyntheticData`; `Person` type with `id`, `name`, `thumbnail`, `hidden`
- `backfillFaceRecognition` mutation
- Cluster lifecycle mutations: `renamePerson`, `mergePeople`, `reassignFaces`, `hidePerson`, `setPersonThumbnail`
- `People` page on the client (name editing, click-through to assets, manual reassignment UI)
- Search syntax extension: `person:<id>`

Acceptance: a freshly imported image with a known face is grouped under the right person; the People page allows renaming, merging, splitting, and hiding clusters; `person:<id>` works in the search bar.

### Face models

Both Tanuki (local detector) and Namazu (Rust detector) use the same ONNX model files so that embeddings are directly comparable across backends:

- **Face detection**: SCRFD-2.5g (~3 MB ONNX) — produces bounding boxes + 5-point landmarks.
- **Face embedding**: MobileFaceNet (~5 MB ONNX, ArcFace-trained, `w600k_mbf` variant) — produces 512-dimensional L2-normalized embeddings.

Both backends ship the same `scrfd_2.5g.onnx` and `mobilefacenet.onnx` files, and use the same `model_version` identifier (e.g. `mobilefacenet-v1`). The embeddings are byte-comparable: a face recognized as Alice by the Namazu detector will match the same Alice cluster after re-import through the local detector.

#### SCRFD inference parameters

Both backends use the InsightFace defaults:

- **Detection score threshold**: `0.5` — discard candidate faces scoring below this.
- **NMS IoU threshold**: `0.4` — suppress overlapping detections.

These are well-characterized defaults and should not be tuned in v1. They can be promoted to env-var knobs later if real usage demands it.

## Label vocabulary

MobileNetV2 outputs the **ImageNet-1000** vocabulary. Raw ImageNet labels are too granular and noisy for end users: many duplicate categories ("tabby cat" vs. "Egyptian cat"), hyper-specific cultivars ("Granny Smith"), and noise classes ("comic book", "envelope") would clutter the Labels page.

The detector pipeline applies a **curated mapping** before emitting `labels[]`:

- Input: 1000 raw ImageNet class names.
- Output: a smaller, deduplicated, human-friendly vocabulary.
- Mechanism: a JSON file shipped with the application, mapping raw label → display label or `null`. Raw labels mapped to `null` are dropped entirely.
- The mapping is the source of truth for both the Tanuki local detector and the Namazu Rust detector — both must read and apply the same file so outputs are consistent across backends.
- The mapping is curated by hand for Tanuki. (PhotoPrism takes a similar but separate approach — a hand-edited YAML config that they compile into Go source code; we draw inspiration from their categorization without sharing any data file.) The mapping lives in the Tanuki repo at `server/data/synthetic/labels-map.json` and is copied into the Namazu deployment artifact.

Each entry in `labels-map.json` has three fields: `raw` (the original ImageNet class name), `label` (the curated display label, or `null` to drop), and `category` (one of `animal`, `plant`, `food`, `nature`, `person`, `vehicle`, `building`, `clothing`, `furniture`, `instrument`, `electronics`, `tool`, `weapon`, `container`, `sport`, `household`, `decoration`, `object`).

### Curation pipeline

The detector applies the following stages in order to the raw classifier output:

1. Take all 1000 softmax scores.
2. **Drop scores below 0.05** — this floors out the long noise tail before any further processing.
3. Look up each surviving raw class in `labels-map.json`.
4. Drop entries whose mapped `label` is `null`.
5. **Drop entries with `category == "person"`** — the three person entries (`baseball player`, `groom`, `diver`) overlap semantically with face recognition and would just add noise to the labels list.
6. De-duplicate by display label, keeping the maximum raw score per display label.
7. Sort by score descending; cap at 20 (see "Namazu inference contract").

The `category` field is **internal** in v1: used only for the person filter above. It is **not** exposed via GraphQL. The field stays in the file as latent metadata for a possible future category-grouped Labels page.

### MobileNetV2 input preprocessing

Both backends must apply the canonical ONNX Model Zoo preprocessing for MobileNetV2 byte-for-byte; deviations cause silent accuracy loss and break cross-backend consistency:

- Decode to RGB.
- Resize so the shorter edge is **256 px**, preserving aspect ratio.
- Center-crop to **224 × 224**.
- Convert to float32 in `[0, 1]`.
- Normalize per-channel: mean `[0.485, 0.456, 0.406]`, std `[0.229, 0.224, 0.225]`.
- Layout: NCHW, shape `[1, 3, 224, 224]`.

## Data Storage

Synthetic data splits cleanly into two stores:

- **Labels and status** live with the asset record in the configured asset backend (CouchDB / PouchDB / SQLite). They're small, scalar, and naturally bound to the asset's lifecycle.
- **Face and person data** live in a **dedicated SQLite face store**, regardless of which backend holds the assets. Binary embeddings, JPEG face crops, high-rev-churn cluster mutations, and the queue all want relational/BLOB semantics that CouchDB/PouchDB serve poorly.

This split means there is exactly one implementation of the face/person/jobs schema — the same SQLite code path is used whether the user runs CouchDB, PouchDB, or SQLite for assets.

### Synthetic data on the asset

The asset record gains a `synthetic` sub-structure (and a `syntheticStatus` field) matching the GraphQL `SyntheticData` type:

```jsonc
{
  // ...existing asset fields...
  "synthetic": {
    "primaryLabel": "beach",
    "labels": ["beach", "palm tree", "sunset"]
    // people are resolved from the face store at query time; not stored here
  },
  "syntheticStatus": "READY"
}
```

For **CouchDB / PouchDB**: a JSON sub-document on the existing asset doc. `couchdb-views.js` gains one new view, `byPrimaryLabel`, used by the Labels page query.

For **SQLite** assets: a new `synthetic_data` table added via the 0003 migration framework, joined to `assets` by `asset_id`:

```
synthetic_data
  asset_id        TEXT PRIMARY KEY REFERENCES assets(id) ON DELETE CASCADE
  primary_label   TEXT
  labels          TEXT  -- JSON array of display labels (post-curation, ordered by score)
  status          TEXT  -- 'PENDING' | 'READY' | 'FAILED' (matches GraphQL enum casing)
  updated_at      INTEGER

INDEX sd_primary_label   ON synthetic_data(primary_label)
```

Note that classification output has no bounding boxes — `synthetic_data` stores only the ranked list of display labels, with `primary_label` denormalized as the top entry.

There is **no** versioning column on `labels`: display labels are stable as long as the curation mapping is stable, and a model swap is handled by re-running `backfillLabels`, which overwrites the row.

### Face store (dedicated SQLite)

A dedicated SQLite database, separate from any asset store the user might be running. Path comes from a new env var `FACE_STORE_PATH` (default: `${dataDir}/faces.db` or similar — the existing data-dir convention applies). Schema:

```
person
  id              TEXT PRIMARY KEY  -- opaque UUID
  name            TEXT              -- user-assigned, nullable
  thumbnail_face  TEXT              -- face_id chosen as representative; nullable
  hidden          INTEGER DEFAULT 0 -- 1 = excluded from People page
  created_at      INTEGER

face
  id              TEXT PRIMARY KEY
  asset_id        TEXT NOT NULL     -- no SQL FK; asset lives in a different store
  person_id       TEXT REFERENCES person(id) ON DELETE SET NULL
  bbox            TEXT NOT NULL      -- JSON [x, y, w, h] in displayed-orientation pixels
  embedding       BLOB NOT NULL      -- raw Float32Array, 512 floats (2048 bytes) for MobileFaceNet
  thumbnail       BLOB NOT NULL      -- ~128px JPEG of the cropped face
  detector_score  REAL
  model_version   TEXT NOT NULL      -- e.g. 'mobilefacenet-v1'

synthetic_jobs
  id              INTEGER PRIMARY KEY
  asset_id        TEXT NOT NULL
  kind            TEXT NOT NULL      -- 'labels' | 'faces'
  priority        INTEGER DEFAULT 0  -- live imports = 10, backfill = 0
  attempts        INTEGER DEFAULT 0
  last_error      TEXT
  enqueued_at     INTEGER

INDEX face_by_person     ON face(person_id)
INDEX face_by_asset      ON face(asset_id)
INDEX face_by_version    ON face(model_version)
INDEX jobs_ready         ON synthetic_jobs(priority DESC, enqueued_at ASC)
```

`face.asset_id` is a plain string id — there is no SQL foreign-key constraint because the asset lives in a different database. Referential integrity is maintained by the use-case layer (see "Cross-store consistency" below).

The `synthetic_jobs` queue lives in the face store rather than the asset store because both labels and faces jobs share the same queue, the workload is high-churn (insert + update + delete per job), and keeping all background-task state in one place simplifies operational reasoning.

#### Vector storage and lookup

Face embeddings are stored as a raw `BLOB` (a Float32Array) on the `face` table. Cosine-similarity lookup is performed in application code: load embeddings for the active `model_version` into memory and score against the query vector. This is fine for personal libraries up to ~100k faces; if we outgrow it, sqlite-vec or an HNSW sidecar can be introduced later without changing the schema.

#### Face crops

Each face row carries its own thumbnail (a JPEG of roughly 128px on the long edge) as a `BLOB`. This keeps the People page rendering to a single query and avoids filesystem coupling. The crop is the aligned face (5-point-landmark warp to the ArcFace 112×112 reference template), not a raw bbox crop.

#### Model versioning

Every face row records the `model_version` that produced its embedding. The matcher only compares embeddings within the same `model_version`. Mixed versions are tolerated in the database (so a partial backfill is safe to pause and resume), but a face from `mobilefacenet-v1` will never match a face from `mobilefacenet-v2`. Upgrading the model is a backfill operation that adds new face rows alongside the old ones; the old rows can be deleted lazily.

#### Person identity

`person.id` is an opaque UUID. The asset's synthetic data references people transitively via `face.person_id`; assets do not store a direct list of people. This means:

- **Rename** is a write to `person.name` — no asset records change.
- **Merge** rewrites `face.person_id` on the source cluster's faces and deletes the now-empty source row (see Cluster lifecycle).
- **Split / reassign** rewrites `face.person_id` on the selected faces.
- When the last face referencing a person row is removed, the person row is **cascade-deleted** (or cleaned up by the same mutation). User-assigned names tied to no faces are not preserved.

### Cross-store consistency

Because face rows reference assets in a different database, the use-case layer (not the database engine) is responsible for keeping the two in sync:

- **Asset deletion**: the asset-deletion use case explicitly calls `faceStore.deleteByAssetId(assetId)` to remove all face rows (and orphaned person rows via the last-face cleanup rule). Replaces the `ON DELETE CASCADE` that we had when face data lived in the asset SQLite DB.
- **Orphan sweep** (defensive): a periodic background task scans `face.asset_id` values and removes any whose asset no longer exists. Belt-and-braces against bugs that bypass the use-case layer; should normally find nothing.
- **Backups**: operators who back up the asset database must also back up `faces.db`. A short note in the deploy docs is enough — the application doesn't try to coordinate this.

## GraphQL

A nullable `synthetic: SyntheticData` field is added to `Asset` and `SearchResult`, mirroring the `metadata: AssetMetadata` field introduced in 0003.

```graphql
type Asset {
  # ...existing fields...
  metadata: AssetMetadata
  synthetic: SyntheticData
  syntheticStatus: SyntheticStatus!
}

type SyntheticData {
  primaryLabel: String
  labels: [String!]!
  people: [Person!]!
}

type Person {
  id: ID!
  name: String
  thumbnail: String!   # URL to /faces/<face_id>/thumb
  hidden: Boolean!
  faceCount: Int!
}

enum SyntheticStatus {
  PENDING
  READY
  FAILED
}
```

Clients learn that synthetic data has landed by observing `syntheticStatus`. The client polls or refetches when status is `PENDING`. No subscription transport is introduced.

New top-level queries:

```graphql
type Query {
  people(includeHidden: Boolean = false): [Person!]!
  labels: [LabelEntry!]!   # distinct primary labels with representative thumbnail + count
  assetsByPerson(id: ID!, offset: Int!, limit: Int!): SearchResults!
  assetsByLabel(label: String!, offset: Int!, limit: Int!): SearchResults!
}
```

New mutations:

```graphql
type Mutation {
  backfillLabels: BackfillResult!
  backfillFaceRecognition: BackfillResult!
  retrySyntheticJobs(kind: String): Int!  # returns count requeued
  renamePerson(id: ID!, name: String): Person!
  mergePeople(sourceId: ID!, targetId: ID!): Person!
  reassignFaces(faceIds: [ID!]!, personId: ID): Person     # null = create new person
  hidePerson(id: ID!, hidden: Boolean!): Person!
  setPersonThumbnail(id: ID!, faceId: ID!): Person!
}
```

### Resolver wiring

A request-scoped Apollo `DataLoader` keyed by asset id resolves `synthetic` and `syntheticStatus`, mirroring the metadata DataLoader from 0003. The loaders compose reads from both stores:

```typescript
// labels + status come from the asset record store
recordRepository.fetchSynthetic(assetIds): Promise<Map<string, { primaryLabel, labels } | null>>
recordRepository.fetchSyntheticStatus(assetIds): Promise<Map<string, SyntheticStatus>>

// people come from the face store
faceStore.fetchPeopleByAssetIds(assetIds): Promise<Map<string, Person[]>>
```

The `synthetic` field resolver merges the two: labels/primaryLabel from the asset store, people from the face store. Two underlying batched fetches per request, regardless of page size. A second DataLoader batches `Person` lookups by id (used when resolving `Person` references that aren't pre-loaded).

### Query repository methods

Mirroring `queryByTags`:

```typescript
// asset record store
recordRepository.queryByLabel(label: string, offset, limit, sort): Promise<SearchResults>

// face store returns asset ids; the use case loads the assets via recordRepository
faceStore.assetIdsByPerson(personId: string, offset, limit, sort): Promise<{ ids: string[], total: number }>
```

For `person:<id>` queries, the use case:
1. Calls `faceStore.assetIdsByPerson(personId, offset, limit, sort)` to get a page of asset ids.
2. Calls `recordRepository.fetchAssets(ids)` (existing batch fetch) to materialize them.

Both `person:<id>` and `label:<class>` integrate into the existing search parser and compose with `tag:`, location, and date filters in the same query. The parser dispatches the `person:` term to the face store and merges asset-id sets when multiple terms are combined.

## Ingestion

`import-asset` writes the asset, then enqueues one row in `synthetic_jobs` per kind (`labels` for Phase 1; both `labels` and `faces` once Phase 2 lands), with `priority = 10`. The HTTP response returns immediately. `syntheticStatus` on the new asset is `PENDING` until a worker completes its jobs.

### Worker pool

A worker pool drains `synthetic_jobs`. Pool size is bounded by `SYNTHETIC_CONCURRENCY` (default `2`). Workers select highest priority first, then oldest enqueued. This means live imports always preempt backfill without separate queue plumbing.

### Failure handling

Each job has up to 3 attempts with exponential backoff. After the third failure, `syntheticStatus = FAILED` is recorded on the asset (kind-aware: an asset can be FAILED for faces but READY for labels, surfaced as the worse of the two). An operator-callable `retrySyntheticJobs(kind: String)` re-enqueues all FAILED jobs (optionally filtered by kind).

## Backfill

Two GraphQL mutations, following 0003's pattern:

- `backfillLabels` — scans assets missing label data and enqueues `kind='labels'` jobs with `priority=0`.
- `backfillFaceRecognition` — scans assets missing face data (or whose face data uses a stale `model_version`) and enqueues `kind='faces'` jobs with `priority=0`.

Both mutations are idempotent and resumable: re-running them does not duplicate work because they enqueue only assets that need it. Partial records are stored — if face detection finds three faces but one face's embedding step fails, the two valid faces are persisted.

## Namazu inference contract

The Namazu blob store has byte-level access to assets; for `NAMAZU_URL` deployments we push inference there rather than streaming bytes back to Tanuki. See the companion spec `0004a-namazu-synthetic-data.md` for the full contract. In summary:

```
POST /synthetic/:blobId
  Response 200 application/json:
    {
      "labels": [ { "name": "beach", "score": 0.91 }, ... ],
      "faces":  [ { "bbox": [x,y,w,h],
                    "embedding": "<base64 little-endian Float32, 512 floats>",
                    "thumbnail": "<base64 JPEG, ~128px>",
                    "score": 0.97,
                    "model_version": "mobilefacenet-v1" }, ... ],
      "model_versions": { "labels": "mobilenetv2-v1", "faces": "mobilefacenet-v1" },
      "truncated": false
    }
  Response 204: not an image
  Response 4xx: extractor failure (4xx body has details)
```

One round-trip per asset. Element-count caps (labels: 20, faces: 20) and a 1 MB total response cap; truncated responses set `"truncated": true`. The `labels[].name` values are **already curated** (i.e. MobileNetV2's raw ImageNet labels have been mapped through `labels-map.json` and `null`-mapped entries dropped) so Tanuki stores them verbatim. Namazu and Tanuki ship with identical copies of `labels-map.json`.

The local blob store's detector implements the same shape so that the use-case-level code is identical regardless of backend.

## Container / DI

`server/container.ts` selects the synthetic-data detector implementation symmetrically with the blob and location repos:

- If `NAMAZU_URL` is set, register `NamazuSyntheticDetector` (HTTP client against the contract above).
- Otherwise, register `LocalSyntheticDetector` (`onnxruntime-node` running MobileNetV2 for labels, SCRFD-2.5g for face detection, and MobileFaceNet for face embedding).

The local detector is a **required dependency**. `onnxruntime-node` is in `dependencies`, the Docker image installs its native requirements, and a failure to load at startup is fatal. This keeps the dependency story predictable: if you can start the server, ML works. All three models load through the same runtime — no separate TF.js stack.

Face crop alignment (5-point landmark warp to the ArcFace reference template) is implemented in TypeScript on the local path; SCRFD returns the landmarks, and a small affine transform plus a JPEG encode via `sharp` produces both the embedding-input crop and the stored thumbnail. The same logic runs in Rust on the Namazu side.

The container also registers a single `FaceStore` implementation (SQLite-backed at `FACE_STORE_PATH`) regardless of which `RecordRepository` was selected for assets. There is no CouchDB or PouchDB variant of the face store — see "Data Storage" for the rationale.

## Presentation

### Thumb list

`thumb-list.tsx` shows the primary label as the first line of `thumb-title` (via `formatTitle()` in `formatting.ts`). When `primaryLabel` is null (asset still pending, no labels produced, or detection failed), `formatTitle()` falls back to the current chain — `location?.label`, then year — so the line is never blank for assets that had a sensible title before.

### People page

A new `People` page lists every non-hidden `person` row with its representative thumbnail and assigned name. The thumbnail is `person.thumbnail_face`'s cropped image; if unset, it defaults to the **largest face by bbox area** in the cluster, breaking ties by `detector_score`. Clicking the name field begins inline editing; clicking the thumbnail navigates to the assets associated with that person (i.e. `assetsByPerson`).

A second view shows all faces in a single cluster and supports multi-select reassignment (`reassignFaces`) for splitting clusters manually. The same view supports `mergePeople` (pick another person to merge into) and `hidePerson` (mark as not-a-person; the cluster is excluded from the People page but rows are preserved).

### Labels page

A new `Labels` page lists every distinct primary label with a representative thumbnail (the most recent asset with that primary label) and count. No editing. Clicking a thumbnail navigates to `assetsByLabel`.

## Cluster lifecycle (summary)

| Operation         | Effect                                                                       |
| ----------------- | ---------------------------------------------------------------------------- |
| Rename            | Update `person.name`. Asset records untouched.                               |
| Merge A → B       | Update all `face.person_id` from A to B. Delete person row A.                |
| Split / reassign  | Update `face.person_id` for selected faces (to new person or existing).      |
| Hide              | Set `person.hidden = 1`. Excluded from People page; face rows preserved.     |
| Last face removed | Cascade-delete the person row (and its assigned name).                       |

There is no alias table; merge is destructive of the source `person.id`. Clients that held the old id will get nothing back for `assetsByPerson(oldId)` and should refetch the People list.

## Privacy

All ML processing is **local** — to Tanuki or to a self-hosted Namazu. No image bytes or embeddings ever leave the user's infrastructure. There is no per-asset opt-out at import; users who do not want biometric data stored at all should not enable Phase 2.

## Resource constraints

The ML solution must **not** require a GPU. MobileNetV2, SCRFD-2.5g, and MobileFaceNet all support CPU-only inference. Combined model footprint is small (~14 MB labels + ~8 MB faces ≈ 22 MB total). The default `SYNTHETIC_CONCURRENCY = 2` is intended to keep a modest server responsive during ingestion; operators with more cores can raise it.

## Model file management

Model weights are not checked into the repo and are not assumed to be supplied via Docker volumes or bind mounts. Instead, the repo carries a small **manifest** that lists the canonical models, and a fetch script downloads them on demand into a gitignored `models/` directory at the project root.

### Manifest

`model-manifest.json` at the project root, checked into git:

```json
{
  "version": "models-v1",
  "files": [
    {
      "name": "mobilenet_v2.onnx",
      "url": "https://github.com/<owner>/tanuki/releases/download/models-v1/mobilenet_v2.onnx",
      "sha256": "<hex>",
      "bytes": 14000000
    },
    {
      "name": "scrfd_2.5g.onnx",
      "url": "https://github.com/<owner>/tanuki/releases/download/models-v1/scrfd_2.5g.onnx",
      "sha256": "<hex>",
      "bytes": 3200000
    },
    {
      "name": "mobilefacenet.onnx",
      "url": "https://github.com/<owner>/tanuki/releases/download/models-v1/mobilefacenet.onnx",
      "sha256": "<hex>",
      "bytes": 5100000
    },
    {
      "name": "labels-map.json",
      "url": "https://github.com/<owner>/tanuki/releases/download/models-v1/labels-map.json",
      "sha256": "<hex>",
      "bytes": 80000
    }
  ]
}
```

The Tanuki repo is the **canonical source** for all four files. Models are uploaded as assets on a GitHub Release tagged with the manifest's `version` field (e.g. `models-v1`). When models change, cut a new release (`models-v2`), update the manifest's URLs / hashes / `version`, and update Namazu's manifest copy in the same coordinated change.

### Fetch script

`scripts/fetch-models.ts` (Bun, ~80 lines, no extra dependencies):

1. Read `model-manifest.json`.
2. For each entry, check `models/<name>` on disk:
   - If present and its SHA256 matches the manifest, skip.
   - Otherwise, fetch to `models/<name>.tmp`, verify SHA256, then atomic-rename to `models/<name>`.
3. Fail with a clear error if any download fails its hash check (catches both corruption and accidental drift between Tanuki and Namazu manifests).

The script uses Bun's built-in `fetch`, `Bun.write`, `Bun.file`, and `crypto.createHash` — no `npm install` required.

### Lifecycle wiring

In `package.json`:

```json
{
  "scripts": {
    "fetch-models": "bun run scripts/fetch-models.ts",
    "prestart": "bun run fetch-models",
    "pretest": "bun run fetch-models",
    "prebuild": "bun run fetch-models"
  }
}
```

`bun start`, `bun test`, and `bun run build` each ensure models are present before doing anything else. After the first run, subsequent invocations are a no-op (hash check passes). The hooks are intentionally not `postinstall` — that would slow `bun install` and trigger downloads in contexts (CI for unrelated changes, fresh `node_modules`) where they aren't needed.

### `.gitignore`

```
/models/
```

### CI caching

GitHub Actions (and equivalents) should cache `models/` keyed by the manifest's hash so the download happens once per manifest version across all CI runs:

```yaml
- uses: actions/cache@v4
  with:
    path: models
    key: tanuki-models-${{ hashFiles('model-manifest.json') }}
```

### Producing release assets

The runtime `scripts/fetch-models.ts` script consumes a GitHub Release. A second script, `scripts/build-release-assets.ts`, **produces** that release by pulling each model from its upstream source, verifying the bytes, and dropping them into a `release/` directory ready to upload.

Outline (~50 lines, no extra deps beyond Bun's built-ins):

```typescript
// scripts/build-release-assets.ts
import { createHash } from "node:crypto";
import { mkdir } from "node:fs/promises";

interface Source {
  name: string;       // filename in release (matches model-manifest.json)
  url: string;        // upstream canonical URL
  sha256?: string;    // optional; if set, must match; if absent, script prints actual hash
}

const SOURCES: Source[] = [
  // MobileNetV2 — official ONNX Model Zoo
  { name: "mobilenet_v2.onnx",    url: "https://github.com/onnx/models/raw/main/.../mobilenetv2-12.onnx" },
  // SCRFD-2.5g — InsightFace, via a stable HuggingFace mirror (e.g. immich-app or deepghs)
  { name: "scrfd_2.5g.onnx",      url: "https://huggingface.co/<org>/<repo>/resolve/main/scrfd_2.5g_bnkps.onnx" },
  // MobileFaceNet (ArcFace-trained), same mirror
  { name: "mobilefacenet.onnx",   url: "https://huggingface.co/<org>/<repo>/resolve/main/w600k_mbf.onnx" },
];

const OUT = "release";
await mkdir(OUT, { recursive: true });

for (const src of SOURCES) {
  console.log(`Fetching ${src.name} from ${src.url}`);
  const res = await fetch(src.url);
  if (!res.ok) throw new Error(`${src.name}: HTTP ${res.status}`);
  const bytes = new Uint8Array(await res.arrayBuffer());
  const sha256 = createHash("sha256").update(bytes).digest("hex");

  if (src.sha256 && src.sha256 !== sha256) {
    throw new Error(`${src.name}: hash mismatch (got ${sha256}, expected ${src.sha256})`);
  }

  await Bun.write(`${OUT}/${src.name}`, bytes);
  console.log(`  ${bytes.length.toLocaleString()} bytes, sha256=${sha256}`);
}
```

`labels-map.json` is hand-curated in this repo (not downloaded) — copy it into `release/` as a separate step, or extend the script with a "local file" source type.

Workflow when bumping models:

1. Edit `scripts/build-release-assets.ts` if any upstream URL changed; otherwise just re-run it.
2. `bun run scripts/build-release-assets.ts` — produces `release/*` and prints each file's SHA256.
3. Copy the curated `labels-map.json` into `release/`.
4. `gh release create models-vN release/* --title "Models vN"` to publish.
5. Update `model-manifest.json` in **both** Tanuki and Namazu with the new `version` and the printed SHA256s. Commit those changes in lockstep with the release.
6. `bun run fetch-models` locally to confirm the new release works end-to-end.

This keeps the production loop tight: one script to fetch upstream, one `gh release` invocation, one matched-pair manifest edit.

#### Notes on each upstream

- **MobileNetV2** — ONNX Model Zoo publishes canonical builds; pin to a specific opset/version filename (e.g. `mobilenetv2-12.onnx`). Verify once that the output class indices match the standard ImageNet-1000 ordering used by `labels-map.json`.
- **SCRFD-2.5g** and **MobileFaceNet** — InsightFace publishes these as part of their `buffalo_s` / `buffalo_sc` model bundles. Several HuggingFace orgs mirror them with stable filenames (look for `scrfd_2.5g_bnkps.onnx` and `w600k_mbf.onnx`). Pick a mirror, pin the SHA, and you're locked.

If a future upstream goes away, the GitHub Release on the Tanuki repo remains an independent source of truth — the build script is for **producing** the release, not a runtime dependency.

### Lockstep with Namazu

Namazu carries a parallel `model-manifest.json` in its repo, with the same `version` field and the same hashes. Namazu's `build.rs` performs the equivalent download-and-verify into its workspace `models/` directory (see `0004a-namazu-synthetic-data.md`). The SHA256 checks on both sides will fail loudly if the manifests drift, which is the early-warning signal we rely on instead of any runtime cross-check.

### Optional cache override

A `TANUKI_MODEL_CACHE` env var may later be added to point the fetch script at a shared external directory (useful if multiple project checkouts on the same machine want to share the ~35 MB of weights). Not implemented in the initial cut.

## Docker

The `onnxruntime-node` package needs native dependencies (`libc6`, `libstdc++`, and on some bases a recent GLIBC). The Dockerfile must:

- install the system packages required by `onnxruntime-node` for the target base image
- run `bun run fetch-models` during the image build so the model files are baked into the image (see "Model file management"). The `prebuild` lifecycle hook makes this automatic if the image build invokes `bun run build`; otherwise add an explicit `RUN bun run fetch-models` step before the application is copied in.

The existing `ffmpeg` addition from 0003 stays.

## Candidate libraries

### JavaScript (local detector)

- [onnxruntime-node](https://www.npmjs.com/package/onnxruntime-node) running **MobileNetV2** (label classification), **SCRFD-2.5g** (face detection), and **MobileFaceNet** (face embedding). One runtime for all three models. The ONNX files are the same on the Namazu side, so label and face outputs are comparable across backends.
- [sharp](https://www.npmjs.com/package/sharp) for image preprocessing (resize + normalize for MobileNetV2 input) and face crop generation (affine warp from SCRFD landmarks + JPEG encode).

### Rust (Namazu detector)

Image classification:

- [tract](https://crates.io/crates/tract) — pure-Rust ONNX inference, runs MobileNetV2 CPU-only without native ML deps. Recommended for the classification path.
- [candle-core](https://crates.io/crates/candle-core) — Hugging Face's Rust ML framework; viable alternative.

Face detection + embedding:

- [tract](https://crates.io/crates/tract) or [ort](https://crates.io/crates/ort) running **SCRFD-2.5g** + **MobileFaceNet** ONNX models. Same files as the Tanuki local path.
- [face_id](https://crates.io/crates/face_id) may be usable as a higher-level wrapper if it accepts custom ONNX model paths; otherwise drop down to `tract`/`ort` directly.

General:

- [image](https://crates.io/crates/image) for decoding inputs and producing the face crops.

## Tests

New tests for:

- **Label classification golden fixtures** — known images produce expected primary label + ordered list within score tolerance.
- **Label curation** — given a fixed `labels-map.json`, raw ImageNet outputs map to the expected display labels, and `null`-mapped raw labels are dropped.
- **Face detection golden fixtures** — known images produce expected face count + bbox shapes.
- **Clustering determinism** — given the same sequence of embeddings and a fixed threshold, the same cluster assignments emerge.
- **Asset-store repository methods, all three backends** (CouchDB, PouchDB, SQLite) — `fetchSynthetic`, `fetchSyntheticStatus`, `queryByLabel`.
- **Face store repository methods** (SQLite only) — `fetchPeopleByAssetIds`, `assetIdsByPerson`, embedding scan + cosine search at the active `model_version`, cluster lifecycle operations.
- **Cross-store consistency** — asset deletion removes face rows via the use-case path; orphan-sweep removes face rows whose `asset_id` no longer exists in the asset store.
- **DataLoader batching** — a single SearchResults page (up to 72 assets) triggers one `fetchSynthetic`, one `fetchSyntheticStatus`, and one `fetchPeopleByAssetIds` call across the two stores combined.
- **Backfill use cases** — partial-record persistence, idempotent re-runs, model-version-aware re-enqueue.
- **Cluster lifecycle mutations** — merge updates `face.person_id` and deletes source person row; reassign updates `face.person_id`; hide flips the flag; last-face cleanup cascades.
- **formatTitle fallback** — null primary label falls back through the existing chain.
- **Search parser** — `person:<id>` and `label:<class>` parse and compose with `tag:` and other terms.
- **Worker pool** — priority ordering, retry-then-fail behavior, retry mutation re-enqueues only FAILED jobs.

## Open items

- Tuning knobs (cosine similarity threshold for clustering, face crop size, retry backoff) will be surfaced during implementation. Starting points: cosine threshold ≈ 0.5 (MobileFaceNet on aligned crops is well-characterized at this range), face crop ~128 px JPEG quality 85, retry backoff 1s/4s/16s. Configurable only if real usage shows they need to be.
