//
// Copyright (c) 2025 Nathan Fiedler
//
import type { Accessor, Setter } from 'solid-js';
import {
  createMemo,
  createResource,
  createSignal,
  For,
  type JSX,
  Match,
  Show,
  Suspense,
  Switch
} from 'solid-js';
import { A, action, useAction, useSubmission } from '@solidjs/router';
import { type TypedDocumentNode, gql } from '@apollo/client';
import { useApolloClient } from '../apollo-provider.tsx';
import {
  type Location,
  type MutationUpdateArgs,
  type Mutation,
  type QueryPendingArgs,
  type Query,
  type SearchResult,
  SortField as GQLSortField,
  SortOrder as GQLSortOrder
} from 'tanuki/generated/graphql.ts';
import AttributeChips from '../components/attribute-chips.tsx';
import Pagination from '../components/pagination.tsx';
import TagSelector from '../components/tag-selector.tsx';
import * as format from '../helpers/formatting.ts';
import * as parse from '../helpers/parsing.ts';
import useClickOutside from '../hooks/use-click-outside.ts';
import useLocalStorage from '../hooks/use-local-storage.ts';

const PENDING_ASSETS: TypedDocumentNode<Query, QueryPendingArgs> = gql`
  query Pending($params: PendingParams!, $offset: Int, $limit: Int) {
    pending(params: $params, offset: $offset, limit: $limit) {
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
        thumbnailUrl
        assetUrl
      }
      count
      lastPage
    }
  }
`;

const UPDATE_ASSET: TypedDocumentNode<Mutation, MutationUpdateArgs> = gql`
  mutation Update($id: ID!, $asset: AssetInput!) {
    update(id: $id, asset: $asset) {
      id
    }
  }
`;

function buildParams({
  range,
  offset,
  limit,
  sortField,
  sortOrder
}: {
  range: number;
  offset: number;
  limit: number;
  sortField: GQLSortField;
  sortOrder: GQLSortOrder;
}): QueryPendingArgs {
  let afterDate: Date | null = new Date();
  if (range === 0) {
    afterDate = null;
  } else {
    afterDate.setDate(afterDate.getDate() - range);
  }
  return {
    params: {
      after: afterDate,
      sortField,
      sortOrder
    },
    offset,
    limit
  };
}

