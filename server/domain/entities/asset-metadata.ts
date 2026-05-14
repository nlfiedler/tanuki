//
// Copyright (c) 2026 Nathan Fiedler
//

/**
 * AssetMetadata captures information extracted from the asset file itself
 * (EXIF tags for images, ffprobe output for videos). All fields are optional
 * because real-world files vary widely in what they record.
 *
 * The `raw` field stores the unparsed extractor output so future additions to
 * this entity can be backfilled by re-parsing already-fetched JSON rather than
 * re-reading every blob.
 */
class AssetMetadata {
  /** Camera manufacturer (EXIF `Make`). */
  cameraMake: string | null = null;
  /** Camera model (EXIF `Model`). */
  cameraModel: string | null = null;
  /** Lens manufacturer (EXIF `LensMake`). */
  lensMake: string | null = null;
  /** Lens model (EXIF `LensModel`). */
  lensModel: string | null = null;
  /** Exposure time, formatted like `"1/250"` (EXIF `ExposureTime`). */
  exposureTime: string | null = null;
  /** Aperture f-stop number (EXIF `FNumber`). */
  fNumber: number | null = null;
  /** ISO speed rating (EXIF `ISOSpeedRatings`). */
  iso: number | null = null;
  /** Focal length in 35mm equivalent millimeters. */
  focalLength35mm: number | null = null;
  /** Timezone offset for the original date-time, e.g. `"+09:00"`. */
  originalDateOffset: string | null = null;
  /** GPS latitude in decimal degrees, negative for south. */
  gpsLatitude: number | null = null;
  /** GPS longitude in decimal degrees, negative for west. */
  gpsLongitude: number | null = null;
  /** Width in pixels in the displayed orientation. */
  displayWidth: number | null = null;
  /** Height in pixels in the displayed orientation. */
  displayHeight: number | null = null;
  /** Video duration in seconds. */
  duration: number | null = null;
  /** Video frame rate in frames per second. */
  frameRate: number | null = null;
  /** Video codec name (ffprobe `codec_name`). */
  videoCodec: string | null = null;
  /** Raw extractor output (EXIF tag map or ffprobe JSON). Not exposed in GraphQL. */
  raw: object | null = null;

  /** Returns true if any field carries a non-null value. */
  hasValues(): boolean {
    return (
      this.cameraMake !== null ||
      this.cameraModel !== null ||
      this.lensMake !== null ||
      this.lensModel !== null ||
      this.exposureTime !== null ||
      this.fNumber !== null ||
      this.iso !== null ||
      this.focalLength35mm !== null ||
      this.originalDateOffset !== null ||
      this.gpsLatitude !== null ||
      this.gpsLongitude !== null ||
      this.displayWidth !== null ||
      this.displayHeight !== null ||
      this.duration !== null ||
      this.frameRate !== null ||
      this.videoCodec !== null
    );
  }
}

export { AssetMetadata };
