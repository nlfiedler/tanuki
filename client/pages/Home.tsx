//
// Copyright (c) 2025 Nathan Fiedler
//
import {
  type Accessor,
  createMemo,
  createResource,
  createSignal,
  For,
  type JSX,
  Match,
  Suspense,
  Switch,
} from 'solid-js'
import { useNavigate } from '@solidjs/router'
import { type TypedDocumentNode, gql } from '@apollo/client'
import { useApolloClient } from '../ApolloProvider'
import {
  type QuerySearchArgs,
  type Query,
  SortField,
  SortOrder,
} from 'tanuki/generated/graphql.ts'
import AttributeChips from '../components/AttributeChips.tsx'
import CardsGrid from '../components/CardsGrid.tsx'
import Pagination from '../components/Pagination.tsx'
import TagSelector from '../components/TagSelector.tsx'
import useClickOutside from '../hooks/useClickOutside.js'

const SEARCH_ASSETS: TypedDocumentNode<Query, QuerySearchArgs> = gql`
  query Search($params: SearchParams!, $offset: Int, $limit: Int) {
    search(params: $params, offset: $offset, limit: $limit) {
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
      }
      count
      lastPage
    }
  }
`

function buildParams({
  tags,
  locations,
  year,
  season,
  mediaType,
  offset,
  limit,
  sortOrder,
}: {
  tags: string[]
  locations: string[]
  year: number | null
  season: Season | null
  mediaType: string | null
  offset: number
  limit: number
  sortOrder: SortOrder
}): QuerySearchArgs {
  let before = undefined
  let after = undefined
  if (year && season) {
    switch (season) {
      case Season.Winter:
        after = new Date(year, 1, 1, 0, 0)
        before = new Date(year, 4, 1, 0, 0)
        break
      case Season.Spring:
        after = new Date(year, 4, 1, 0, 0)
        before = new Date(year, 7, 1, 0, 0)
        break
      case Season.Summer:
        after = new Date(year, 7, 1, 0, 0)
        before = new Date(year, 10, 1, 0, 0)
        break
      case Season.Fall:
        after = new Date(year, 10, 1, 0, 0)
        before = new Date(year + 1, 1, 1, 0, 0)
        break
    }
  } else if (year) {
    after = new Date(year, 1, 1, 0, 0)
    before = new Date(year + 1, 1, 1, 0, 0)
  } else if (tags.length === 0 && locations.length === 0) {
    // if not searching by tags or locations, then show all assets
    before = new Date(275760, 8, 12)
  }
  return {
    params: {
      tags,
      locations,
      before,
      after,
      mediaType,
      sortField: SortField.Date,
      sortOrder,
    },
    offset,
    limit,
  }
}

function Home() {
  const navigate = useNavigate()
  const client = useApolloClient()
  const [selectedTags, setSelectedTags] = createSignal<string[]>([])
  const [selectedLocations, setSelectedLocations] = createSignal<string[]>([])
  const [selectedYear, setSelectedYear] = createSignal<number | null>(null)
  const [selectedSeason, setSelectedSeason] = createSignal<Season | null>(null)
  const [selectedMediaType, setSelectedMediaType] = createSignal<string | null>(
    null
  )
  const [selectedSortOrder, setSelectedSortOrder] = createSignal<SortOrder>(
    SortOrder.Descending
  )
  const [selectedPage, setSelectedPage] = createSignal(1)
  const [pageSize, setPageSize] = createSignal(18)
  const pendingParams = createMemo(() => ({
    tags: selectedTags(),
    locations: selectedLocations(),
    year: selectedYear(),
    season: selectedSeason(),
    mediaType: selectedMediaType(),
    offset: pageSize() * (selectedPage() - 1),
    limit: pageSize(),
    sortOrder: selectedSortOrder(),
  }))
  const [assetsQuery] = createResource(pendingParams, async (params) => {
    const { data } = await client.query({
      query: SEARCH_ASSETS,
      variables: buildParams(params),
    })
    return data
  })
  const lastPage = () => assetsQuery()?.search.lastPage ?? 1

  return (
    <>
      <div class="container">
        <nav class="level">
          <div class="level-left">
            <div class="level-item">
              <TagSelector
                addfun={(value) => {
                  setSelectedTags((tags) => {
                    if (tags.indexOf(value) < 0) {
                      // return a new array so SolidJS will take note
                      return [...tags, value]
                    }
                    return tags
                  })
                  setSelectedPage(1)
                }}
              />
            </div>
            <div class="level-item">
              <LocationSelector
                addfun={(value) => {
                  setSelectedLocations((locations) => {
                    if (locations.indexOf(value) < 0) {
                      // return a new array so SolidJS will take note
                      return [...locations, value]
                    }
                    return locations
                  })
                  setSelectedPage(1)
                }}
              />
            </div>
            <div class="level-item">
              <YearSelector
                selectedYear={selectedYear}
                setyear={(year) => {
                  setSelectedYear(year)
                  setSelectedPage(1)
                }}
              />
            </div>
            <div class="level-item">
              <SeasonSelector
                selectedSeason={selectedSeason}
                setSeason={(season) => {
                  if (selectedYear() === null) {
                    setSelectedYear(new Date().getFullYear())
                  }
                  setSelectedSeason(season)
                  setSelectedPage(1)
                }}
              />
            </div>
            <div class="level-item">
              <MediaTypeSelector
                selectedMediaType={selectedMediaType}
                setMediaType={(mediaType) => {
                  setSelectedMediaType(mediaType)
                  setSelectedPage(1)
                }}
              />
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

      <div class="container mt-3 mb-3">
        <div class="field is-grouped is-grouped-multiline">
          <AttributeChips
            attrs={selectedTags}
            rmfun={(attr) => {
              setSelectedTags((tags) => {
                // return a new array so SolidJS will take note
                return tags.filter((t) => t !== attr)
              })
            }}
          />
          <AttributeChips
            attrs={selectedLocations}
            rmfun={(attr) => {
              setSelectedLocations((locations) => {
                // return a new array so SolidJS will take note
                return locations.filter((l) => l !== attr)
              })
            }}
          />
        </div>
      </div>

      <Suspense fallback={<button class="button is-loading">...</button>}>
        <CardsGrid
          results={assetsQuery()?.search.results}
          onClick={(assetId) => navigate(`/asset/${assetId}`)}
        />
      </Suspense>
    </>
  )
}

