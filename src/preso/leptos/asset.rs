//
// Copyright (c) 2024 Nathan Fiedler
//
use crate::domain::entities::{Asset, AssetInput, Location};
use crate::preso::leptos::nav;
use crate::preso::leptos::BrowseParams;
use chrono::{DateTime, Local, NaiveDateTime, Utc};
use codee::string::JsonSerdeCodec;
use leptos::html::Input;
use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use leptos_use::storage::{use_local_storage_with_options, UseStorageOptions};

///
/// Retrieve an asset by its unique identifier.
///
#[leptos::server]
async fn fetch_asset(id: String) -> Result<Asset, ServerFnError> {
    use crate::domain::usecases::fetch::{FetchAsset, Params};
    use crate::domain::usecases::UseCase;
    use server_fn::error::ServerFnErrorErr;

    let repo = super::ssr::db()?;
    let usecase = FetchAsset::new(Box::new(repo));
    let params = Params::new(id);
    let asset = usecase
        .call(params)
        .map_err(|e| ServerFnErrorErr::ServerError(e.to_string()))?;
    Ok(asset)
}

///
/// Update the asset with the given values.
///
#[leptos::server]
async fn update_asset(asset: AssetInput) -> Result<Option<Asset>, ServerFnError> {
    use crate::domain::usecases::update::{Params, UpdateAsset};
    use crate::domain::usecases::UseCase;
    use server_fn::error::ServerFnErrorErr;

    if asset.has_values() {
        let repo = super::ssr::db()?;
        let cache = super::ssr::cache()?;
        let usecase = UpdateAsset::new(Box::new(repo), Box::new(cache));
        let params: Params = Params::new(asset.into());
        let result: Asset = usecase
            .call(params)
            .map_err(|e| ServerFnErrorErr::ServerError(e.to_string()))?;
        Ok(Some(result))
    } else {
        Ok(None)
    }
}

///
/// Upload a replacement file, returning the identifier of the new asset.
///
/// The same identifier will be returned if an identical file already exists,
/// otherwise returns the new identifier of the changed asset.
///
#[leptos::server(input = server_fn::codec::MultipartFormData)]
pub async fn replace_asset(data: server_fn::codec::MultipartData) -> Result<String, ServerFnError> {
    use crate::data::repositories::geo::find_location_repository;
    use crate::data::repositories::SearchRepositoryImpl;
    use crate::domain::usecases::replace::{Params, ReplaceAsset};
    use crate::domain::usecases::UseCase;
    use chrono::TimeZone;
    use leptos::server_fn::error::ServerFnErrorErr;
    use std::io::Write;
    use std::str::FromStr;
    use std::sync::Arc;
    // `.into_inner()` returns the inner `multer` stream; it is `None` if called
    // from the client, but always `Some(_)` on the server, so it is safe to
    // unwrap from within a server function
    let mut data = data.into_inner().unwrap();

    let mut params = Params {
        asset_id: String::new(),
        filepath: std::path::PathBuf::new(),
        media_type: mime::APPLICATION_OCTET_STREAM,
        last_modified: None,
    };
    // expected fields:
    // - asset_id: original asset identifier
    // - file_blob: file content in chunks
    // - last_modified: date/time of the file
    while let Ok(Some(mut field)) = data.next_field().await {
        let field_name = field.name().unwrap_or_default().to_string();
        if field_name == "asset_id" {
            params.asset_id = field
                .text()
                .await
                .map_err(|e| ServerFnErrorErr::ServerError(e.to_string()))?;
        } else if field_name == "file_blob" {
            if let Some(content_type) = field.content_type() {
                params.media_type = content_type.to_owned();
            }
            let filename = field.file_name().unwrap_or("unknown");
            let (filepath, mut f) = super::ssr::create_upload_file(filename)?;
            params.filepath = filepath;
            while let Ok(Some(chunk)) = field.chunk().await {
                f.write_all(&chunk)?;
            }
        } else if field_name == "last_modified" {
            let text = field
                .text()
                .await
                .map_err(|e| ServerFnErrorErr::ServerError(e.to_string()))?;
            let float =
                f64::from_str(&text).map_err(|e| ServerFnErrorErr::ServerError(e.to_string()))?;
            params.last_modified = chrono::Utc.timestamp_millis_opt(float as i64).single();
        }
    }

    // prepare and invoke ReplaceAsset usecase, return identifier
    let records = Arc::new(super::ssr::db()?);
    let blobs = Arc::new(super::ssr::blobs()?);
    let geocoder = find_location_repository();
    let cache = Arc::new(SearchRepositoryImpl::new());
    let usecase = ReplaceAsset::new(records, cache, blobs, geocoder);
    let asset = usecase
        .call(params)
        .map_err(|e| ServerFnErrorErr::ServerError(e.to_string()))?;
    Ok(asset.key)
}

