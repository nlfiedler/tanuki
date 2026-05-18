//
// Copyright (c) 2026 Nathan Fiedler
//
import { Match, Switch } from 'solid-js';
import type { SearchResult } from 'tanuki/generated/graphql.ts';

interface GalleryMediaProps {
  asset: SearchResult;
  imageClass: string;
  // Layouts that pack tiles by aspect (justified rows) provide a callback to
  // receive the preview's natural aspect ratio once it loads; layouts that
  // don't (masonry) omit it.
  onAspect?: (aspect: number) => void;
  // Override the image source for image assets (e.g. a smaller sized preview
  // for fixed-size layouts). Video assets always use thumbnailUrl regardless.
  imageSrc?: string;
}

function GalleryMedia(props: GalleryMediaProps) {
  const mediaType = () => props.asset.mediaType;
  const isImage = () => mediaType().startsWith('image/');
  const isVideo = () => mediaType().startsWith('video/');
  const isAudio = () => mediaType().startsWith('audio/');

  return (
    <Switch fallback={<FilePlaceholder asset={props.asset} />}>
      <Match when={isImage()}>
        <PreviewImage
          asset={props.asset}
          imageClass={props.imageClass}
          onAspect={props.onAspect}
          src={props.imageSrc}
        />
      </Match>
      <Match when={isVideo()}>
        <>
          <PreviewImage
            asset={props.asset}
            imageClass={props.imageClass}
            onAspect={props.onAspect}
            src={props.asset.thumbnailUrl}
          />
          <span class="gallery-play-overlay" aria-hidden="true">
            <i class="fa-solid fa-circle-play"></i>
          </span>
        </>
      </Match>
      <Match when={isAudio()}>
        <AudioPlaceholder asset={props.asset} />
      </Match>
    </Switch>
  );
}

interface PreviewImageProps {
  asset: SearchResult;
  imageClass: string;
  onAspect?: (aspect: number) => void;
  // Override the image source; defaults to the asset's preview URL. Used for
  // videos, where the thumbnail route extracts a frame but the preview route
  // (on the namazu backend) does not.
  src?: string;
}

function PreviewImage(props: PreviewImageProps) {
  // Intrinsic dimensions from extracted metadata let the browser reserve the
  // correct aspect ratio before the image loads (preventing layout shift). CSS
  // still controls the rendered size; width/height here only fix the ratio.
  const intrinsicWidth = () => props.asset.metadata?.displayWidth ?? undefined;
  const intrinsicHeight = () => props.asset.metadata?.displayHeight ?? undefined;
  return (
    <img
      class={props.imageClass}
      src={props.src ?? props.asset.previewUrl}
      alt={props.asset.filename}
      loading="lazy"
      width={intrinsicWidth()}
      height={intrinsicHeight()}
      onLoad={(e) => {
        if (!props.onAspect) return;
        const img = e.currentTarget;
        if (img.naturalHeight > 0) {
          props.onAspect(img.naturalWidth / img.naturalHeight);
        }
      }}
    />
  );
}

function AudioPlaceholder(props: { asset: SearchResult }) {
  return (
    <span class="gallery-audio-placeholder">
      <i class="fa-solid fa-music" aria-hidden="true"></i>
      <span class="gallery-audio-filename">{props.asset.filename}</span>
    </span>
  );
}

// Fallback for media types we can't render a preview for (e.g. PDF) — shows
// a neutral icon and the filename instead of a broken image.
function FilePlaceholder(props: { asset: SearchResult }) {
  return (
    <span class="gallery-audio-placeholder">
      <i class="fa-solid fa-file" aria-hidden="true"></i>
      <span class="gallery-audio-filename">{props.asset.filename}</span>
    </span>
  );
}

export default GalleryMedia;
