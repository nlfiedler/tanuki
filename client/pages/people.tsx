//
// Copyright (c) 2026 Nathan Fiedler
//
import {
  createEffect,
  createMemo,
  createResource,
  createSignal,
  For,
  on,
  Show,
  Suspense
} from 'solid-js';
import { useNavigate, useParams } from '@solidjs/router';
import { type TypedDocumentNode, gql } from '@apollo/client';
import { useApolloClient } from '../apollo-provider.tsx';
import type {
  Face,
  Mutation,
  MutationHidePersonArgs,
  MutationMergePeopleArgs,
  MutationReassignFacesArgs,
  MutationRenamePersonArgs,
  MutationSetPersonThumbnailArgs,
  Person,
  Query,
  QueryAssetsByPersonArgs,
  QueryPeopleArgs,
  QueryPersonFacesArgs
} from 'tanuki/generated/graphql.ts';
import Pagination from '../components/pagination.tsx';
import ThumbList from '../components/thumb-list.tsx';
import useLocalStorage from '../hooks/use-local-storage.ts';

const PEOPLE: TypedDocumentNode<Query, QueryPeopleArgs> = gql`
  query People($includeHidden: Boolean) {
    people(includeHidden: $includeHidden) {
      id
      name
      thumbnail
      hidden
      faceCount
    }
  }
`;

const PERSON_FACES: TypedDocumentNode<Query, QueryPersonFacesArgs> = gql`
  query PersonFaces($id: ID!) {
    personFaces(id: $id) {
      id
      assetId
      thumbnail
    }
  }
`;

const ASSETS_BY_PERSON: TypedDocumentNode<Query, QueryAssetsByPersonArgs> = gql`
  query AssetsByPerson($id: ID!, $offset: Int, $limit: Int) {
    assetsByPerson(id: $id, offset: $offset, limit: $limit) {
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

const RENAME_PERSON: TypedDocumentNode<Mutation, MutationRenamePersonArgs> = gql`
  mutation RenamePerson($id: ID!, $name: String) {
    renamePerson(id: $id, name: $name) {
      id
      name
    }
  }
`;

const HIDE_PERSON: TypedDocumentNode<Mutation, MutationHidePersonArgs> = gql`
  mutation HidePerson($id: ID!, $hidden: Boolean!) {
    hidePerson(id: $id, hidden: $hidden) {
      id
      hidden
    }
  }
`;

const REASSIGN_FACES: TypedDocumentNode<Mutation, MutationReassignFacesArgs> =
  gql`
    mutation ReassignFaces($faceIds: [ID!]!, $personId: ID) {
      reassignFaces(faceIds: $faceIds, personId: $personId) {
        id
      }
    }
  `;

const MERGE_PEOPLE: TypedDocumentNode<Mutation, MutationMergePeopleArgs> = gql`
  mutation MergePeople($sourceId: ID!, $targetId: ID!) {
    mergePeople(sourceId: $sourceId, targetId: $targetId) {
      id
    }
  }
`;

const SET_THUMBNAIL: TypedDocumentNode<
  Mutation,
  MutationSetPersonThumbnailArgs
> = gql`
  mutation SetPersonThumbnail($id: ID!, $faceId: ID!) {
    setPersonThumbnail(id: $id, faceId: $faceId) {
      id
      thumbnail
    }
  }
