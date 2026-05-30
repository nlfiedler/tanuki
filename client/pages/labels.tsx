//
// Copyright (c) 2026 Nathan Fiedler
//
import {
  createMemo,
  createResource,
  createSignal,
  For,
  Show,
  Suspense
} from 'solid-js';
import { useNavigate, useParams } from '@solidjs/router';
import { type TypedDocumentNode, gql } from '@apollo/client';
import { useApolloClient } from '../apollo-provider.tsx';
import type {
  LabelEntry,
  Query,
  QueryAssetsByLabelArgs
} from 'tanuki/generated/graphql.ts';
import Pagination from '../components/pagination.tsx';
import ThumbList from '../components/thumb-list.tsx';
import useLocalStorage from '../hooks/use-local-storage.ts';

const LABELS: TypedDocumentNode<Query, Record<string, never>> = gql`
  query Labels {
    labels {
      label
      count
      thumbnail
    }
  }
`;

const ASSETS_BY_LABEL: TypedDocumentNode<Query, QueryAssetsByLabelArgs> = gql`
  query AssetsByLabel($label: String!, $offset: Int, $limit: Int) {
    assetsByLabel(label: $label, offset: $offset, limit: $limit) {
      results {
        assetId
        datetime
        filename
        location {
          label
          city
          region
        }
        mediaType
        previewUrl
        thumbnailUrl
        assetUrl
        metadata {
          displayWidth
          displayHeight
        }
        synthetic {
          primaryLabel
        }
      }
      count
      lastPage
    }
  }
`;

/**
 * Index of every distinct `primaryLabel` in the library, each rendered as a
 * tile with its representative thumbnail and asset count. Clicking a tile
 * navigates to `/labels/<label>` to see the matching assets.
 */
function Labels() {
  const client = useApolloClient();
  const navigate = useNavigate();
  const [labelsQuery] = createResource(async () => {
    const { data } = await client.query({ query: LABELS });
    return data;
  });
  const entries = createMemo(() => {
    const list = labelsQuery()?.labels ?? [];
    // alphabetize so the page is stable across reloads; the spec leaves this
    // open and alpha is more useful than backend order
    return [...list].sort((a, b) => a.label.localeCompare(b.label));
  });

  return (
    <section class="section">
      <div class="container">
        <h1 class="title is-4">Labels</h1>
        <Suspense fallback={<p>Loading labels…</p>}>
          <Show
            when={entries().length > 0}
            fallback={
              <p class="has-text-grey">
                No labels yet — synthetic-data extraction is still in progress
                or no images have been imported.
              </p>
            }
          >
            <div class="thumb-grid">
              <For each={entries()}>
                {(entry) => (
                  <LabelTile
                    entry={entry}
                    onClick={() =>
                      navigate(`/labels/${encodeURIComponent(entry.label)}`)
                    }
                  />
                )}
              </For>
            </div>
          </Show>
        </Suspense>
      </div>
    </section>
  );
}

interface LabelTileProps {
  entry: LabelEntry;
  onClick: () => void;
}

function LabelTile(props: LabelTileProps) {
  const countText = () =>
    `${props.entry.count} ${props.entry.count === 1 ? 'asset' : 'assets'}`;
  return (
    <button type="button" class="thumb-tile" onClick={props.onClick}>
      <span class="thumb-image-wrap">
        <img
          class="thumb-image"
          src={props.entry.thumbnail}
          alt={props.entry.label}
          loading="lazy"
        />
      </span>
      <div class="thumb-info">
        <div class="thumb-title">{props.entry.label}</div>
        <div class="thumb-line">
          <span class="icon">
            <i class="fa-regular fa-image" aria-hidden="true"></i>
          </span>
          <span>{countText()}</span>
        </div>
      </div>
    </button>
  );
}

/**
 * Grid of every asset whose `primaryLabel` matches the `:label` route
 * parameter. Backed by the `assetsByLabel` GraphQL query, which uses the
 * indexed view rather than a full scan.
 */
function LabelAssets() {
  const client = useApolloClient();
  const navigate = useNavigate();
  const params = useParams<{ label: string }>();
  const label = () => decodeURIComponent(params.label);
  const [selectedPage, setSelectedPage] = createSignal(1);
  const [pageSize, setPageSize] = useLocalStorage('page-size', 18);
  const queryArgs = createMemo(() => ({
    label: label(),
    offset: pageSize() * (selectedPage() - 1),
    limit: pageSize()
  }));
  const [assetsQuery] = createResource(queryArgs, async (args) => {
    const { data } = await client.query({
      query: ASSETS_BY_LABEL,
      variables: args
    });
    return data;
  });
  const results = () => assetsQuery()?.assetsByLabel.results ?? [];
  const total = () => assetsQuery()?.assetsByLabel.count ?? 0;
  const lastPage = () => assetsQuery()?.assetsByLabel.lastPage ?? 1;
  const openAsset = (assetId: string) => navigate(`/asset/${assetId}`);

  return (
    <section class="section">
      <div class="container">
        <nav class="level">
          <div class="level-left">
            <div class="level-item">
              <h1 class="title is-4">
                <span class="has-text-grey-light">Label /</span> {label()}
              </h1>
            </div>
            <div class="level-item has-text-grey">
              <Suspense fallback="…">
                {total()} {total() === 1 ? 'asset' : 'assets'}
              </Suspense>
            </div>
          </div>
          <div class="level-right">
            <Pagination
              lastPage={lastPage}
              selectedPage={selectedPage}
              setSelectedPage={setSelectedPage}
              pageSize={pageSize}
              setPageSize={setPageSize}
            />
          </div>
        </nav>
        <Suspense fallback={<p>Loading…</p>}>
          <Show
            when={results().length > 0}
            fallback={
              <p class="has-text-grey">
                No assets currently carry this label.
              </p>
            }
          >
            <ThumbList results={results()} onClick={openAsset} />
          </Show>
        </Suspense>
      </div>
    </section>
  );
}

export { LabelAssets, Labels };
export default Labels;
