//
// Copyright (c) 2026 Nathan Fiedler
//
import { createSignal, For } from 'solid-js';
import type { SearchResult } from 'tanuki/generated/graphql.ts';
import * as format from '../helpers/formatting.ts';
import GalleryMedia from './gallery-media.tsx';

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
  // Seed from extracted metadata when available so the row packs correctly on
  // first paint; otherwise fall back to 3:2 landscape and let onAspect update
  // once the image loads. The flex-basis formula (--aspect * --row-height)
  // gives each tile its proper proportional width within the row.
  const initialAspect = () => {
    const w = props.asset.metadata?.displayWidth;
    const h = props.asset.metadata?.displayHeight;
    return w && h && h > 0 ? w / h : 1.5;
  };
  const [aspect, setAspect] = createSignal(initialAspect());

  return (
    <button
      type="button"
      class="gallery-tile"
      style={{ '--aspect': String(aspect()) }}
      onClick={props.onClick}
    >
      <GalleryMedia
        asset={props.asset}
        imageClass="gallery-image"
        onAspect={setAspect}
      />
      <time class="gallery-caption">
        {format.formatDatetime(props.asset.datetime)}
      </time>
    </button>
  );
}

export default JustifiedRows;
