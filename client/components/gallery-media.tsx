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
        />
      </Match>
      <Match when={isVideo()}>
        <>
          <PreviewImage
            asset={props.asset}
            imageClass={props.imageClass}
            onAspect={props.onAspect}
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
}

function PreviewImage(props: PreviewImageProps) {
  return (
    <img
      class={props.imageClass}
      src={props.asset.previewUrl}
      alt={props.asset.filename}
      loading="lazy"
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