function Pending() {
  const client = useApolloClient();
  const [range, setRange] = useLocalStorage('pending-selected-range', 0);
  const [sortCombo, setSortCombo] = useLocalStorage(
    'pending-sort-combo',
    'DateAsc'
  );
  const [selectedPage, setSelectedPage] = useLocalStorage(
    'pending-selected-page',
    1
  );
  const [pageSize, setPageSize] = useLocalStorage('page-size', 18);
  const pendingParams = createMemo(() => ({
    range: range(),
    offset: pageSize() * (selectedPage() - 1),
    limit: pageSize(),
    sortField: sortComboField(sortCombo()),
    sortOrder: sortComboOrder(sortCombo())
  }));
  // query() and createAsync() are neat but they do not automatically run when
  // input signals change, or it is difficult to understand how they work
  const [assetsQuery] = createResource(pendingParams, async (params) => {
    const { data } = await client.query({
      query: PENDING_ASSETS,
      variables: buildParams(params)
    });
    return data;
  });
  const lastPage = () => assetsQuery()?.pending.lastPage ?? 1;
  let datetimeRef: HTMLInputElement | undefined;
  const [selectedTags, setSelectedTags] = createSignal<string[]>([]);
  const [selectedLocation, setSelectedLocation] = createSignal<Location>({
    label: null,
    city: null,
    region: null
  });
  const [selectedAssets, setSelectedAssets] = createSignal<Set<string>>(
    new Set(),
    {
      // avoid having to create a new set in order for SolidJS to notice
      equals: (prev, next) => prev.size !== next.size
    }
  );
  const submittable = createMemo(() => {
    return (
      (selectedTags().length > 0 || locationHasValues(selectedLocation())) &&
      selectedAssets().size > 0
    );
  });
  const updateAction = action(async (): Promise<any> => {
    const tags = selectedTags() || null;
    const location = locationHasValues(selectedLocation())
      ? selectedLocation()
      : null;
    const datetime = datetimeRef?.value ? new Date(datetimeRef?.value) : null;
    for (const assetId of selectedAssets()) {
      try {
        await client.mutate({
          mutation: UPDATE_ASSET,
          variables: {
            id: assetId,
            asset: { tags, location, datetime }
          }
        });
      } catch (error) {
        console.error('asset update failed:', error);
        // force an early exit so the user has a chance to look at the browser
        // console to see the error message
        return { ok: false };
      }
    }
    // SolidJS router is _supposed_ to revalidate the queries on this page, but
    // nothing makes any difference, even calling revalidate() or reload()
    // explicitly does nothing, so just force the page to reload instead.
    window.location.reload();
    return { ok: true };
  }, 'updateAssets');
  const startUpdate = useAction(updateAction);
  const updateSubmission = useSubmission(updateAction);
  const saveButtonClass = createMemo(() => {
    if (updateSubmission.pending) {
      return 'button is-loading';
    } else if (submittable()) {
      return 'button is-primary';
    }
    return 'button';
  });

  return (
    <>
      <div class="container mb-4">
        <nav class="level">
          <div class="level-left">
            <div class="level-item">
              <RecentRange range={range} setRange={setRange} />
            </div>
            <div class="level-item">
              <PendingCount data={assetsQuery} />
            </div>
          </div>
          <div class="level-right">
            <div class="level-item">
              <SortOrder sortCombo={sortCombo} setSortCombo={setSortCombo} />
            </div>
            <Suspense>
              <Pagination
                lastPage={lastPage}
                selectedPage={selectedPage}
                setSelectedPage={setSelectedPage}
                pageSize={pageSize}
                setPageSize={setPageSize}
              />
            </Suspense>
          </div>
        </nav>
      </div>

      <div class="container">
        <nav class="level">
          <div class="level-left">
            <div class="level-item">
              <TagSelector
                addfun={(value) => {
                  setSelectedTags((tags) => {
                    if (!tags.includes(value)) {
                      // return a new array so SolidJS will take note
                      return [...tags, value];
                    }
                    return tags;
                  });
                }}
              />
            </div>
            <div class="level-item">
              <LocationRecordSelector
                setLocation={(value) => {
                  setSelectedLocation(value);
                }}
              />
            </div>
            <div class="level-item">
              <div class="field is-horizontal">
                <div class="field-label is-normal">
                  <label class="label" for="date-input">
                    Date
                  </label>
                </div>
                <div class="field-body">
                  <p class="control">
                    <input
                      class="input"
                      id="date-input"
                      type="datetime-local"
                      ref={(el: HTMLInputElement) => (datetimeRef = el)}
                    />
                  </p>
                </div>
              </div>
            </div>
          </div>

          <div class="level-right">
            <div class="level-item">
              <input
                class={saveButtonClass()}
                type="submit"
                value="Save"
                disabled={!submittable() || updateSubmission.pending}
                on:click={(_) => startUpdate()}
              />
            </div>
          </div>
        </nav>
      </div>

      <div class="container mt-3 mb-3">
        <div class="field is-grouped is-grouped-multiline">
          <AttributeChips
            attrs={selectedTags}
            rmfun={(attr) => {
              setSelectedTags((tags) => {
                // return a new array so SolidJS will take note
                return tags.filter((t) => t !== attr);
              });
            }}
          />
        </div>
      </div>

      <Suspense fallback={<button class="button is-loading">...</button>}>
        <PendingAssets
          results={assetsQuery()?.pending.results}
          selectedAssets={selectedAssets}
          setSelectedAssets={setSelectedAssets}
        />
      </Suspense>
    </>
  );
}

function locationHasValues(location: Location): boolean {
  return (
    location.label !== null ||
    location.city !== null ||
    location.region !== null
  );
}

interface RecentRangeProps {
  range: Accessor<number>;
  setRange: Setter<number>;
}

const rangeValues = [
  { label: 'All', value: 0 },
  { label: 'Year', value: 365 },
  { label: 'Month', value: 30 },
  { label: 'Week', value: 7 },
  { label: 'Day', value: 1 }
];

