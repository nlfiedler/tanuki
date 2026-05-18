//
// Copyright (c) 2026 Nathan Fiedler
//
import { For } from 'solid-js';
import type { SearchResult } from 'tanuki/generated/graphql.ts';
import * as format from '../helpers/formatting.ts';
import GalleryMedia from './gallery-media.tsx';

// The page query selects `previewUrlSmall: previewUrl(height: 320)` so each
// square can load a tile-sized preview instead of the 800px-tall default. The
// alias isn't on the schema-level SearchResult type, so we read it via a local
// cast at the boundary where this assumption lives.
type WithSmallPreview = SearchResult & { previewUrlSmall?: string };

interface SquareGridProps {
  results?: SearchResult[];
  onClick: (assetId: string, index: number) => void;
}

function SquareGrid(props: SquareGridProps) {
  return (
    <div class="square-grid">
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
  const smallSrc = () => (props.asset as WithSmallPreview).previewUrlSmall;
  return (
    <button type="button" class="square-tile" onClick={props.onClick}>
      <GalleryMedia
        asset={props.asset}
        imageClass="square-image"
        imageSrc={smallSrc()}
      />
      <time class="square-caption">
        {format.formatDatetime(props.asset.datetime)}
      </time>
    </button>
  );
}

export default SquareGrid;
