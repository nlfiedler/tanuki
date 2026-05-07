//
// Copyright (c) 2026 Nathan Fiedler
//
import { createSignal, For, Match, Switch } from 'solid-js';
import type { SearchResult } from 'tanuki/generated/graphql.ts';
import * as format from '../helpers/formatting.ts';

interface JustifiedRowsProps {
  results?: SearchResult[];
  onClick: (assetId: string, index: number) => void;
}

function JustifiedRows(props: JustifiedRowsProps) {
  return (
    <div class="gallery">
      <For each={props.results}>
        {(asset, index) => (
          <Tile
            asset={asset}
            onClick={() => props.onClick(asset.assetId, index())}
          />
        )}
      </For>
    </div>
  );
}

interface TileProps {
  asset: SearchResult;
  onClick: () => void;
}

function Tile(props: TileProps) {
  // Default 3:2 landscape until the image reports its natural aspect; then
  // the flex-basis formula (--aspect * --row-height) gives each tile its
  // proper proportional width within the row.
  const [aspect, setAspect] = createSignal(1.5);
  const mediaType = () => props.asset.mediaType;
  const isImage = () => mediaType().startsWith('image/');
  const isVideo = () => mediaType().startsWith('video/');
  const isAudio = () => mediaType().startsWith('audio/');

  return (
    <button
      type="button"
      class="gallery-tile"
      style={{ '--aspect': String(aspect()) }}
      onClick={props.onClick}
    >
      <Switch fallback={<GenericTile asset={props.asset} />}>
        <Match when={isImage()}>
          <ImageTile asset={props.asset} onAspect={setAspect} />
        </Match>
        <Match when={isVideo()}>
          <VideoTile asset={props.asset} onAspect={setAspect} />
        </Match>
        <Match when={isAudio()}>
          <AudioTile asset={props.asset} />
        </Match>
      </Switch>
      <time class="gallery-caption">
        {format.formatDatetime(props.asset.datetime)}
      </time>
    </button>
  );
}

interface MediaTileProps {
  asset: SearchResult;
  onAspect: (aspect: number) => void;
}

function ImageTile(props: MediaTileProps) {
  return (
    <img
      class="gallery-image"
      src={props.asset.previewUrl}
      alt={props.asset.filename}
      loading="lazy"
      onLoad={(e) => {
        const img = e.currentTarget;
        if (img.naturalHeight > 0) {
          props.onAspect(img.naturalWidth / img.naturalHeight);
        }
      }}
    />
  );
}

function VideoTile(props: MediaTileProps) {
  return (
    <>
      <img
        class="gallery-image"
        src={props.asset.previewUrl}
        alt={props.asset.filename}
        loading="lazy"
        onLoad={(e) => {
          const img = e.currentTarget;
          if (img.naturalHeight > 0) {
            props.onAspect(img.naturalWidth / img.naturalHeight);
          }
        }}
      />
      <span class="gallery-play-overlay" aria-hidden="true">
        <i class="fa-solid fa-circle-play"></i>
      </span>
    </>
  );
}

function AudioTile(props: { asset: SearchResult }) {
  return (
    <span class="gallery-audio-placeholder">
      <i class="fa-solid fa-music" aria-hidden="true"></i>
      <span class="gallery-audio-filename">{props.asset.filename}</span>
    </span>
  );
}

function GenericTile(props: { asset: SearchResult }) {
  // Fallback for media types we can't render a preview for (e.g. PDF) —
  // shows a neutral icon and the filename instead of a broken image.
  return (
    <span class="gallery-audio-placeholder">
      <i class="fa-solid fa-file" aria-hidden="true"></i>
      <span class="gallery-audio-filename">{props.asset.filename}</span>
    </span>
  );
}

export default JustifiedRows;