#[component]
pub fn AssetPage() -> impl IntoView {
    let params = use_params_map();
    let asset_resource = Resource::new(
        move || params.with(|params| params.get("id")),
        |id| async move { fetch_asset(id.unwrap_or_default().to_owned()).await },
    );
    let asset_replaced = Action::new(move |id: &String| {
        let id = id.to_owned();
        async move {
            let navigate = leptos_router::hooks::use_navigate();
            let url = format!("/asset/{}", id);
            navigate(&url, Default::default());
        }
    });

    view! {
        <nav::NavBar />

        <div class="container">
            <Transition fallback=move || {
                view! { "Loading asset details..." }
            }>
                {move || {
                    asset_resource
                        .get()
                        .map(|result| match result {
                            Err(err) => {
                                view! { <span>{move || format!("Error: {}", err)}</span> }
                                    .into_any()
                            }
                            Ok(asset) => {
                                view! {
                                    <AssetFigure asset=asset.clone() />
                                    <AssetForm
                                        asset=asset
                                        replaced=move |id| {
                                            asset_replaced.dispatch(id);
                                        }
                                    />
                                }
                                    .into_any()
                            }
                        })
                }}
            </Transition>
        </div>
    }
}

#[component]
pub fn BrowsePage() -> impl IntoView {
    let (browse_params, set_browse_params, _) =
        use_local_storage_with_options::<BrowseParams, JsonSerdeCodec>(
            "browse-params",
            UseStorageOptions::default()
                .initial_value(BrowseParams::default())
                .delay_during_hydration(true),
        );
    let browse_resource = Resource::new(
        move || browse_params.get(),
        |params| async move { super::browse(params).await },
    );
    let asset_replaced = Action::new(move |id: &String| {
        // when an asset is replaced, the cached search results are cleared, and
        // the position of the new asset within the updated results may change;
        // refresh the search results, find the new position of the asset, and
        // restart the browsing process
        let id = id.to_owned();
        async move {
            let params = browse_params.get_untracked();
            if let Ok(Some(index)) = super::browse_replace(params, id).await {
                let params = browse_params.get_untracked();
                if index == params.asset_index {
                    // setting the params field to the same value will have no
                    // effect, so refresh the page instead; this may cause the
                    // local storage signal to revert momentarily, so the view
                    // below will handle this by displaying nothing in place of
                    // the missing asset
                    let window = web_sys::window().unwrap();
                    let document = window.document().unwrap();
                    let location = document.location().unwrap();
                    if let Err(err) = location.reload() {
                        log::error!("page reload failed: {err:#?}");
                    }
                } else {
                    // if the index has in fact changed, then setting that in
                    // the browse params signal will cause the page to refresh
                    set_browse_params.update(|p| p.asset_index = index);
                }
            } else {
                // on the other hand, the replaced asset may disappear from the
                // search results, such as when a png becomes a jpg for a search
                // that was specifically for image/png
                let params = browse_params.get_untracked();
                if params.asset_index > 0 {
                    set_browse_params.update(|p| p.asset_index -= 1);
                } else {
                    let navigate = leptos_router::hooks::use_navigate();
                    navigate("/", Default::default());
                }
            }
        }
    });

    view! {
        <nav::NavBar />

        <div class="container">
            <Transition fallback=move || {
                view! { "Loading asset details..." }
            }>
                {move || {
                    browse_resource
                        .get()
                        .map(|result| match result {
                            Err(err) => {
                                view! { <span>{move || format!("Error: {}", err)}</span> }
                                    .into_any()
                            }
                            Ok(meta) => {
                                let asset = StoredValue::new(meta.asset);
                                view! {
                                    <div class="columns">
                                        <div
                                            class="px-0 column is-narrow"
                                            style="display: flex; min-height: 90vh; max-height: 90vh; align-items: center;"
                                        >
                                            <Show
                                                when=move || { browse_params.get().asset_index > 0 }
                                                fallback=move || {
                                                    view! {
                                                        <button class="button" disabled>
                                                            <span class="icon">
                                                                <i class="fa-solid fa-arrow-left"></i>
                                                            </span>
                                                        </button>
                                                    }
                                                }
                                            >
                                                <button
                                                    class="button"
                                                    on:click=move |_| {
                                                        set_browse_params.update(|p| p.asset_index -= 1)
                                                    }
                                                >
                                                    <span class="icon">
                                                        <i class="fa-solid fa-arrow-left"></i>
                                                    </span>
                                                </button>
                                            </Show>
                                        </div>
                                        <div class="column">
                                            <Show
                                                when=move || asset.with_value(|a| a.is_some())
                                                fallback=move || {}
                                            >
                                                <AssetFigure asset=asset.get_value().unwrap() />
                                                <AssetForm
                                                    asset=asset.get_value().unwrap()
                                                    replaced=move |id| {
                                                        asset_replaced.dispatch(id);
                                                    }
                                                />
                                            </Show>
                                        </div>
                                        <div
                                            class="px-0 column is-narrow"
                                            style="display: flex; min-height: 90vh; max-height: 90vh; align-items: center;"
                                        >
                                            <Show
                                                when=move || {
                                                    meta.last_index > browse_params.get().asset_index
                                                }
                                                fallback=move || {
                                                    view! {
                                                        <button class="button" disabled>
                                                            <span class="icon">
                                                                <i class="fa-solid fa-arrow-right"></i>
                                                            </span>
                                                        </button>
                                                    }
                                                }
                                            >
                                                <button
                                                    class="button"
                                                    on:click=move |_| {
                                                        set_browse_params.update(|p| p.asset_index += 1)
                                                    }
                                                >
                                                    <span class="icon">
                                                        <i class="fa-solid fa-arrow-right"></i>
                                                    </span>
                                                </button>
                                            </Show>
                                        </div>
                                    </div>
                                }
                                    .into_any()
                            }
                        })
                }}
            </Transition>
        </div>
    }
}