const ALL_LOCATION_PARTS: TypedDocumentNode<Query, Record<string, never>> = gql`
  query {
    locationParts {
      label
      count
    }
  }
`

interface LocationSelectorProps {
  addfun: (value: string) => void
}

function LocationSelector(props: LocationSelectorProps) {
  const client = useApolloClient()
  const [locationsQuery] = createResource(async () => {
    const { data } = await client.query({ query: ALL_LOCATION_PARTS })
    return data
  })
  const sortedLocations = () => {
    // the locations returned from the server are in no particular order
    const sorted = new Array()
    for (let location of locationsQuery()?.locationParts ?? []) {
      sorted.push({ label: location.label, count: location.count })
    }
    sorted.sort((a, b) => a.label.localeCompare(b.label))
    return sorted
  }
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
    const target = event.currentTarget
    if (target) {
      const value = target.value
      if (value) {
        props.addfun(value)
        target.value = ''
      }
      event.stopPropagation()
    }
  }

  return (
    <Suspense fallback={'...'}>
      <div class="field is-horizontal">
        <div class="field-label is-normal">
          <label class="label" for="locations-input">
            Locations
          </label>
        </div>
        <div class="field-body">
          <p class="control">
            <input
              class="input"
              type="text"
              id="locations-input"
              list="location-labels"
              placeholder="Choose locations"
              on:change={onChange}
            />
            <datalist id="location-labels">
              <For each={sortedLocations()}>
                {(location) => <option value={location.label}></option>}
              </For>
            </datalist>
          </p>
        </div>
      </div>
    </Suspense>
  )
}

const ALL_YEARS: TypedDocumentNode<Query, Record<string, never>> = gql`
  query {
    years {
      label
      count
    }
  }
`

class YearAttribute {
  year: number
  count: number

  constructor(year: number | string, count: number) {
    if (typeof year === 'string') {
      this.year = parseInt(year) || -1
    } else {
      this.year = year
    }
    this.count = count
  }
}

interface YearSelectorProps {
  selectedYear: Accessor<number | null>
  setyear: (value: number | null) => void
}

function YearSelector(props: YearSelectorProps) {
  const [dropdownOpen, setDropdownOpen] = createSignal(false)
  let dropdownRef: HTMLDivElement | undefined
  useClickOutside(
    () => dropdownRef,
    () => setDropdownOpen(false)
  )
  const client = useApolloClient()
  const [yearsQuery] = createResource(async () => {
    const { data } = await client.query({ query: ALL_YEARS })
    return data
  })
  const sortedYears = () => {
    // the years returned from the server are in no particular order
    const sorted = new Array<YearAttribute>()
    for (let year of yearsQuery()?.years ?? []) {
      sorted.push(new YearAttribute(year.label, year.count))
    }
    // inject the current year if not already present so that the season
    // selection has something to select when year is unset
    //
    // do this before sorting since there may be assets marked as being from the
    // future
    const currentYear = new Date().getFullYear()
    const hasCurrentYear = sorted.some((entry) => entry.year === currentYear)
    if (!hasCurrentYear) {
      sorted.push(new YearAttribute(currentYear, 0))
    }
    // sort in reverse chronological order for selection convenience (most
    // recent years near the top of the dropdown menu)
    sorted.sort((a, b) => b.year - a.year)
    return sorted
  }

  return (
    <Suspense fallback={'...'}>
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
            aria-controls="dropdown-menu"
          >
            {props.selectedYear() ?? 'Year'}
          </button>
        </div>
        <div class="dropdown-menu" id="dropdown-menu" role="menu">
          <div class="dropdown-content">
            <a
              class="dropdown-item"
              on:click={(_) => {
                props.setyear(null)
                setDropdownOpen(false)
              }}
            >
              Any
            </a>
            <For each={sortedYears()}>
              {(year) => (
                <a
                  class="dropdown-item"
                  on:click={(_) => {
                    props.setyear(year.year)
                    setDropdownOpen(false)
                  }}
                >
                  {year.year.toString()}
                </a>
              )}
            </For>
          </div>
        </div>
      </div>
    </Suspense>
  )
}