function RecentRange(props: RecentRangeProps) {
  return (
    <div class="field is-grouped">
      <For each={rangeValues}>
        {(range) => (
          <Show
            when={range.value === props.range()}
            fallback={
              <p class="control">
                <button
                  class="button"
                  on:click={() => props.setRange(range.value)}
                >
                  {range.label}
                </button>
              </p>
            }
          >
            <p class="control">
              <button class="button is-active">{range.label}</button>
            </p>
          </Show>
        )}
      </For>
    </div>
  );
}

interface PendingCountProps {
  data: Accessor<Query | undefined>;
}

function PendingCount(props: PendingCountProps) {
  return (
    <Suspense fallback={'...'}>
      <span>{`Pending items: ${props.data()?.pending.count}`}</span>
    </Suspense>
  );
}

const LOCATION_RECORDS: TypedDocumentNode<Query, Record<string, never>> = gql`
  query {
    locationRecords {
      label
      city
      region
    }
  }
`;

interface LocationRecordSelectorProps {
  setLocation: (location: Location) => void;
}

function LocationRecordSelector(props: LocationRecordSelectorProps) {
  const client = useApolloClient();
  const [locationsQuery] = createResource(async () => {
    const { data } = await client.query({ query: LOCATION_RECORDS });
    return data;
  });

  //
  // n.b. on:input is called for every single keystroke, while on:change is
  // called under several conditions:
  //
  // - user selects one of the available datalist options
  // - user types some text and presses the Enter key
  // - user types some text and moves the focus
  //
  const onChange: JSX.EventHandlerWithOptionsUnion<
    HTMLInputElement,
    Event,
    JSX.ChangeEventHandler<HTMLInputElement, Event>
  > = (event) => {
    const target = event.currentTarget;
    if (target) {
      const value = target.value;
      if (value) {
        props.setLocation(parse.parseLocation(value));
      } else {
        props.setLocation({ label: null, city: null, region: null });
      }
      event.stopPropagation();
    }
  };

  return (
    <Suspense fallback={'...'}>
      <div class="field is-horizontal">
        <div class="field-label is-normal">
          <label class="label" for="locations-input">
            Location
          </label>
        </div>
        <div class="field-body">
          <p class="control">
            <input
              class="input"
              type="text"
              id="locations-input"
              list="location-labels"
              placeholder="Choose location"
              on:change={onChange}
            />
            <datalist id="location-labels">
              <For each={locationsQuery()?.locationRecords}>
                {(location) => (
                  <option value={format.formatLocation(location)}></option>
                )}
              </For>
            </datalist>
          </p>
        </div>
      </div>
    </Suspense>
  );
}

function sortComboIcon(value: string): string {
  switch (value) {
    case 'DateAsc': {
      return 'fa-solid fa-arrow-down-1-9';
    }
    case 'DateDesc': {
      return 'fa-solid fa-arrow-up-9-1';
    }
    case 'FileAsc': {
      return 'fa-solid fa-arrow-down-a-z';
    }
    case 'FileDesc': {
      return 'fa-solid fa-arrow-up-z-a';
    }
  }
  // return the default if nothing else
  return 'fa-solid fa-arrow-down-1-9';
}

function sortComboLabel(value: string): string {
  switch (value) {
    case 'DateAsc':
    case 'DateDesc': {
      return 'Date';
    }
    case 'FileAsc':
    case 'FileDesc': {
      return 'Filename';
    }
  }
  // return the default if nothing else
  return 'Date';
}

function sortComboField(value: string): GQLSortField {
  switch (value) {
    case 'DateAsc':
    case 'DateDesc': {
      return GQLSortField.Date;
    }
    case 'FileAsc':
    case 'FileDesc': {
      return GQLSortField.Filename;
    }
  }
  // return the default if nothing else
  return GQLSortField.Date;
}

function sortComboOrder(value: string): GQLSortOrder {
  switch (value) {
    case 'DateAsc':
    case 'FileAsc': {
      return GQLSortOrder.Ascending;
    }
    case 'DateDesc':
    case 'FileDesc': {
      return GQLSortOrder.Descending;
    }
  }
  // return the default if nothing else
  return GQLSortOrder.Ascending;
}

