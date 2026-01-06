//
// Copyright (c) 2025 Nathan Fiedler
//
import {
  createMemo,
  createResource,
  createSignal,
  type Accessor,
  type JSX,
  Match,
  Suspense,
  Switch
} from 'solid-js';
import { useNavigate } from '@solidjs/router';
import { type TypedDocumentNode, gql } from '@apollo/client';
import { useApolloClient } from '../apollo-provider.tsx';
import {
  type QueryScanArgs,
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

function Search() {
  const navigate = useNavigate();
  const client = useApolloClient();
  const [queryString, setQueryString] = useLocalStorage<string>(
    'search-query',
    ''
  );
  const [selectedSortOrder, setSelectedSortOrder] = useLocalStorage<SortOrder>(
    'search-sort-order',
    SortOrder.Descending
  );
  const [selectedPage, setSelectedPage] = useLocalStorage(
    'search-selected-page',
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
  const onChange: JSX.EventHandlerWithOptionsUnion<
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
  const [selectedAssets] = createSignal(new Set<string>());

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
                      on:change={onChange}
                    />
                  </p>
                </div>
              </div>
            </div>
          </div>

          <div class="level-right">
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
          </div>
        </nav>
      </div>

      <Suspense fallback={<button class="button is-loading">...</button>}>
        <CardsGrid
          results={assetsQuery()?.scan.results}
          selectedAssets={selectedAssets}
          onClick={(assetId) => navigate(`/asset/${assetId}`)}
        />
      </Suspense>
    </>
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

export default Search;
