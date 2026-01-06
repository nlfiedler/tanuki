//
// Copyright (c) 2025 Nathan Fiedler
//
import {
  createMemo,
  createResource,
  createSignal,
  For,
  type Accessor,
  type JSX,
  Match,
  type Setter,
  Show,
  Suspense,
  Switch
} from 'solid-js';
import { action, useAction, useSubmission } from '@solidjs/router';
import { type TypedDocumentNode, gql } from '@apollo/client';
import { useApolloClient } from '../apollo-provider.tsx';
import {
  type AttributeCount,
  LocationField,
  type Mutation,
  type MutationEditArgs,
  type QueryScanArgs,
  type QueryTagsForAssetsArgs,
  type Query,
  SortField,
  SortOrder
} from 'tanuki/generated/graphql.ts';
import CardsGrid from '../components/cards-grid.tsx';
import Pagination from '../components/pagination.tsx';
import useLocalStorage from '../hooks/use-local-storage.ts';

const SCAN_ASSETS: TypedDocumentNode<Query, QueryScanArgs> = gql`
  query Scan(
    $query: String!
    $field: SortField
    $order: SortOrder
    $offset: Int
    $limit: Int
  ) {
    scan(
      query: $query
      sortField: $field
      sortOrder: $order
      offset: $offset
      limit: $limit
    ) {
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

function buildParams({
  query,
  offset,
  limit,
  sortOrder
}: {
  query: string;
  offset: number;
  limit: number;
  sortOrder: SortOrder;
}): QueryScanArgs {
  return {
    query,
    sortField: SortField.Date,
    sortOrder,
    offset,
    limit
  };
}

function Edit() {
  const client = useApolloClient();
  const [queryString, setQueryString] = useLocalStorage<string>(
    'edit-query',
    ''
  );
  const [selectedSortOrder, setSelectedSortOrder] = useLocalStorage<SortOrder>(
    'edit-sort-order',
    SortOrder.Descending
  );
  const [selectedPage, setSelectedPage] = useLocalStorage(
    'edit-selected-page',
    1
  );
  const [pageSize, setPageSize] = useLocalStorage('page-size', 18);
  const scanParams = createMemo(() => ({
    query: queryString(),
    offset: pageSize() * (selectedPage() - 1),
    limit: pageSize(),
    sortOrder: selectedSortOrder()
  }));
  const [assetsQuery] = createResource(scanParams, async (params) => {
    const { data } = await client.query({
      query: SCAN_ASSETS,
      variables: buildParams(params)
    });
    return data;
  });
  const lastPage = () => assetsQuery()?.scan.lastPage ?? 1;
  const queryOnChange: JSX.EventHandlerWithOptionsUnion<
    HTMLInputElement,
    Event,
    JSX.ChangeEventHandler<HTMLInputElement, Event>
  > = (event) => {
    const target = event.currentTarget;
    if (target) {
      setQueryString(target.value);
      event.stopPropagation();
    }
  };
  const [selectedAssets, setSelectedAssets] = createSignal(new Set<string>());
  const cardOnClick = (assetId: string) => {
    // Create a new set so that SolidJS will notice the change; while we could
    // override the comparator, createResource() will not notice the difference.
    const coll = new Set<string>(selectedAssets());
    if (coll.has(assetId)) {
      coll.delete(assetId);
    } else {
      coll.add(assetId);
    }
    setSelectedAssets(coll);
    if (coll.size === 0) {
      // wipe out the invisible add/remove tags inputs
      setAddedTags(new Set<string>());
      setRemovedTags(new Set<string>());
    }
  };
  const selectAllAction = action(async () => {
    const coll = new Set<string>();
    for (const asset of assetsQuery()?.scan.results ?? []) {
      coll.add(asset.assetId);
    }
    setSelectedAssets(coll);
  });
  const selectAll = useAction(selectAllAction);
  const unselectAllAction = action(async () => {
    setSelectedAssets(new Set<string>());
    // wipe out the invisible add/remove tags inputs
    setAddedTags(new Set<string>());
    setRemovedTags(new Set<string>());
  });
  const unselectAll = useAction(unselectAllAction);
  const [addedTags, setAddedTags] = createSignal(new Set<string>());
  const addTag = (label: string) => {
    const coll = new Set<string>(addedTags());
    coll.add(label);
    setAddedTags(coll);
    // cannot add and remove the same tag
    if (removedTags().has(label)) {
      const coll = new Set<string>(removedTags());
      coll.delete(label);
      setRemovedTags(coll);
    }
  };
  const [removedTags, setRemovedTags] = createSignal(new Set<string>());
  const removeTag = (label: string) => {
    const coll = new Set<string>(removedTags());
    coll.add(label);
    setRemovedTags(coll);
    // cannot add and remove the same tag
    if (addedTags().has(label)) {
      const coll = new Set<string>(addedTags());
      coll.delete(label);
      setAddedTags(coll);
    }
  };
  const [place, setPlace] = createSignal('');
  const [city, setCity] = createSignal('');
  const [region, setRegion] = createSignal('');
  const [datetime, setDatetime] = createSignal('');
  const [days, setDays] = createSignal(0);
  const submittable = createMemo(() => {
    return ((place() ||
      city() ||
      region() ||
      datetime() ||
      days() ||
      addedTags().size > 0 ||
      removedTags().size > 0) &&
      selectedAssets().size > 0) as boolean;
  });
  const [modalOpen, setModalOpen] = createSignal(false);

  return (
    <>
      <div class="container my-3">
        <nav class="level">
          <div class="level-left">
            <div class="level-item">
              <div class="field is-horizontal">
                <div class="field-label is-normal">
                  <label class="label" for="query-input">
                    Query
                  </label>
                </div>
                <div class="field-body">
                  <p class="control is-expanded">
                    <input
                      class="input"
                      style="max-width: 300%; width: 300%;"
                      type="text"
                      id="query-input"
                      placeholder="Enter a search query"
                      value={queryString()}
                      on:change={queryOnChange}
                    />
                  </p>
                </div>
              </div>
            </div>
          </div>

          <div class="level-right">
            <div class="level-item">
              <button class="button" on:click={(_) => unselectAll()}>
                <span class="icon">
                  <i class="fa-regular fa-square"></i>
                </span>
              </button>
            </div>
            <div class="level-item">
              <button class="button" on:click={(_) => selectAll()}>
                <span class="icon">
                  <i class="fa-regular fa-square-check"></i>
                </span>
              </button>
            </div>
            <div class="level-item">
              <div class="field">
                <p class="control">
                  <SortOrderSelector
                    selectedSortOrder={selectedSortOrder}
                    setSortOrder={(order) => setSelectedSortOrder(order)}
                  />
                </p>
              </div>
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
            <div class="level-item">
              <div class="field">
                <p class="control">
                  <button
                    classList={{
                      button: true,
                      'is-primary': submittable()
                    }}
                    disabled={!submittable()}
                    on:click={(_) => setModalOpen(true)}
                  >
                    Confirm
                  </button>
                </p>
              </div>
            </div>
          </div>
        </nav>

        <TagsSetter
          selectedAssets={selectedAssets}
          addedTags={addedTags}
          addTag={addTag}
          removedTags={removedTags}
          removeTag={removeTag}
        />

        <LocationSetter
          setPlace={setPlace}
          setCity={setCity}
          setRegion={setRegion}
        />

        <div class="field is-horizontal">
          <div class="field-label is-normal">
            <label class="label" for="date-input">
              Set Date
            </label>
          </div>
          <div class="field-body">
            <div class="field">
              <p class="control">
                <input
                  class="input"
                  id="date-input"
                  type="datetime-local"
                  value={
                    datetime()
                      ? new Date(datetime()).toISOString().slice(0, 16)
                      : ''
                  }
                  on:change={(event) => {
                    if (event.currentTarget.value) {
                      setDatetime(event.currentTarget.value + 'Z');
                      // can set date-time or set days but not both
                      setDays(0);
                    } else {
                      setDatetime('');
                    }
                  }}
                />
              </p>
              <p class="help">Set a specific date-time</p>
            </div>
            <div class="field">
              <p class="control">
                <input
                  class="input"
                  id="days-input"
                  type="number"
                  value={days()}
                  on:change={(event) => {
                    setDays(Number.parseInt(event.currentTarget.value));
                    // can set date-time or set days but not both
                    setDatetime('');
                  }}
                />
              </p>
              <p class="help">Add or subtract days</p>
            </div>
          </div>
        </div>
      </div>

      <div
        classList={{
          modal: true,
          'is-active': modalOpen()
        }}
      >
        <div class="modal-background"></div>
        <div class="modal-card">
          <ConfirmDialog
            assetIds={selectedAssets()}
            setModalOpen={setModalOpen}
            addedTags={addedTags()}
            removedTags={removedTags()}
            place={place()}
            city={city()}
            region={region()}
            datetime={datetime()}
            days={days()}
          />
        </div>
      </div>

      <Suspense fallback={<button class="button is-loading">...</button>}>
        <CardsGrid
          results={assetsQuery()?.scan.results}
          selectedAssets={selectedAssets}
          onClick={cardOnClick}
        />
      </Suspense>
    </>
  );
}

const ALL_TAGS: TypedDocumentNode<Query, Record<string, never>> = gql`
  query {
    tags {
      label
      count
    }
  }
