//
// Copyright (c) 2026 Nathan Fiedler
//
import { For } from 'solid-js';
import type { SearchResult } from 'tanuki/generated/graphql.ts';
import * as format from '../helpers/formatting.ts';
import GalleryMedia from './gallery-media.tsx';

interface MasonryGridProps {
  results?: SearchResult[];
  onClick: (assetId: string, index: number) => void;
}

function MasonryGrid(props: MasonryGridProps) {
  return (
    <div class="masonry">
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
  return (
    <button type="button" class="masonry-tile" onClick={props.onClick}>
      <GalleryMedia asset={props.asset} imageClass="masonry-image" />
      <time class="masonry-caption">
        {format.formatDatetime(props.asset.datetime)}
      </time>
    </button>
  );
}

export default MasonryGrid;