enum Season {
  Winter = 1,
  Spring,
  Summer,
  Fall,
}

function labelForSeason(season: Season | null): string {
  switch (season) {
    case Season.Winter:
      return 'Jan-Mar'
    case Season.Spring:
      return 'Apr-Jun'
    case Season.Summer:
      return 'Jul-Sep'
    case Season.Fall:
      return 'Oct-Dec'
    default:
      return 'Season'
  }
}

interface SeasonSelectorProps {
  selectedSeason: Accessor<Season | null>
  setSeason: (value: Season | null) => void
}

function SeasonSelector(props: SeasonSelectorProps) {
  const [dropdownOpen, setDropdownOpen] = createSignal(false)
  let dropdownRef: HTMLDivElement | undefined
  useClickOutside(
    () => dropdownRef,
    () => setDropdownOpen(false)
  )

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
          aria-controls="season-menu"
        >
          {labelForSeason(props.selectedSeason())}
        </button>
      </div>
      <div class="dropdown-menu" id="season-menu" role="menu">
        <div class="dropdown-content">
          <a
            class="dropdown-item"
            on:click={(_) => {
              props.setSeason(null)
              setDropdownOpen(false)
            }}
          >
            Any
          </a>
          <a
            class="dropdown-item"
            on:click={(_) => {
              props.setSeason(Season.Winter)
              setDropdownOpen(false)
            }}
          >
            {labelForSeason(Season.Winter)}
          </a>
          <a
            class="dropdown-item"
            on:click={(_) => {
              props.setSeason(Season.Spring)
              setDropdownOpen(false)
            }}
          >
            {labelForSeason(Season.Spring)}
          </a>
          <a
            class="dropdown-item"
            on:click={(_) => {
              props.setSeason(Season.Summer)
              setDropdownOpen(false)
            }}
          >
            {labelForSeason(Season.Summer)}
          </a>
          <a
            class="dropdown-item"
            on:click={(_) => {
              props.setSeason(Season.Fall)
              setDropdownOpen(false)
            }}
          >
            {labelForSeason(Season.Fall)}
          </a>
        </div>
      </div>
    </div>
  )
}

const ALL_MEDIA_TYPES: TypedDocumentNode<Query, Record<string, never>> = gql`
  query {
    mediaTypes {
      label
      count
    }
  }
`

interface MediaTypeSelectorProps {
  selectedMediaType: Accessor<string | null>
  setMediaType: (value: string | null) => void
}

function MediaTypeSelector(props: MediaTypeSelectorProps) {
  const [dropdownOpen, setDropdownOpen] = createSignal(false)
  let dropdownRef: HTMLDivElement | undefined
  useClickOutside(
    () => dropdownRef,
    () => setDropdownOpen(false)
  )
  const client = useApolloClient()
  const [mediaTypesQuery] = createResource(async () => {
    const { data } = await client.query({ query: ALL_MEDIA_TYPES })
    return data
  })
  const sortedMediaTypes = () => {
    // the media types returned from the server are in no particular order
    const sorted = new Array()
    for (let mediaType of mediaTypesQuery()?.mediaTypes ?? []) {
      sorted.push({ label: mediaType.label, count: mediaType.count })
    }
    sorted.sort((a, b) => a.label.localeCompare(b.label))
    return sorted
  }

  return (
    <Suspense fallback={'...'}>
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
            aria-controls="dropdown-menu"
          >
            {props.selectedMediaType() ?? 'Media Type'}
          </button>
        </div>
        <div class="dropdown-menu" id="dropdown-menu" role="menu">
          <div class="dropdown-content">
            <a
              class="dropdown-item"
              on:click={(_) => {
                props.setMediaType(null)
                setDropdownOpen(false)
              }}
            >
              Any
            </a>
            <For each={sortedMediaTypes()}>
              {(mediaType) => (
                <a
                  class="dropdown-item"
                  on:click={(_) => {
                    props.setMediaType(mediaType.label)
                    setDropdownOpen(false)
                  }}
                >
                  {mediaType.label}
                </a>
              )}
            </For>
          </div>
        </div>
      </div>
    </Suspense>
  )
}

interface SortOrderSelectorProps {
  selectedSortOrder: Accessor<SortOrder>
  setSortOrder: (order: SortOrder) => void
}

function SortOrderSelector(props: SortOrderSelectorProps) {
  return (
    <Switch fallback={'...'}>
      <Match when={props.selectedSortOrder() === SortOrder.Descending}>
        <button
          class="button"
          on:click={(_) => {
            props.setSortOrder(SortOrder.Ascending)
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
            props.setSortOrder(SortOrder.Descending)
          }}
        >
          <span class="icon">
            <i class="fa-solid fa-arrow-down-1-9" aria-hidden="true"></i>
          </span>
        </button>
      </Match>
    </Switch>
  )
}

export default Home