#[component]
fn AssetFigure(asset: Asset) -> impl IntoView {
    let asset = StoredValue::new(asset);
    view! {
        <figure class="image">
            {move || {
                let src = format!("/rest/asset/{}", asset.get_value().key);
                if asset.get_value().media_type.starts_with("video/") {
                    let mut media_type = asset.get_value().media_type;
                    if media_type == "video/quicktime" {
                        media_type = "video/mp4".into();
                    }
                    view! {
                        <video controls>
                            <source src=src type=media_type />
                            Bummer, your browser does not support the HTML5
                            <code>video</code>
                            tag.
                        </video>
                    }
                        .into_any()
                } else if asset.get_value().media_type.starts_with("audio/") {
                    let src = format!("/rest/asset/{}", asset.get_value().key);
                    view! {
                        <figcaption>{move || asset.get_value().filename}</figcaption>
                        <audio controls>
                            <source src=src type=asset.get_value().media_type />
                        </audio>
                    }
                        .into_any()
                } else {
                    view! {
                        <img
                            src=src
                            alt=asset.get_value().filename
                            style="max-width: 100%; width: auto; padding: inherit; margin: auto; display: block;"
                        />
                    }
                        .into_any()
                }
            }}
        </figure>
    }
}

#[component]
fn AssetForm<E>(asset: Asset, replaced: E) -> impl IntoView
where
    E: Fn(String) + Copy + 'static + Send,
{
    let asset = StoredValue::new(asset);
    let datetime_input_ref: NodeRef<Input> = NodeRef::new();
    let filename_input_ref: NodeRef<Input> = NodeRef::new();
    let caption_input_ref: NodeRef<Input> = NodeRef::new();
    let tags_input_ref: NodeRef<Input> = NodeRef::new();
    let mediatype_input_ref: NodeRef<Input> = NodeRef::new();
    let location_input_ref: NodeRef<Input> = NodeRef::new();
    let city_input_ref: NodeRef<Input> = NodeRef::new();
    let region_input_ref: NodeRef<Input> = NodeRef::new();
    let save_action = Action::new(move |_input: &()| {
        let mut input = AssetInput::new(asset.get_value().key.clone());
        // caption
        let new_caption = caption_input_ref.get().unwrap().value();
        if asset
            .get_value()
            .caption
            .map(|c| c != new_caption)
            .unwrap_or(new_caption.len() > 0)
        {
            // new or different value
            input.caption = Some(new_caption);
        }
        // filename
        let new_filename = filename_input_ref.get().unwrap().value();
        if asset.get_value().filename != new_filename {
            input.filename = Some(new_filename);
        }
        // tags: split on commas, trim, filter empty
        let raw_tags_text = tags_input_ref.get().unwrap().value();
        let mut tags: Vec<String> = raw_tags_text
            .split(',')
            .map(|e| e.trim())
            .filter(|e| !e.is_empty())
            .map(|e| e.to_owned())
            .collect();
        tags.sort();
        let mut old_tags: Vec<String> = asset.get_value().tags.clone();
        old_tags.sort();
        let matching = tags
            .iter()
            .zip(old_tags.iter())
            .filter(|&(tags, old_tags)| tags == old_tags)
            .count();
        if matching != tags.len() || matching != old_tags.len() {
            input.tags = Some(tags);
        }
        // datetime: convert from local to UTC
        let local = chrono::offset::Local::now();
        let datetime_str = format!(
            "{}{}",
            datetime_input_ref.get().unwrap().value(),
            local.offset().to_string()
        );
        // Despite formatting the asset datetime with seconds into the input
        // field, sometimes the browser does not display or return the seconds,
        // so we must be flexible here.
        let pattern = if datetime_str.len() == 22 {
            "%Y-%m-%dT%H:%M%z"
        } else {
            "%Y-%m-%dT%H:%M:%S%z"
        };
        match DateTime::parse_from_str(&datetime_str, pattern) {
            Ok(datetime) => {
                if asset.get_value().best_date() != datetime {
                    input.datetime = Some(datetime.to_utc());
                }
            }
            Err(err) => log::error!(
                "datetime parse error: {:?}; input: {}, format: {}",
                err,
                datetime_str,
                pattern
            ),
        }
        // location (label, city, region)
        let mut location = Location::default();
        let location_str = location_input_ref.get().unwrap().value();
        location.label = if location_str.len() > 0 {
            Some(location_str)
        } else {
            None
        };
        let city_str = city_input_ref.get().unwrap().value();
        location.city = if city_str.len() > 0 {
            Some(city_str)
        } else {
            None
        };
        let region_str = region_input_ref.get().unwrap().value();
        location.region = if region_str.len() > 0 {
            Some(region_str)
        } else {
            None
        };
        if asset
            .get_value()
            .location
            .map(|l| l != location)
            .unwrap_or(location.has_values())
        {
            // new or different value
            input.location = Some(location);
        }
        // media_type
        let new_mediatype = mediatype_input_ref.get().unwrap().value();
        if asset.get_value().media_type != new_mediatype {
            input.media_type = Some(new_mediatype);
        }
        update_asset(input)
    });
    let save_pending = save_action.pending();
    let save_result = save_action.value();
    // convert the Option<Result<Option<Asset>>> into a simple bool
    let save_success = move || {
        save_result
            .get()
            .map(|r| r.map(|o| o.is_some()).unwrap_or(false))
            .unwrap_or(false)
    };
    // upload replacement file asynchronously and show updated asset
    let (asset_unchanged, set_asset_unchanged) = signal(false);
    let upload_action = Action::new_local(move |data: &web_sys::FormData| {
        let fd = data.clone();
        let old_asset_id = asset.get_value().key;
        async move {
            // `MultipartData` implements `From<FormData>`
            let new_asset_id = replace_asset(fd.into()).await.expect("upload file");
            if old_asset_id != new_asset_id {
                replaced(new_asset_id)
            } else {
                set_asset_unchanged.set(true);
                set_timeout(
                    move || {
                        set_asset_unchanged.set(false);
                    },
                    std::time::Duration::from_secs(5),
                );
            }
        }
    });

    view! {
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
                                multiple=false
                                disabled=move || upload_action.pending().get()
                                on:input=move |ev| {
                                    ev.prevent_default();
                                    let asset_id = asset.get_value().key;
                                    let form_data = file_event_to_form_data(&ev, &asset_id)
                                        .expect("file event to form data");
                                    upload_action.dispatch_local(form_data);
                                }
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
                        href=format!("/rest/asset/{}", asset.get_value().key)
                        download=asset.get_value().filename
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
                    <Show
                        when=move || save_pending.get()
                        fallback=move || {
                            view! {
                                <Show
                                    when=move || save_success()
                                    fallback=move || {
                                        view! {
                                            <button
                                                class="button"
                                                on:click=move |_| {
                                                    save_action.dispatch(());
                                                }
                                            >
                                                <span class="icon">
                                                    <i class="fa-solid fa-floppy-disk"></i>
                                                </span>
                                                <span>Save</span>
                                            </button>
                                        }
                                    }
                                >
                                    <button
                                        class="button is-success"
                                        on:click=move |_| {
                                            save_action.dispatch(());
                                        }
                                    >
                                        <span class="icon is-small">
                                            <i class="fas fa-check"></i>
                                        </span>
                                        <span>Save</span>
                                    </button>
                                </Show>
                            }
                        }
                    >
                        <button class="button is-loading">Save</button>
                    </Show>
                </div>
            </div>
        </nav>
        <div class="notification is-warning" class:is-hidden=move || !asset_unchanged.get()>
            <button class="delete" on:click=move |_| set_asset_unchanged.set(false)></button>
            The replacement asset is identical to the original,
            please choose a different file.
        </div>
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
                                node_ref=datetime_input_ref
                                value=convert_utc_to_local(asset.get_value().best_date())
                                    .format("%Y-%m-%dT%H:%M:%S")
                                    .to_string()
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
                                node_ref=filename_input_ref
                                value=asset.get_value().filename
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
                                    node_ref=caption_input_ref
                                    placeholder="Description with #tags and @location"
                                    value=asset.get_value().caption.unwrap_or_default()
                                />
                                <span class="icon is-small is-left">
                                    <i class="fa-solid fa-quote-left"></i>
                                </span>
                            </p>
                        </div>
                        <p class="help">The @location can be in quotes if needed.</p>
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
                                    node_ref=tags_input_ref
                                    placeholder="List of tags separated by commas."
                                    value=asset.get_value().tags.join(", ")
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
                                node_ref=location_input_ref
                                placeholder="Description"
                                value=asset
                                    .get_value()
                                    .location
                                    .map(|l| l.label)
                                    .unwrap_or_default()
                            />
                        </p>
                    </div>
                    <div class="field">
                        <p class="control is-expanded">
                            <input
                                class="input"
                                type="text"
                                id="city-input"
                                node_ref=city_input_ref
                                placeholder="City"
                                value=asset.get_value().location.map(|l| l.city).unwrap_or_default()
                            />
                        </p>
                    </div>
                    <div class="field">
                        <p class="control is-expanded">
                            <input
                                class="input"
                                type="text"
                                id="region-input"
                                node_ref=region_input_ref
                                placeholder="Region"
                                value=asset
                                    .get_value()
                                    .location
                                    .map(|l| l.region)
                                    .unwrap_or_default()
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
                                value=asset.get_value().byte_length
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
                                node_ref=mediatype_input_ref
                                value=asset.get_value().media_type
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
                                value=asset.get_value().filepath()
                            />
                        </p>
                    </div>
                </div>
            </div>
        </div>
    }
}