interface SortOrderProps {
  sortCombo: Accessor<string>;
  setSortCombo: Setter<string>;
}

function SortOrder(props: SortOrderProps) {
  const [dropdownOpen, setDropdownOpen] = createSignal(false);
  let dropdownRef: HTMLDivElement | undefined;
  useClickOutside(
    () => dropdownRef,
    () => setDropdownOpen(false)
  );

  return (
    <div
      class="dropdown"
      ref={(el: HTMLDivElement) => (dropdownRef = el)}
      class:is-active={dropdownOpen()}
    >
      <div class="dropdown-trigger">
        <button
          class="button"
          on:click={() => setDropdownOpen((v) => !v)}
          aria-haspopup="true"
          aria-controls="sort-menu"
        >
          <span>{sortComboLabel(props.sortCombo())}</span>
          <span class="icon">
            <i class={sortComboIcon(props.sortCombo())} aria-hidden="true"></i>
          </span>
        </button>
      </div>

      <div class="dropdown-menu" id="sort-menu" role="menu">
        <div class="dropdown-content">
          <a
            class={
              props.sortCombo() === 'DateAsc'
                ? 'dropdown-item is-active'
                : 'dropdown-item'
            }
            on:click={(_) => {
              props.setSortCombo('DateAsc');
              setDropdownOpen(false);
            }}
          >
            <span>Date</span>
            <span class="icon">
              <i class="fa-solid fa-arrow-down-1-9" aria-hidden="true"></i>
            </span>
          </a>
          <a
            class={
              props.sortCombo() === 'DateDesc'
                ? 'dropdown-item is-active'
                : 'dropdown-item'
            }
            on:click={(_) => {
              props.setSortCombo('DateDesc');
              setDropdownOpen(false);
            }}
          >
            <span>Date</span>
            <span class="icon">
              <i class="fa-solid fa-arrow-up-9-1" aria-hidden="true"></i>
            </span>
          </a>
          <a
            class={
              props.sortCombo() === 'FileAsc'
                ? 'dropdown-item is-active'
                : 'dropdown-item'
            }
            on:click={(_) => {
              props.setSortCombo('FileAsc');
              setDropdownOpen(false);
            }}
          >
            <span>Filename</span>
            <span class="icon">
              <i class="fa-solid fa-arrow-down-a-z" aria-hidden="true"></i>
            </span>
          </a>
          <a
            class={
              props.sortCombo() === 'FileDesc'
                ? 'dropdown-item is-active'
                : 'dropdown-item'
            }
            on:click={(_) => {
              props.setSortCombo('FileDesc');
              setDropdownOpen(false);
            }}
          >
            <span>Filename</span>
            <span class="icon">
              <i class="fa-solid fa-arrow-up-z-a" aria-hidden="true"></i>
            </span>
          </a>
        </div>
      </div>
    </div>
  );
}

interface PendingAssetsProps {
  results?: SearchResult[];
  selectedAssets: Accessor<Set<string>>;
  setSelectedAssets: Setter<Set<string>>;
}

function PendingAssets(props: PendingAssetsProps) {
  // would use createSelector() here but that appears to only track a single
  // selection rather than a selected status based on set membership
  const toggleAsset = (id: string) => {
    props.setSelectedAssets((s) => {
      if (s.has(id)) {
        s.delete(id);
      } else {
        s.add(id);
      }
      return s;
    });
  };
  const cardClass = (id: string): string => {
    return props.selectedAssets().has(id) ? 'card selected' : 'card';
  };

  return (
    <div class="grid is-col-min-16 padding-2">
      <For each={props.results}>
        {(asset) => (
          <div class="cell">
            <a onClick={() => toggleAsset(asset.assetId)}>
              <div class={cardClass(asset.assetId)}>
                <header class="card-header">
                  <p class="card-header-title">{asset.filename}</p>
                  <A href={`/asset/${asset.assetId}`}>
                    <button class="card-header-icon">
                      <span class="icon">
                        <i class="fas fa-angle-right"></i>
                      </span>
                    </button>
                  </A>
                </header>
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

export default Pending;