`;

/** Display name for a person, falling back to a placeholder when unnamed. */
function displayName(name: string | null | undefined): string {
  return name && name.length > 0 ? name : 'Unnamed';
}

/**
 * `ref` callback that focuses (and selects) an input once it is in the DOM.
 * Needed because the native `autofocus` attribute only applies to elements
 * present at initial page load, not ones revealed later by `<Show>`.
 */
function focusOnMount(el: HTMLInputElement): void {
  queueMicrotask(() => {
    el.focus();
    el.select();
  });
}

/**
 * Grid of every (non-hidden by default) person, each a tile with a
 * representative face crop, an inline-editable name, and a face count. The
 * thumbnail navigates to the person's detail page.
 */
function People() {
  const client = useApolloClient();
  const navigate = useNavigate();
  const [includeHidden, setIncludeHidden] = createSignal(false);
  // Wrap the source in an object: createResource skips the fetch when the
  // source value is false/null/undefined, and `includeHidden()` is false by
  // default — an object is always truthy, so the query always runs (and
  // re-runs when the toggle changes).
  const [peopleQuery, { refetch }] = createResource(
    () => ({ includeHidden: includeHidden() }),
    async (vars) => {
      const { data } = await client.query({
        query: PEOPLE,
        variables: vars,
        fetchPolicy: 'network-only'
      });
      return data;
    }
  );
  const people = createMemo(() => peopleQuery()?.people ?? []);

  async function rename(id: string, name: string | null) {
    await client.mutate({ mutation: RENAME_PERSON, variables: { id, name } });
    await refetch();
  }

  return (
    <section class="section">
      <div class="container">
        <nav class="level">
          <div class="level-left">
            <div class="level-item">
              <h1 class="title is-4">People</h1>
            </div>
          </div>
          <div class="level-right">
            <label class="checkbox level-item">
              <input
                type="checkbox"
                checked={includeHidden()}
                onChange={(e) => setIncludeHidden(e.currentTarget.checked)}
              />{' '}
              Show hidden
            </label>
          </div>
        </nav>
        <Suspense fallback={<p>Loading people…</p>}>
          <Show
            when={people().length > 0}
            fallback={
              <p class="has-text-grey">
                No people yet — face recognition is still in progress or no
                images with faces have been imported.
              </p>
            }
          >
            <div class="thumb-grid">
              <For each={people()}>
                {(person) => (
                  <PersonTile
                    person={person}
                    onOpen={() => navigate(`/people/${person.id}`)}
                    onRename={(name) => rename(person.id, name)}
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

interface PersonTileProps {
  person: Person;
  onOpen: () => void;
  onRename: (name: string | null) => void;
}

function PersonTile(props: PersonTileProps) {
  const [editing, setEditing] = createSignal(false);
  const [value, setValue] = createSignal('');
  const countText = () =>
    `${props.person.faceCount} ${props.person.faceCount === 1 ? 'face' : 'faces'}`;

  function startEdit(e: MouseEvent) {
    e.stopPropagation();
    setValue(props.person.name ?? '');
    setEditing(true);
  }
  function commit() {
    const next = value().trim();
    setEditing(false);
    if (next !== (props.person.name ?? '')) {
      props.onRename(next.length > 0 ? next : null);
    }
  }

  return (
    <div class="thumb-tile">
      <button
        type="button"
        class="thumb-image-wrap"
        onClick={props.onOpen}
        title="View photos"
      >
        <img
          class="thumb-image"
          src={props.person.thumbnail}
          alt={displayName(props.person.name)}
          loading="lazy"
        />
      </button>
      <div class="thumb-info">
        <Show
          when={editing()}
          fallback={
            <button
              type="button"
              class="thumb-title button is-ghost is-small px-0"
              onClick={startEdit}
              title="Click to rename"
            >
              {displayName(props.person.name)}
              <Show when={props.person.hidden}>
                <span class="tag is-light ml-2">hidden</span>
              </Show>
            </button>
          }
        >
          <input
            class="input is-small"
            value={value()}
            placeholder="Name"
            ref={focusOnMount}
            onInput={(e) => setValue(e.currentTarget.value)}
            onBlur={commit}
            onKeyDown={(e) => {
              if (e.key === 'Enter') commit();
              if (e.key === 'Escape') setEditing(false);
            }}
          />
        </Show>
        <div class="thumb-line">
          <span class="icon">
            <i class="fa-regular fa-user" aria-hidden="true"></i>
          </span>
          <span>{countText()}</span>
        </div>
      </div>
    </div>
  );
}

/**
 * Detail view for one person: their photos (paginated) plus a face-cluster
 * management panel for splitting, merging, hiding, and pinning a thumbnail.
 */
function PersonDetail() {
  const client = useApolloClient();
  const navigate = useNavigate();
  const params = useParams<{ id: string }>();
  const id = () => params.id;

  // Header info comes from the full people list (so hidden persons resolve too).
  const [peopleQuery, peopleCtl] = createResource(async () => {
    const { data } = await client.query({
      query: PEOPLE,
      variables: { includeHidden: true },
      fetchPolicy: 'network-only'
    });
    return data;
  });
  const person = createMemo(() =>
    (peopleQuery()?.people ?? []).find((p) => p.id === id())
  );

  const [facesQuery, facesCtl] = createResource(id, async (personId) => {
    const { data } = await client.query({
      query: PERSON_FACES,
      variables: { id: personId },
      fetchPolicy: 'network-only'
    });
    return data;
  });
  const faces = createMemo(() => facesQuery()?.personFaces ?? []);

  const [selectedPage, setSelectedPage] = createSignal(1);
  const [pageSize, setPageSize] = useLocalStorage('page-size', 18);
  // The component instance is reused across /people/:id navigations and the
  // page survives a page-size change, either of which can leave the offset past
  // the new cluster's last page (e.g. the navigate() after a merge). Reset to
  // page 1 whenever the person or page size changes. `defer` skips the initial
  // run so it doesn't clobber a deep link.
  createEffect(on([id, pageSize], () => setSelectedPage(1), { defer: true }));
  const assetArgs = createMemo(() => ({
    id: id(),
    offset: pageSize() * (selectedPage() - 1),
    limit: pageSize()
  }));
  const [assetsQuery] = createResource(assetArgs, async (args) => {
    const { data } = await client.query({
      query: ASSETS_BY_PERSON,
      variables: args,
      fetchPolicy: 'network-only'
    });
    return data;
  });
  const results = () => assetsQuery()?.assetsByPerson.results ?? [];
  const total = () => assetsQuery()?.assetsByPerson.count ?? 0;
  const lastPage = () => assetsQuery()?.assetsByPerson.lastPage ?? 1;

  const [selected, setSelected] = createSignal<Set<string>>(new Set<string>());
  function toggle(faceId: string) {
    setSelected((prev) => {
      const next = new Set(prev);
      if (next.has(faceId)) next.delete(faceId);
      else next.add(faceId);
      return next;
    });
  }
  const selectedIds = () => [...selected()];

  async function refetchAll() {
    setSelected(new Set<string>());
    await Promise.all([facesCtl.refetch(), peopleCtl.refetch()]);
  }

  async function splitToNew() {
    await client.mutate({
      mutation: REASSIGN_FACES,
      variables: { faceIds: selectedIds(), personId: null }
    });
    await refetchAll();
  }
  async function setThumbnail() {
    const faceId = selectedIds()[0];
    if (!faceId) return;
    await client.mutate({
      mutation: SET_THUMBNAIL,
      variables: { id: id(), faceId }
    });
    await refetchAll();
  }
  async function toggleHidden() {
    await client.mutate({
      mutation: HIDE_PERSON,
      variables: { id: id(), hidden: !(person()?.hidden ?? false) }
    });
    await peopleCtl.refetch();
  }
  async function mergeInto(targetId: string) {
    if (!targetId || targetId === id()) return;
    await client.mutate({
      mutation: MERGE_PEOPLE,
      variables: { sourceId: id(), targetId }
    });
    // this person is gone after a merge; go to the target
    navigate(`/people/${targetId}`);
  }
  async function rename(name: string | null) {
    await client.mutate({
      mutation: RENAME_PERSON,
      variables: { id: id(), name }
    });
    await peopleCtl.refetch();
  }

  const otherPeople = createMemo(() =>
    (peopleQuery()?.people ?? []).filter((p) => p.id !== id())
  );

  return (
    <section class="section">
      <div class="container">
        <Suspense fallback={<p>Loading…</p>}>
          <Show
            when={person()}
            fallback={<p class="has-text-grey">No such person.</p>}
          >
            <PersonHeader
              person={person()!}
              total={total()}
              onRename={rename}
              onToggleHidden={toggleHidden}
            />

            <h2 class="title is-5 mt-5">Photos</h2>
            <nav class="level">
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
            <Show
              when={results().length > 0}
              fallback={<p class="has-text-grey">No photos.</p>}
            >
              <ThumbList
                results={results()}
                onClick={(assetId) => navigate(`/asset/${assetId}`)}
              />
            </Show>

            <h2 class="title is-5 mt-5">
              Faces{' '}
              <span class="has-text-grey-light is-size-6">
                ({faces().length}) — select to split, merge, or set thumbnail
              </span>
            </h2>
            <FaceActions
              selectedCount={selected().size}
              otherPeople={otherPeople()}
              onSplit={splitToNew}
              onSetThumbnail={setThumbnail}
              onMerge={mergeInto}
            />
            <FaceGrid
              faces={faces()}
              selected={selected()}
              onToggle={toggle}
            />
          </Show>
        </Suspense>
      </div>
    </section>
  );
}

interface PersonHeaderProps {
  person: Person;
  total: number;
  onRename: (name: string | null) => void;
  onToggleHidden: () => void;
}

function PersonHeader(props: PersonHeaderProps) {
  const [editing, setEditing] = createSignal(false);
  const [value, setValue] = createSignal('');
  function commit() {
    const next = value().trim();
    setEditing(false);
    if (next !== (props.person.name ?? '')) {
      props.onRename(next.length > 0 ? next : null);
    }
  }
  return (
    <nav class="level">
      <div class="level-left">
        <div class="level-item">
          <Show
            when={editing()}
            fallback={
              <h1
                class="title is-4"
                onClick={() => {
                  setValue(props.person.name ?? '');
                  setEditing(true);
                }}
                title="Click to rename"
              >
                <span class="has-text-grey-light">Person /</span>{' '}
                {displayName(props.person.name)}
              </h1>
            }
          >
            <input
              class="input"
              value={value()}
              placeholder="Name"
              ref={focusOnMount}
              onInput={(e) => setValue(e.currentTarget.value)}
              onBlur={commit}
              onKeyDown={(e) => {
                if (e.key === 'Enter') commit();
                if (e.key === 'Escape') setEditing(false);
              }}
            />
          </Show>
        </div>
        <div class="level-item has-text-grey">
          {props.total} {props.total === 1 ? 'photo' : 'photos'}
        </div>
      </div>
      <div class="level-right">
        <button
          type="button"
          class="button"
          onClick={props.onToggleHidden}
        >
          {props.person.hidden ? 'Unhide' : 'Hide'}
        </button>
      </div>
    </nav>
  );
}

interface FaceActionsProps {
  selectedCount: number;
  otherPeople: Person[];
  onSplit: () => void;
  onSetThumbnail: () => void;
  onMerge: (targetId: string) => void;
}

function FaceActions(props: FaceActionsProps) {
  return (
    <div class="field is-grouped is-grouped-multiline mb-3">
      <p class="control">
        <button
          type="button"
          class="button"
          disabled={props.selectedCount === 0}
          onClick={props.onSplit}
        >
          Split {props.selectedCount} to new person
        </button>
      </p>
      <p class="control">
        <button
          type="button"
          class="button"
          disabled={props.selectedCount !== 1}
          onClick={props.onSetThumbnail}
        >
          Set as thumbnail
        </button>
      </p>
      <p class="control">
        <span class="select">
          <select
            onChange={(e) => {
              props.onMerge(e.currentTarget.value);
              e.currentTarget.value = '';
            }}
          >
            <option value="">Merge this person into…</option>
            <For each={props.otherPeople}>
              {(p) => <option value={p.id}>{displayName(p.name)}</option>}
            </For>
          </select>
        </span>
      </p>
    </div>
  );
}

interface FaceGridProps {
  faces: Face[];
  selected: Set<string>;
  onToggle: (faceId: string) => void;
}

function FaceGrid(props: FaceGridProps) {
  return (
    <div class="face-grid">
      <For each={props.faces}>
        {(face) => (
          <button
            type="button"
            class="face-tile"
            classList={{ 'is-selected': props.selected.has(face.id) }}
            onClick={() => props.onToggle(face.id)}
          >
            <img src={face.thumbnail} alt="face" loading="lazy" />
          </button>
        )}
      </For>
    </div>
  );
}

export { People, PersonDetail };
export default People;