`;

const TAGS_FOR_ASSETS: TypedDocumentNode<Query, QueryTagsForAssetsArgs> = gql`
  query GetAssetTags($assets: [String!]!) {
    tagsForAssets(assets: $assets) {
      label
      count
    }
  }
`;

interface TagsSetterProps {
  selectedAssets: Accessor<Set<string>>;
  addedTags: Accessor<Set<string>>;
  addTag: (label: string) => void;
  removedTags: Accessor<Set<string>>;
  removeTag: (label: string) => void;
}

function TagsSetter(props: TagsSetterProps) {
  const client = useApolloClient();
  const [tagsQuery] = createResource(async () => {
    const { data } = await client.query({ query: ALL_TAGS });
    return data;
  });
  const sortedTags = createMemo(() => {
    return Array.from(tagsQuery()?.tags ?? []).sort((a, b) =>
      a.label.localeCompare(b.label)
    );
  });
  const [assetTagsQuery] = createResource(
    props.selectedAssets,
    async (selections) => {
      const { data } = await client.query({
        query: TAGS_FOR_ASSETS,
        variables: { assets: Array.from(selections) }
      });
      return data;
    }
  );
  const numAssets = createMemo(() => props.selectedAssets().size);
  const assetTags = createMemo(() => {
    const assetCount = numAssets();
    // all of the tags in the selected assets
    let tags = assetTagsQuery()?.tagsForAssets ?? [];
    // add in the new tags to be assigned to all selected assets; like any other
    // signal of a collection, cannot modify elements internally
    const added = props.addedTags();
    tags = tags.filter((elem) => !added.has(elem.label));
    for (const added of props.addedTags()) {
      tags.push({ label: added, count: assetCount });
    }
    // filter out any tags that are to be removed from the assets
    const removed = props.removedTags();
    return tags.filter((elem) => !removed.has(elem.label));
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
        props.addTag(value);
        target.value = '';
      }
      event.stopPropagation();
    }
  };
  const tagOnClick = (attr: AttributeCount) => {
    if (attr.count < numAssets()) {
      props.addTag(attr.label);
    }
  };

  return (
    <div class="field is-horizontal">
      <div class="field-label is-normal">
        <label class="label" for="set-tags-input">
          Set Tags
        </label>
      </div>
      <div class="field-body">
        <div class="field">
          <div class="field is-grouped">
            <p class="control">
              <input
                class="input"
                type="text"
                id="set-tags-input"
                list="add-tag-labels"
                placeholder="Choose tags"
                on:change={onChange}
              />
              <datalist id="add-tag-labels">
                <For each={sortedTags()}>
                  {(place) => <option value={place.label}></option>}
                </For>
              </datalist>
            </p>
            <div class="field is-grouped is-grouped-multiline">
              <Suspense fallback={<></>}>
                <For each={assetTags()}>
                  {(element) => (
                    <div class="control">
                      <div class="tags has-addons">
                        <a
                          classList={{
                            tag: true,
                            'is-success': element.count === numAssets()
                          }}
                          on:click={(_) => tagOnClick(element)}
                        >
                          {element.label}
                        </a>
                        <a
                          class="tag is-delete"
                          on:click={(_) => props.removeTag(element.label)}
                        ></a>
                      </div>
                    </div>
                  )}
                </For>
              </Suspense>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

const ALL_LOCATION_VALUES: TypedDocumentNode<
  Query,
  Record<string, never>
> = gql`
  query {
    locationValues {
      labels
      cities
      regions
    }
  }
