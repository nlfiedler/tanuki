//
// Copyright (c) 2026 Nathan Fiedler
//
import sharp from 'sharp';

/** Target square crop fed to the network. */
export const CROP = 224;
/** Shorter-edge resize target before center-cropping. */
const RESIZE = 256;
/** Per-channel ImageNet normalization (RGB). */
const MEAN = [0.485, 0.456, 0.406];
const STD = [0.229, 0.224, 0.225];

/**
 * Preprocess an encoded image into the exact tensor MobileNetV2 expects, per
 * the ONNX Model Zoo recipe (deviating breaks cross-backend consistency):
 * decode to RGB, resize the shorter edge to 256 preserving aspect, center-crop
 * 224×224, scale to `[0,1]`, normalize per channel, and lay out as NCHW.
 *
 * @param image - encoded image bytes (JPEG/PNG/HEIC/...).
 * @returns a Float32Array of length `3*224*224` (shape `[1,3,224,224]`).
 */
export async function preprocessImage(
  image: Buffer | Uint8Array
): Promise<Float32Array> {
  // Resize so the shorter edge is 256 (fit: 'outside' covers the box), drop any
  // alpha, and force sRGB so channel order and range are predictable.
  const resized = await sharp(image)
    .removeAlpha()
    .toColourspace('srgb')
    .resize(RESIZE, RESIZE, { fit: 'outside' })
    .toBuffer();

  const meta = await sharp(resized).metadata();
  const left = Math.max(0, Math.floor(((meta.width ?? RESIZE) - CROP) / 2));
  const top = Math.max(0, Math.floor(((meta.height ?? RESIZE) - CROP) / 2));

  const { data } = await sharp(resized)
    .extract({ left, top, width: CROP, height: CROP })
    .raw()
    .toBuffer({ resolveWithObject: true });

  // data is interleaved RGB uint8 (length CROP*CROP*3); emit planar NCHW float.
  const plane = CROP * CROP;
  const tensor = new Float32Array(3 * plane);
  for (let i = 0; i < plane; i++) {
    tensor[i] = (data[i * 3]! / 255 - MEAN[0]!) / STD[0]!;
    tensor[plane + i] = (data[i * 3 + 1]! / 255 - MEAN[1]!) / STD[1]!;
    tensor[2 * plane + i] = (data[i * 3 + 2]! / 255 - MEAN[2]!) / STD[2]!;
  }
  return tensor;
}