/// Convert a DateTime<Utc> to a NaiveDateTime for the local timezone.
fn convert_utc_to_local(datetime: DateTime<Utc>) -> NaiveDateTime {
    // this is quite complicated for some reason
    let local_now = Local::now();
    let naive_utc = datetime.naive_utc();
    let datetime_local =
        DateTime::<Local>::from_naive_utc_and_offset(naive_utc, local_now.offset().clone());
    datetime_local.naive_local()
}

// Convert an HtmlInputElement event for a file selector into a FormData that
// can be sent to the server via a server function. This is a separate function
// because 1) that's a good thing in general, and 2) leptos-fmt wipes out all
// comments in markup.
fn file_event_to_form_data(
    ev: &web_sys::Event,
    asset_id: &str,
) -> Result<web_sys::FormData, Error> {
    use web_sys::wasm_bindgen::JsCast;
    let files = ev
        .target()
        .unwrap()
        .unchecked_ref::<web_sys::HtmlInputElement>()
        .files()
        .unwrap();
    // assume there is exactly one file in the list
    let file: web_sys::File = files.item(0).unwrap();
    let form_data = web_sys::FormData::new().unwrap();
    let filename = file.name();
    form_data
        .set_with_blob_and_filename("file_blob", &file, &filename)
        .unwrap();
    let filedate = file.last_modified().to_string();
    // apps like Photos will set the file modified time to match the information
    // it has when exporting the asset
    form_data.set_with_str("last_modified", &filedate).unwrap();
    form_data.set_with_str("asset_id", asset_id).unwrap();
    Ok(form_data)
}
