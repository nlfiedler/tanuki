# ML Models

## Overview

The image classification and face recognition features of tanuki rely on
pre-trained ML models. This project owns the fetch and release process which
enables the sister project, namazu, to easily retrieve the identical files in
order to produce matching results.

## Labels Mapping

See the mapping in `server/data/synthetic/labels-map.json` that maps NASNET
labels to more generic labels suitable for casual use. This mapping file is part
of the released files and is utilized by namazu.

## Fetching and Releasing

Verify the upstream URLs before first run. The Immich HuggingFace mirror for
`buffalo_s/detection.onnx` and `buffalo_s/recognition.onnx` has been stable; the
ONNX Model Zoo MobileNetV2 path may have moved — open the URL in a browser to
confirm before running.

Workflow to publish `models-v1`:

```shell
bun run build-release-assets       # downloads + copies labels-map.json + prints SHAs
# paste the 4 SHA256 values + actual byte counts into model-manifest.json
gh release create models-v1 release/* --title "Models v1" --notes "..."
git add model-manifest.json && git commit -m "chore: pin models-v1 sha256"
bun run fetch-models               # smoke-test the release end-to-end
```

After step 4 succeeds you will have `models/mobilenet_v2.onnx`,
`models/scrfd_2.5g.onnx`, and `models/mobilefacenet.onnx` (labels-map is skipped per
the spec), and namazu can mirror the same `model-manifest.json` for its `build.rs`.
