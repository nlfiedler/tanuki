//
// Copyright (c) 2026 Nathan Fiedler
//
import { For, Show } from 'solid-js';
import type { SearchResult } from 'tanuki/generated/graphql.ts';
import * as format from '../helpers/formatting.ts';
import GalleryMedia from './gallery-media.tsx';
import cameraShutterRaw from '../assets/icons/camera-shutter.svg?raw';

interface ThumbListProps {
  results?: SearchResult[];
  onClick: (assetId: string, index: number) => void;
}

function ThumbList(props: ThumbListProps) {
  return (
    <div class="thumb-grid">
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
  const datetime = () =>
    format.formatDatetimeWithTZ(
      props.asset.datetime,
      props.asset.metadata?.originalDateOffset
    );
  const camera = () => format.formatCamera(props.asset.metadata);
  const lens = () => format.formatLens(props.asset.metadata);
  const dimensions = () =>
    format.formatFormat(props.asset.mediaType, props.asset.metadata);
  const location = () =>
    props.asset.location ? format.formatLocation(props.asset.location) : '';

  return (
    <button type="button" class="thumb-tile" onClick={props.onClick}>
      <span class="thumb-image-wrap">
        <GalleryMedia asset={props.asset} imageClass="thumb-image" />
      </span>
      <div class="thumb-info">
        <div class="thumb-title">{format.formatTitle(props.asset)}</div>
        <Show when={datetime()}>
          <Line iconClass="fa-regular fa-calendar-days" text={datetime()} />
        </Show>
        <Show when={camera()}>
          <Line iconClass="fa-solid fa-camera-retro" text={camera()} />
        </Show>
        <Show when={lens()}>
          <Line iconSvg={cameraShutterRaw} text={lens()} />
        </Show>
        <Show when={dimensions()}>
          <Line iconClass="fa-regular fa-image" text={dimensions()} />
        </Show>
        <Show when={props.asset.filename}>
          <Line iconClass="fa-regular fa-file" text={props.asset.filename} />
        </Show>
        <Show when={location()}>
          <Line iconClass="fa-solid fa-location-dot" text={location()} />
        </Show>
      </div>
    </button>
  );
}

interface LineProps {
  iconClass?: string;
  iconSvg?: string;
  text: string;
}

function Line(props: LineProps) {
  return (
    <div class="thumb-line">
      <Show
        when={props.iconSvg}
        fallback={
          <span class="icon">
            <i class={props.iconClass} aria-hidden="true"></i>
          </span>
        }
      >
        <span class="icon" innerHTML={props.iconSvg} aria-hidden="true" />
      </Show>
      <span>{props.text}</span>
    </div>
  );
}

export default ThumbList;
