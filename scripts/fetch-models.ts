#!/usr/bin/env bun
/**
 * Fetch ML model files listed in model-manifest.json into ./models.
 *
 * Idempotent: skips any file whose on-disk SHA256 already matches the manifest.
 * Wired into the `prestart`, `pretest`, and `prebuild` package.json hooks so
 * that running `bun start`, `bun test`, or `bun run build` ensures models are
 * present before doing anything else.
 *
 * The labels-map.json entry is intentionally skipped here: the canonical copy
 * lives in the repo at server/data/synthetic/labels-map.json and that is what
 * the local detector reads. The manifest still lists it so Namazu's parallel
 * tooling can fetch it from the same release.
 */

import { createHash } from "node:crypto";
import { mkdir, rename } from "node:fs/promises";
import path from "node:path";

interface ManifestEntry {
  name: string;
  url: string;
  sha256: string;
  bytes?: number;
}

interface Manifest {
  version: string;
  files: ManifestEntry[];
}

const MANIFEST_PATH = "model-manifest.json";
const MODELS_DIR = "models";

/** Files listed in the manifest that this runtime does not need locally. */
const SKIP = new Set<string>(["labels-map.json"]);

async function readManifest(): Promise<Manifest> {
  const text = await Bun.file(MANIFEST_PATH).text();
  return JSON.parse(text);
}

async function sha256Of(path: string): Promise<string | null> {
  try {
    const bytes = new Uint8Array(await Bun.file(path).arrayBuffer());
    return createHash("sha256").update(bytes).digest("hex");
  } catch {
    return null;
  }
}

async function fetchEntry(entry: ManifestEntry): Promise<void> {
  const finalPath = path.join(MODELS_DIR, entry.name);
  const tmpPath = `${finalPath}.tmp`;

  // Hash placeholder in the manifest means the release hasn't been cut yet.
  if (!/^[0-9a-f]{64}$/i.test(entry.sha256)) {
    throw new Error(
      `${entry.name}: manifest sha256 is not a 64-char hex string ` +
        `(got "${entry.sha256}"). Has the release been uploaded and the ` +
        `manifest populated?`
    );
  }

  // Skip if file exists and matches manifest hash.
  const existing = await sha256Of(finalPath);
  if (existing === entry.sha256) {
    console.log(`  ${entry.name}: up to date`);
    return;
  }
  if (existing) {
    console.log(
      `  ${entry.name}: hash mismatch on disk, refetching ` +
        `(expected ${entry.sha256.slice(0, 12)}…, got ${existing.slice(0, 12)}…)`
    );
  }

  process.stdout.write(`  ${entry.name}: fetching from ${entry.url} ... `);
  const res = await fetch(entry.url);
  if (!res.ok) {
    throw new Error(`${entry.name}: HTTP ${res.status} ${res.statusText}`);
  }
  const bytes = new Uint8Array(await res.arrayBuffer());
  const actual = createHash("sha256").update(bytes).digest("hex");

  if (actual !== entry.sha256) {
    throw new Error(
      `${entry.name}: downloaded bytes do not match manifest hash\n` +
        `  expected: ${entry.sha256}\n  actual:   ${actual}`
    );
  }

  await Bun.write(tmpPath, bytes);
  await rename(tmpPath, finalPath);
  console.log(`OK (${bytes.length.toLocaleString()} bytes)`);
}

async function main(): Promise<void> {
  const manifest = await readManifest();
  console.log(`Fetching models (manifest version: ${manifest.version})`);

  await mkdir(MODELS_DIR, { recursive: true });

  for (const entry of manifest.files) {
    if (SKIP.has(entry.name)) {
      console.log(`  ${entry.name}: skipped (read from repo, not models/)`);
      continue;
    }
    await fetchEntry(entry);
  }

  console.log("Done.");
}

await main();
