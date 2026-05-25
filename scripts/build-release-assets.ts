#!/usr/bin/env bun
/**
 * Build the release assets for the synthetic-data ML models.
 *
 * Fetches the three ONNX files from their upstream mirrors, verifies (or
 * prints) their SHA256 hashes, copies the in-repo labels-map.json into
 * place, and drops everything into ./release ready for:
 *
 *   gh release create models-vN release/* --title "Models vN"
 *
 * After uploading, paste the printed SHA256 values into model-manifest.json
 * (and into Namazu's parallel manifest) and commit. The runtime
 * scripts/fetch-models.ts script will then verify against those hashes.
 *
 * This script is run rarely — only when bumping model versions. The URLs
 * below should be sanity-checked in a browser before each run; if an
 * upstream moves, update the URL here.
 */

import { createHash } from "node:crypto";
import { copyFile, mkdir, rm } from "node:fs/promises";
import path from "node:path";

interface Source {
  /** Filename in the release (must match model-manifest.json `name`). */
  name: string;
  /** Upstream URL to fetch from. */
  url: string;
  /** Optional expected SHA256 (hex). If absent, script prints the actual hash. */
  sha256?: string;
}

// VERIFY: confirm these URLs resolve before each release. Upstream mirrors
// occasionally restructure. If you change an upstream, also rev the manifest
// version (`models-vN`) since the bytes may differ from the previous release.
const SOURCES: Source[] = [
  {
    // MobileNetV2 from the ONNX Model Zoo (opset 12 / "validated" path).
    // Roughly 14 MB. Pre-trained on ImageNet-1000, uses the canonical
    // mean/std preprocessing recipe documented in the spec.
    name: "mobilenet_v2.onnx",
    url: "https://github.com/onnx/models/raw/main/validated/vision/classification/mobilenet/model/mobilenetv2-12.onnx",
  },
  {
    // SCRFD-2.5g face detector from InsightFace, mirrored by Immich. ~3 MB.
    // The buffalo_s bundle's detection model — outputs bboxes + 5 landmarks.
    name: "scrfd_2.5g.onnx",
    url: "https://huggingface.co/immich-app/buffalo_s/resolve/main/detection/model.onnx",
  },
  {
    // MobileFaceNet (ArcFace-trained, w600k_mbf), same Immich mirror. ~5 MB.
    // 128-dimensional L2-normalized embeddings from aligned 112x112 crops.
    name: "mobilefacenet.onnx",
    url: "https://huggingface.co/immich-app/buffalo_s/resolve/main/recognition/model.onnx",
  },
];

const LABELS_MAP_SRC = "server/data/synthetic/labels-map.json";
const OUT_DIR = "release";

async function hashFile(bytes: Uint8Array): Promise<string> {
  return createHash("sha256").update(bytes).digest("hex");
}

async function fetchOne(src: Source): Promise<void> {
  process.stdout.write(`Fetching ${src.name}\n  from ${src.url} ... `);

  const res = await fetch(src.url);
  if (!res.ok) {
    throw new Error(`HTTP ${res.status} ${res.statusText}`);
  }

  const bytes = new Uint8Array(await res.arrayBuffer());
  const sha256 = await hashFile(bytes);

  if (src.sha256 && src.sha256 !== sha256) {
    throw new Error(
      `${src.name}: hash mismatch\n  expected: ${src.sha256}\n  actual:   ${sha256}`
    );
  }

  await Bun.write(path.join(OUT_DIR, src.name), bytes);
  console.log(
    `OK\n  ${bytes.length.toLocaleString()} bytes, sha256=${sha256}`
  );
}

async function copyLabelsMap(): Promise<void> {
  const dest = path.join(OUT_DIR, "labels-map.json");
  await copyFile(LABELS_MAP_SRC, dest);
  const bytes = new Uint8Array(await Bun.file(dest).arrayBuffer());
  const sha256 = await hashFile(bytes);
  console.log(
    `Copied ${LABELS_MAP_SRC} -> ${dest}\n  ${bytes.length.toLocaleString()} bytes, sha256=${sha256}`
  );
}

async function main(): Promise<void> {
  // Start with a clean release/ so stale files from a previous run don't sneak in.
  await rm(OUT_DIR, { recursive: true, force: true });
  await mkdir(OUT_DIR, { recursive: true });

  for (const src of SOURCES) {
    await fetchOne(src);
  }
  await copyLabelsMap();

  console.log(`\nDone. Files written to ./${OUT_DIR}/`);
  console.log("\nNext steps:");
  console.log(
    "  1. Paste the printed sha256 values into model-manifest.json (and Namazu's manifest)."
  );
  console.log(
    `  2. gh release create models-vN ${OUT_DIR}/* --title "Models vN"`
  );
  console.log(
    "  3. Bump `version` in both manifests to match the release tag and commit."
  );
}

await main();
