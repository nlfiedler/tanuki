//
// Copyright (c) 2025 Nathan Fiedler
//
import { type Accessor, For, Match, Show, Switch } from 'solid-js';
import type { Location, SearchResult } from 'tanuki/generated/graphql.ts';
import * as format from '../helpers/formatting.ts';

interface CardsGridProps {
  results?: SearchResult[];
  selectedAssets: Accessor<Set<string>>;
  onClick: (assetId: string) => void;
}

function CardsGrid(props: CardsGridProps) {
  const cardClass = (id: string): string => {
    return props.selectedAssets().has(id) ? 'card selected' : 'card';
  };

  return (
    <div class="grid is-col-min-16 padding-2">
      <For each={props.results}>
        {(asset) => (
          <div class="cell">
            <a onClick={() => props.onClick(asset.assetId)}>
              <div class={cardClass(asset.assetId)}>
                <div class="card-image">
                  <figure class="image">
                    <Switch fallback={<ImageThumbnail asset={asset} />}>
                      <Match when={asset.mediaType.startsWith('video/')}>
                        <VideoThumbnail asset={asset} />
                      </Match>
                      <Match when={asset.mediaType.startsWith('audio/')}>
                        <AudioThumbnail asset={asset} />
                      </Match>
                    </Switch>
                  </figure>
                </div>
                <div class="card-content">
                  <div class="content">
                    <CardContent
                      datetime={asset.datetime}
                      location={asset.location}
                    />
                  </div>
                </div>
              </div>
            </a>
          </div>
        )}
      </For>
    </div>
  );
}

interface ImageThumbnailProps {
  asset: SearchResult;
}

function ImageThumbnail(props: ImageThumbnailProps) {
  return (
    <img
      src={props.asset.thumbnailUrl}
      alt={props.asset.filename}
      style="max-width: 100%; width: auto; padding: inherit; margin: auto; display: block;"
    />
  );
}

interface VideoThumbnailProps {
  asset: SearchResult;
}

function VideoThumbnail(props: VideoThumbnailProps) {
  let media_type = props.asset.mediaType;
  if (media_type == 'video/quicktime') {
    media_type = 'video/mp4';
  }
  return (
    <video controls>
      <source src={props.asset.assetUrl} type={media_type} />
      Bummer, your browser does not support the HTML5
      <code>video</code>
      tag.
    </video>
  );
}

interface AudioThumbnailProps {
  asset: SearchResult;
}

function AudioThumbnail(props: AudioThumbnailProps) {
  return (
    <>
      <figcaption>{props.asset.filename}</figcaption>
      <audio controls>
        <source src={props.asset.assetUrl} type={props.asset.mediaType} />
      </audio>
    </>
  );
}

interface CardContentProps {
  datetime: Date;
  location: Location | null | undefined;
}

function CardContent(props: CardContentProps) {
  return (
    <div class="content">
      <time>{format.formatDatetime(props.datetime)}</time>
      <Show when={props.location} fallback={<></>}>
        <br />
        <span>{format.formatLocation(props.location!)}</span>
      </Show>
    </div>
  );
}

export default CardsGrid;
