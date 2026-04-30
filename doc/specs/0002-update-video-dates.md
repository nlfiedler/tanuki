# Backfill Video Dates

## Original Prompt

Now that we have mp4box added to the project, and the ability to get the
creation time of an mp4/mov video, I would like to fix all of the existing
records in the database. That is, for any Asset entity whose originalDate is
null, and it's mediaType starts with "video/", we should get a sufficient number
of bytes from the file to use mp4box to get the creation time, then update the
originalDate property and update the database record. For this, several things
must happen: add a use case like scan-assets.ts that iterates over the Assets
from the record repository in chunks; for each Asset that matches the criteria
stated above, use the blob repository to fetch enough content from the video
file to get the creation time. This will require duplicating parts of
getCreationTime() from helpers.ts, since we will be dealing with raw bytes
rather than a file path. Another necessary change to support this use case is
for the blob repository to have a new function that retrieves a range of bytes
from the asset. The local-blob-repository.ts can simply use file system
operations to read from the local file, but namazu-blob-repository.ts will have
to use an HTTP GET request with a Content-Range header to retrieve the desired
raw data from the remote blob server. The namazu blob repository supports
Content-Range so this should be reasonably efficient. Also, this "update all
videos so they have an original date value" is only going to be run once, from a
GraphQL mutation that we will add shortly, so efficiency is not a concern, it
just has to work. please use plan mode to work out the details for this request.

---

## Strategy

### Goal

Walk every asset in the database, find videos whose `originalDate` is null,
read just enough of each blob to parse the mp4/mov creation time via mp4box,
and persist the updated record. Triggered once via a GraphQL mutation.

### Design

#### 1. Byte-range read primitive on `BlobRepository`

Add a new method to the repository interface so that the use case can read
partial blob content without caring whether the blob is local or remote:

```ts
fetchRange(assetId: string, start: number, end: number): Promise<Buffer>;
```

Returns however many bytes are actually available within the inclusive
`[start, end]` range; an empty buffer means EOF.

- **Local blob repository**: opens the file with `fs.open`, calls
  `handle.read` into a `Uint8Array` at the requested offset, returns a
  `Buffer` view of however many bytes were actually read.
- **Namazu blob repository**: HTTP `GET` with header `Range: bytes=start-end`.
  - `206 Partial Content` is the expected response — return the body as-is.
  - `200 OK` (server ignored the `Range` header, e.g. for tiny blobs) — slice
    the body down to the requested range so the caller's offset bookkeeping
    stays correct.
  - `416 Range Not Satisfiable` — return an empty buffer (past EOF).
  - Any other status throws.

#### 2. Buffer-based mp4box parser in `helpers.ts`

The existing `getCreationTime(filepath)` streams a local file into mp4box.
We add a sibling that drives the same parser from a caller-supplied byte
fetcher:

```ts
getCreationTimeFromBlob(
  fetcher: (start: number, end: number) => Promise<Buffer>
): Promise<number | null>
```

It reads 64 KiB chunks in a loop, wraps each chunk with `MP4BoxBuffer.fromArrayBuffer(buf, offset)`, and feeds it via `appendBuffer`. The loop
exits when:

- mp4box's `onReady` fires — capture `info.created.getTime()` and return.
- mp4box's `onError` fires, or `appendBuffer` throws — return null.
- The fetcher returns an empty buffer (EOF) — flush mp4box and return
  whatever (if anything) `onReady` produced, otherwise null.

The original file-path `getCreationTime` is left as-is; the import path that
already uses it keeps working.

#### 3. Use case: `update-video-dates.ts`

Modeled on `scan-assets.ts`. Iterates `recordRepository.fetchAssets(cursor, 1024)` until exhausted. For each asset:

1. Skip if `mediaType` does not start with `video/`.
2. Skip if `originalDate` is already set.
3. Call `getCreationTimeFromBlob` with a fetcher closed over
   `blobRepository.fetchRange(asset.key, ...)`.
4. If a millisecond timestamp comes back, `asset.setOriginalDate(new Date(ms))` and `recordRepository.putAsset(asset)`; increment the updated counter.
5. Swallow per-asset errors (missing/unreadable blob is non-fatal; just
   move on).

Returns the total number of records updated.

#### 4. Wiring

- Register `updateVideoDates: asFunction(UpdateVideoDates)` in `server/container.ts`.
- Add to `schema.graphql`:

  ```graphql
  updateVideoDates: Int!
  ```

- Add a resolver in `schema.ts` that resolves `updateVideoDates` from the
  container and returns its result. Pattern matches the existing `import` mutation.
- Run `bun run codegen` to regenerate the GraphQL TypeScript types.

#### 5. Test mock

`test/domain/usecases/mocking.ts` builds a mock `BlobRepository` literal,
which would otherwise fail the new interface contract. Extended the mock
factory with a `fetchRange` slot defaulting to a no-op that returns an empty
buffer.

### Files changed

| File | Change |
|---|---|
| `server/domain/repositories/blob-repository.ts` | added `fetchRange` to the interface |
| `server/data/repositories/local-blob-repository.ts` | implemented `fetchRange` via `fs.open`/`read` |
| `server/data/repositories/namazu-blob-repository.ts` | implemented `fetchRange` via `fetch` + `Range` |
| `server/domain/usecases/helpers.ts` | added `getCreationTimeFromBlob` |
| `server/domain/usecases/update-video-dates.ts` | new use case |
| `server/container.ts` | registered `updateVideoDates` |
| `server/preso/graphql/schema.graphql` | added `updateVideoDates` mutation |
| `server/preso/graphql/schema.ts` | added resolver |
| `test/domain/usecases/mocking.ts` | extended `blobRepositoryMock` with `fetchRange` |

### Invocation

```graphql
mutation { updateVideoDates }
```

Returns the number of asset records whose `originalDate` was filled in.

### Verification

- `bun run codegen` — regenerates GraphQL types.
- `bunx tsc --noEmit -p .` — clean.
- `bun test` — 209/209 passing.