`;

interface LocationSetterProps {
  setPlace: (value: string) => void;
  setCity: (value: string) => void;
  setRegion: (value: string) => void;
}

function LocationSetter(props: LocationSetterProps) {
  const client = useApolloClient();
  const [locationsQuery] = createResource(async () => {
    const { data } = await client.query({ query: ALL_LOCATION_VALUES });
    return data;
  });
  const sortedPlaces = createMemo(() => {
    return Array.from(locationsQuery()?.locationValues.labels ?? []).sort();
  });
  const sortedCities = createMemo(() => {
    return Array.from(locationsQuery()?.locationValues.cities ?? []).sort();
  });
  const sortedRegions = createMemo(() => {
    return Array.from(locationsQuery()?.locationValues.regions ?? []).sort();
  });

  return (
    <div class="field is-horizontal">
      <div class="field-label is-normal">
        <label class="label" for="place-input">
          Set Location
        </label>
      </div>
      <div class="field-body">
        <div class="field">
          <p class="control">
            <input
              class="input"
              type="text"
              id="place-input"
              list="place-labels"
              placeholder="Place"
              on:change={(event) => {
                const target = event.currentTarget;
                if (target) {
                  props.setPlace(target.value);
                  event.stopPropagation();
                }
              }}
            />
            <datalist id="place-labels">
              <For each={sortedPlaces()}>
                {(place) => <option value={place}></option>}
              </For>
            </datalist>
          </p>
          <p class="help">
            Enter <code>nihil</code> to clear this field.
          </p>
        </div>
        <div class="field">
          <p class="control">
            <input
              class="input"
              type="text"
              id="city-input"
              list="city-labels"
              placeholder="City"
              on:change={(event) => {
                const target = event.currentTarget;
                if (target) {
                  props.setCity(target.value);
                  event.stopPropagation();
                }
              }}
            />
            <datalist id="city-labels">
              <For each={sortedCities()}>
                {(city) => <option value={city}></option>}
              </For>
            </datalist>
          </p>
          <p class="help">
            Enter <code>nihil</code> to clear this field.
          </p>
        </div>
        <div class="field">
          <p class="control">
            <input
              class="input"
              type="text"
              id="region-input"
              list="region-labels"
              placeholder="Region"
              on:change={(event) => {
                const target = event.currentTarget;
                if (target) {
                  props.setRegion(target.value);
                  event.stopPropagation();
                }
              }}
            />
            <datalist id="region-labels">
              <For each={sortedRegions()}>
                {(region) => <option value={region}></option>}
              </For>
            </datalist>
          </p>
          <p class="help">
            Enter <code>nihil</code> to clear this field.
          </p>
        </div>
      </div>
    </div>
  );
}

interface SortOrderSelectorProps {
  selectedSortOrder: Accessor<SortOrder>;
  setSortOrder: (order: SortOrder) => void;
}

function SortOrderSelector(props: SortOrderSelectorProps) {
  return (
    <Switch fallback={'...'}>
      <Match when={props.selectedSortOrder() === SortOrder.Descending}>
        <button
          class="button"
          on:click={(_) => {
            props.setSortOrder(SortOrder.Ascending);
          }}
        >
          <span class="icon">
            <i class="fa-solid fa-arrow-up-9-1" aria-hidden="true"></i>
          </span>
        </button>
      </Match>
      <Match when={props.selectedSortOrder() === SortOrder.Ascending}>
        <button
          class="button"
          on:click={(_) => {
            props.setSortOrder(SortOrder.Descending);
          }}
        >
          <span class="icon">
            <i class="fa-solid fa-arrow-down-1-9" aria-hidden="true"></i>
          </span>
        </button>
      </Match>
    </Switch>
  );
}

const EDIT_ASSETS: TypedDocumentNode<Mutation, MutationEditArgs> = gql`
  mutation Edit($assetIds: [String!]!, $operations: AssetEdit!) {
    edit(assetIds: $assetIds, operations: $operations)
  }
