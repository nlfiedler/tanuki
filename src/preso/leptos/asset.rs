//
// Copyright (c) 2024 Nathan Fiedler
//
use crate::domain::entities::{Asset, AssetInput, Location};
use crate::preso::leptos::nav;
use chrono::{DateTime, Local, NaiveDateTime, Utc};
use leptos::*;
use leptos_router::use_params_map;
use std::collections::HashMap;

///
/// Retrieve an asset by its unique identifier.
///
#[leptos::server]
pub async fn fetch_asset(id: String) -> Result<Asset, ServerFnError> {
    use crate::domain::usecases::fetch::{FetchAsset, Params};
    use crate::domain::usecases::UseCase;

    let repo = super::ssr::db()?;
    let usecase = FetchAsset::new(Box::new(repo));
    let params = Params::new(id);
    let asset = usecase
        .call(params)
        .map_err(|e| leptos::ServerFnErrorErr::WrappedServerError(e))?;
    Ok(asset)
}

///
/// Update the asset with the given values.
///
#[leptos::server]
pub async fn update_asset(asset: AssetInput) -> Result<Option<Asset>, ServerFnError> {
    use crate::domain::usecases::update::{Params, UpdateAsset};
    use crate::domain::usecases::UseCase;

    if asset.has_values() {
        let repo = super::ssr::db()?;
        let usecase = UpdateAsset::new(Box::new(repo));
        let params: Params = Params::new(asset.into());
        let result: Asset = usecase
            .call(params)
            .map_err(|e| leptos::ServerFnErrorErr::WrappedServerError(e))?;
        Ok(Some(result))
    } else {
        Ok(None)
    }
}

#[component]
pub fn AssetPage() -> impl IntoView {
    let params = use_params_map();
    let asset_resource = create_resource(
        move || params.with(|params| params.get("id").cloned()),
        |id| async move { fetch_asset(id.unwrap_or_default().to_owned()).await },
    );

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
                                    .into_view()
                            }
                            Ok(asset) => {
                                view! {
                                    <AssetFigure asset=asset.clone() />
                                    <AssetForm asset />
                                }
                                    .into_view()
                            }
                        })
                }}
            </Transition>
        </div>
    }
}

#[component]
fn AssetFigure(asset: Asset) -> impl IntoView {
    let asset = store_value(asset);
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
                        .into_view()
                } else if asset.get_value().media_type.starts_with("audio/") {
                    let src = format!("/rest/asset/{}", asset.get_value().key);
                    view! {
                        <figcaption>{move || asset.get_value().filename}</figcaption>
                        <audio controls>
                            <source src=src type=asset.get_value().media_type />
                        </audio>
                    }
                        .into_view()
                } else {
                    view! {
                        <img
                            src=src
                            alt=asset.get_value().filename
                            style="max-width: 100%; width: auto; padding: inherit; margin: auto; display: block;"
                        />
                    }
                        .into_view()
                }
            }}
        </figure>
    }
}

#[component]
fn AssetForm(asset: Asset) -> impl IntoView {
    let asset = store_value(asset);
    let datetime_input_ref: NodeRef<html::Input> = create_node_ref();
    let filename_input_ref: NodeRef<html::Input> = create_node_ref();
    let caption_input_ref: NodeRef<html::Input> = create_node_ref();
    let tags_input_ref: NodeRef<html::Input> = create_node_ref();
    let mediatype_input_ref: NodeRef<html::Input> = create_node_ref();
    let location_input_ref: NodeRef<html::Input> = create_node_ref();
    let city_input_ref: NodeRef<html::Input> = create_node_ref();
    let region_input_ref: NodeRef<html::Input> = create_node_ref();
    let save_action = create_action(move |_input: &()| {
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
    let upload_action = create_action(move |ev: &leptos::ev::Event| {
        let selected = file_event_to_file_vec(ev.clone());
        upload_file(asset.get_value().key, selected[0].clone())
    });

    view! {
        <nav class="m-4 level">
            <div class="level-left">
                <div class="level-item">
                    <p class="control">
                        <div class="file">
                            <label class="file-label">
                                <input
                                    class="file-input"
                                    type="file"
                                    id="file-input"
                                    name="replacement"
                                    disabled=move || upload_action.pending().get()
                                    on:input=move |ev| upload_action.dispatch(ev)
                                />
                                <span class="file-cta">
                                    <span class="file-icon">
                                        <i class="fas fa-upload"></i>
                                    </span>
                                    <span class="file-label">Replace</span>
                                </span>
                            </label>
                        </div>
                    </p>
                </div>
                <div class="level-item">
                    <p class="control">
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
                    </p>
                </div>
            </div>
            <div class="level-right">
                <div class="level-item">
                    <p class="control">
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
                                                    on:click=move |_| save_action.dispatch(())
                                                >
                                                    Save
                                                </button>
                                            }
                                        }
                                    >
                                        <button
                                            class="button is-success"
                                            on:click=move |_| save_action.dispatch(())
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
                    </p>
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

/// Convert the FileList from the given event into a Vec<File> type.
fn file_event_to_file_vec(ev: leptos::ev::Event) -> Vec<web_sys::File> {
    use web_sys::wasm_bindgen::JsCast;
    let files = ev
        .target()
        .unwrap()
        .unchecked_ref::<web_sys::HtmlInputElement>()
        .files()
        .unwrap();
    let mut results: Vec<web_sys::File> = Vec::new();
    for idx in 0..files.length() {
        results.push(files.item(idx).unwrap());
    }
    results
}

/// Upload a file, then navigate to the details page for the replacement asset.
async fn upload_file(key: String, file: web_sys::File) {
    let form_data = web_sys::FormData::new().unwrap();
    let filename = file.name();
    form_data
        .set_with_blob_and_filename("asset", &file, &filename)
        .unwrap();
    let url = format!("/rest/replace/{}", key);
    let result = gloo::net::http::Request::post(&url)
        .body(form_data)
        .unwrap()
        .send()
        .await;
    match result {
        Err(err) => log::error!("file upload failed: {err:#?}"),
        Ok(res) => {
            // expected body {"ids:", ["new-id"]} with exactly one entry
            let results: HashMap<String, Vec<String>> = res.json().await.unwrap();
            let url = format!("/asset/{}", results["ids"][0]);
            let navigate = leptos_router::use_navigate();
            let mut options = leptos_router::NavigateOptions::default();
            options.replace = true;
            navigate(&url, options);
        }
    }
}
