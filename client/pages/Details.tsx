//
// Copyright (c) 2025 Nathan Fiedler
//
import {
  createMemo,
  createRenderEffect,
  createResource,
  createSignal,
  type Accessor,
  Match,
  type Signal,
  Show,
  Suspense,
  Switch
} from 'solid-js';
import { useParams } from '@solidjs/router';
import { action, useAction, useSubmission } from '@solidjs/router';
import { type TypedDocumentNode, gql } from '@apollo/client';
import { useApolloClient } from '../apollo-provider';
import type {
  Asset,
  Mutation,
  MutationUpdateArgs,
  QueryAssetArgs,
  Query
} from 'tanuki/generated/graphql.ts';

const GET_ASSET: TypedDocumentNode<Query, QueryAssetArgs> = gql`
  query Asset($id: ID!) {
    asset(id: $id) {
      id
      checksum
      filename
      filepath
      byteLength
      datetime
      mediaType
      tags
      caption
      location {
        label
        city
        region
      }
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

function AssetDetails() {
  const params = useParams();
  const client = useApolloClient();
  const [assetQuery] = createResource(params, async (params) => {
    const { data } = await client.query({
      query: GET_ASSET,
      variables: { id: params.id! }
    });
    return data;
  });

  return (
    <Suspense fallback={<button class="button is-loading">...</button>}>
      <div>
        <Show when={assetQuery()} fallback="Loading...">
          <AssetFigure asset={assetQuery()?.asset!} />
          <AssetForm asset={assetQuery()?.asset!} />
        </Show>
      </div>
    </Suspense>
  );
}

interface AssetFigureProps {
  asset: Asset;
}

function AssetFigure(props: AssetFigureProps) {
  return (
    <figure class="image">
      <Switch fallback={<ImageThumbnail asset={props.asset} />}>
        <Match when={props.asset.mediaType.startsWith('video/')}>
          <VideoThumbnail asset={props.asset} />
        </Match>
        <Match when={props.asset.mediaType.startsWith('audio/')}>
          <AudioThumbnail asset={props.asset} />
        </Match>
      </Switch>
    </figure>
  );
}

interface ImageThumbnailProps {
  asset: Asset;
}

function ImageThumbnail(props: ImageThumbnailProps) {
  return (
    <img
      src={`/assets/raw/${props.asset.id}`}
      alt={props.asset.filename}
      style="max-width: 100%; width: auto; padding: inherit; margin: auto; display: block;"
    />
  );
}

interface VideoThumbnailProps {
  asset: Asset;
}

function VideoThumbnail(props: VideoThumbnailProps) {
  let media_type = props.asset.mediaType;
  if (media_type == 'video/quicktime') {
    media_type = 'video/mp4';
  }
  return (
    <video controls>
      <source src={`/assets/raw/${props.asset.id}`} type={media_type} />
      Bummer, your browser does not support the HTML5
      <code>video</code>
      tag.
    </video>
  );
}

interface AudioThumbnailProps {
  asset: Asset;
}

function AudioThumbnail(props: AudioThumbnailProps) {
  return (
    <>
      <figcaption>{props.asset.filename}</figcaption>
      <audio controls>
        <source
          src={`/assets/raw/${props.asset.id}`}
          type={props.asset.mediaType}
        />
      </audio>
    </>
  );
}

interface AssetFormProps {
  asset: Asset;
}

// define a directive to make text input handling more concise
function textField(element: HTMLInputElement, value: Accessor<Signal<string>>) {
  const [field, setField] = value();
  createRenderEffect(() => (element.value = field()));
  element.addEventListener('input', ({ target }) =>
    setField((target as HTMLInputElement).value)
  );
}

// define a directive to make date-time input handling more concise
//
// n.b. the datetime from the Asset type in GraphQL is a string
function datetimeField(
  element: HTMLInputElement,
  value: Accessor<Signal<string>>
) {
  const [field, setField] = value();
  createRenderEffect(() => {
    // datetime-local input needs the value in a specific format
    element.value = new Date(field()).toISOString().slice(0, 16);
  });
  element.addEventListener('input', ({ target }) =>
    setField((target as HTMLInputElement).value)
  );
}

// define a directive to make text input (for tags) handling more concise
function tagsField(
  element: HTMLInputElement,
  value: Accessor<Signal<string[]>>
) {
  const [field, setField] = value();
  createRenderEffect(() => (element.value = field().join(', ')));
  element.addEventListener('input', ({ target }) => {
    const value = (target as HTMLInputElement).value;
    const tags = value
      .split(',')
      .map((e: string) => e.trim())
      .filter((e: string) => e.length > 0);
    setField(tags);
  });
}

// Patch in the types for the custom directives to satisfy TypeScript.
declare module 'solid-js' {
  namespace JSX {
    interface DirectiveFunctions {
      datetimeField: typeof datetimeField;
      tagsField: typeof tagsField;
      textField: typeof textField;
    }
  }
}

function AssetForm(props: AssetFormProps) {
  const client = useApolloClient();
  const [datetime, setDatetime] = createSignal(props.asset.datetime);
  const [filename, setFilename] = createSignal(props.asset.filename);
  const [caption, setCaption] = createSignal(props.asset.caption ?? '');
  const [tags, setTags] = createSignal(props.asset.tags);
  const [locationLabel, setLocationLabel] = createSignal(
    props.asset.location?.label ?? ''
  );
  const [locationCity, setLocationCity] = createSignal(
    props.asset.location?.city ?? ''
  );
  const [locationRegion, setLocationRegion] = createSignal(
    props.asset.location?.region ?? ''
  );
  const [mediaType, setMediaType] = createSignal(props.asset.mediaType);
  const updateAction = action(async (): Promise<{ ok: boolean }> => {
    const location = {
      label: locationLabel(),
      city: locationCity(),
      region: locationRegion()
    };
    const datetimeDate = datetime() ? new Date(datetime()) : null;
    try {
      await client.mutate({
        mutation: UPDATE_ASSET,
        variables: {
          id: props.asset.id,
          asset: {
            datetime: datetimeDate,
            tags: tags(),
            caption: caption(),
            location,
            mediaType: mediaType(),
            filename: filename()
          }
        }
      });
    } catch (error) {
      console.error('asset update failed:', error);
      return { ok: false };
    }
    return { ok: true };
  }, 'updateAssets');
  const startUpdate = useAction(updateAction);
  const updateSubmission = useSubmission(updateAction);
  const saveButtonClass = createMemo(() => {
    if (updateSubmission.pending) {
      return 'button is-loading';
    } else if (updateSubmission.result?.ok == false) {
      return 'button is-danger';
    } else if (updateSubmission.result?.ok) {
      return 'button is-success';
    }
    return 'button is-primary';
  });

  // TODO: make Replace a separate component that takes the assetId as a prop
  // TODO: add the notification when replacing the asset and it is unchanged

  return (
    <>
      <nav class="m-4 level">
        <div class="level-left">
          <div class="level-item">
            <div class="file">
              <label class="file-label">
                <input
                  class="file-input"
                  type="file"
                  id="file-input"
                  name="file_blob"
                  multiple={false}
                  disabled={true}
                />
                <span class="file-cta">
                  <span class="file-icon">
                    <i class="fas fa-upload"></i>
                  </span>
                  <span class="file-label">Replace</span>
                </span>
              </label>
            </div>
          </div>
          <div class="level-item">
            <a
              href={`/assets/raw/${props.asset.id}`}
              download={props.asset.filename}
            >
              <button class="button">
                <span class="icon">
                  <i class="fa-solid fa-download"></i>
                </span>
                <span>Download</span>
              </button>
            </a>
          </div>
        </div>
        <div class="level-right">
          <div class="level-item">
            <button
              class={saveButtonClass()}
              type="submit"
              value="Save"
              disabled={updateSubmission.pending}
              on:click={(_) => startUpdate()}
            >
              <span class="icon">
                <i class="fa-solid fa-floppy-disk"></i>
              </span>
              <span>Save</span>
            </button>
          </div>
        </div>
      </nav>
      <div class="m-4">
        <div class="mb-2 field is-horizontal">
          <div class="field-label is-normal">
            <label class="label" for="datetime-input">
              Date
            </label>
          </div>
          <div class="field-body">
            <div class="field">
              <p class="control is-expanded has-icons-left">
                <input
                  class="input"
                  type="datetime-local"
                  id="datetime-input"
                  use:datetimeField={[datetime, setDatetime]}
                />
                <span class="icon is-small is-left">
                  <i class="fa-regular fa-calendar"></i>
                </span>
              </p>
            </div>
            <div class="field">
              <p class="control is-expanded has-icons-left">
                <input
                  class="input"
                  type="text"
                  id="filename-input"
                  use:textField={[filename, setFilename]}
                />
                <span class="icon is-small is-left">
                  <i class="fa-regular fa-file"></i>
                </span>
              </p>
            </div>
          </div>
        </div>

        <div class="field is-horizontal">
          <div class="field-label is-normal">
            <label class="label" for="caption-input">
              Caption
            </label>
          </div>
          <div class="field-body">
            <div class="field is-expanded">
              <div class="field">
                <p class="control is-expanded has-icons-left">
                  <input
                    class="input"
                    type="text"
                    id="caption-input"
                    placeholder="Description with #tags and @location"
                    use:textField={[caption, setCaption]}
                  />
                  <span class="icon is-small is-left">
                    <i class="fa-solid fa-quote-left"></i>
                  </span>
                </p>
              </div>
              <p class="help">
                The @location can be in quotes if needed. Tags here are merged
                with those below. A @location here is ignored if the fields
                below are populated.
              </p>
            </div>
          </div>
        </div>

        <div class="mb-2 field is-horizontal">
          <div class="field-label is-normal">
            <label class="label" for="tags-input">
              Tags
            </label>
          </div>
          <div class="field-body">
            <div class="field is-expanded">
              <div class="field">
                <p class="control is-expanded has-icons-left">
                  <input
                    class="input"
                    type="text"
                    id="tags-input"
                    placeholder="List of tags separated by commas."
                    use:tagsField={[tags, setTags]}
                  />
                  <span class="icon is-small is-left">
                    <i class="fa-solid fa-tags"></i>
                  </span>
                </p>
              </div>
            </div>
          </div>
        </div>

        <div class="mb-2 field is-horizontal">
          <div class="field-label is-normal">
            <label class="label" for="location-input">
              Location
            </label>
          </div>
          <div class="field-body">
            <div class="field">
              <p class="control is-expanded">
                <input
                  class="input"
                  type="text"
                  id="location-input"
                  placeholder="Description"
                  use:textField={[locationLabel, setLocationLabel]}
                />
              </p>
            </div>
            <div class="field">
              <p class="control is-expanded">
                <input
                  class="input"
                  type="text"
                  id="city-input"
                  placeholder="City"
                  use:textField={[locationCity, setLocationCity]}
                />
              </p>
            </div>
            <div class="field">
              <p class="control is-expanded">
                <input
                  class="input"
                  type="text"
                  id="region-input"
                  placeholder="Region"
                  use:textField={[locationRegion, setLocationRegion]}
                />
              </p>
            </div>
          </div>
        </div>

        <div class="mb-2 field is-horizontal">
          <div class="field-label is-normal">
            <label class="label" for="filesize-input">
              File Size
            </label>
          </div>
          <div class="field-body">
            <div class="field">
              <p class="control is-expanded">
                <input
                  class="input"
                  type="number"
                  id="filesize-input"
                  readonly
                  value={props.asset.byteLength}
                />
              </p>
            </div>
          </div>
        </div>

        <div class="mb-2 field is-horizontal">
          <div class="field-label is-normal">
            <label class="label" for="mediatype-input">
              Media Type
            </label>
          </div>
          <div class="field-body">
            <div class="field">
              <p class="control is-expanded">
                <input
                  class="input"
                  type="text"
                  id="mediatype-input"
                  use:textField={[mediaType, setMediaType]}
                />
              </p>
            </div>
          </div>
        </div>

        <div class="mb-2 field is-horizontal">
          <div class="field-label is-normal">
            <label class="label" for="path-input">
              Asset Path
            </label>
          </div>
          <div class="field-body">
            <div class="field">
              <p class="control is-expanded">
                <input
                  class="input"
                  type="text"
                  id="path-input"
                  readonly
                  value={props.asset.filepath}
                />
              </p>
            </div>
          </div>
        </div>
      </div>
    </>
  );
}

export default AssetDetails;