`;

interface ConfirmDialogProps {
  assetIds: Set<string>;
  setModalOpen: Setter<boolean>;
  addedTags: Set<string>;
  removedTags: Set<string>;
  place: string;
  city: string;
  region: string;
  datetime: string;
  days: number;
}

function ConfirmDialog(props: ConfirmDialogProps) {
  const client = useApolloClient();
  const commitAction = action(async (): Promise<any> => {
    const locations = [];
    if (props.place) {
      if (props.place === 'nihil') {
        locations.push({ field: LocationField.Label, value: null });
      } else {
        locations.push({ field: LocationField.Label, value: props.place });
      }
    }
    if (props.city) {
      if (props.city === 'nihil') {
        locations.push({ field: LocationField.City, value: null });
      } else {
        locations.push({ field: LocationField.City, value: props.city });
      }
    }
    if (props.region) {
      if (props.region === 'nihil') {
        locations.push({ field: LocationField.Region, value: null });
      } else {
        locations.push({ field: LocationField.Region, value: props.region });
      }
    }
    const params: MutationEditArgs = {
      assetIds: Array.from(props.assetIds),
      operations: {
        addTags: Array.from(props.addedTags),
        removeTags: Array.from(props.removedTags),
        setLocation: locations || null,
        setDate: props.datetime || null,
        addDays: props.days
      }
    };
    try {
      await client.mutate({ mutation: EDIT_ASSETS, variables: params });
    } catch (error) {
      console.error('assets edit failed:', error);
      // force an early exit so the user has a chance to look at the browser
      // console to see the error message
      return { ok: false };
    }
    // SolidJS router is _supposed_ to revalidate the queries on this page, but
    // nothing makes any difference, even calling revalidate() or reload()
    // explicitly does nothing, so just force the page to reload instead.
    window.location.reload();
    return { ok: true };
  });
  const startCommit = useAction(commitAction);
  const commitSubmission = useSubmission(commitAction);

  return (
    <>
      <header class="modal-card-head">
        <p class="modal-card-title">Confirm changes to selected assets</p>
        <button
          class="delete"
          aria-label="close"
          on:click={(_) => {
            props.setModalOpen(false);
          }}
        ></button>
      </header>
      <section class="modal-card-body">
        <Show when={props.addedTags.size > 0} fallback={<></>}>
          <p>
            <strong>Add tags:</strong> {Array.from(props.addedTags).join(', ')}
          </p>
        </Show>
        <Show when={props.removedTags.size > 0} fallback={<></>}>
          <p>
            <strong>Remove tags:</strong>{' '}
            {Array.from(props.removedTags).join(', ')}
          </p>
        </Show>
        <Switch>
          <Match when={props.place === 'nihil'}>
            <p>
              <strong>
                <em>Clear</em> location place
              </strong>
            </p>
          </Match>
          <Match when={props.place.length > 0}>
            <p>
              <strong>Set location place:</strong> {props.place}
            </p>
          </Match>
        </Switch>
        <Switch>
          <Match when={props.city === 'nihil'}>
            <p>
              <strong>
                <em>Clear</em> location city
              </strong>
            </p>
          </Match>
          <Match when={props.city.length > 0}>
            <p>
              <strong>Set location city:</strong> {props.city}
            </p>
          </Match>
        </Switch>
        <Switch>
          <Match when={props.region === 'nihil'}>
            <p>
              <strong>
                <em>Clear</em> location region
              </strong>
            </p>
          </Match>
          <Match when={props.region.length > 0}>
            <p>
              <strong>Set location region:</strong> {props.region}
            </p>
          </Match>
        </Switch>
        <Show when={props.datetime.length > 0} fallback={<></>}>
          <p>
            <strong>Set date-time:</strong>{' '}
            {new Date(props.datetime).toISOString().slice(0, 16)}
          </p>
        </Show>
        <Show when={props.days != 0} fallback={<></>}>
          <p>
            <strong>Add/Subtract days:</strong> {props.days}
          </p>
        </Show>
      </section>
      <footer class="modal-card-foot">
        <div class="buttons">
          <button
            classList={{
              button: true,
              'is-success': true,
              'is-loading': commitSubmission.pending
            }}
            disabled={commitSubmission.pending}
            on:click={(_) => startCommit()}
          >
            Apply
          </button>
          <button class="button" on:click={(_) => props.setModalOpen(false)}>
            Cancel
          </button>
        </div>
      </footer>
    </>
  );
}

export default Edit;
