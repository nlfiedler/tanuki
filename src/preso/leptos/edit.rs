//
// Copyright (c) 2024 Nathan Fiedler
//
use crate::domain::entities::*;
use crate::preso::leptos::{forms, nav, paging};
use crate::preso::leptos::{BulkEditParams, SearchMeta};
use chrono::{DateTime, Utc};
use codee::string::{FromToStringCodec, JsonSerdeCodec};
use leptos::ev::Event;
use leptos::html::Input;
use leptos::*;
use leptos_use::storage::{use_local_storage_with_options, UseStorageOptions};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LocationData {
    labels: Vec<String>,
    cities: Vec<String>,
    regions: Vec<String>,
}

///
/// Retrieve all of the unique location parts as separate lists (labels, cities,
/// and regions each in their own list).
///
#[leptos::server]
pub async fn fetch_location_parts() -> Result<LocationData, ServerFnError> {
    use crate::domain::usecases::location::CompleteLocations;
    use crate::domain::usecases::{NoParams, UseCase};

    let repo = super::ssr::db()?;
    let usecase = CompleteLocations::new(Box::new(repo));
    let locations: Vec<Location> = usecase
        .call(NoParams {})
        .map_err(|e| leptos::ServerFnErrorErr::WrappedServerError(e))?;
    let mut data = LocationData {
        labels: vec![],
        cities: vec![],
        regions: vec![],
    };
    for loc in locations.into_iter() {
        if let Some(label) = loc.label {
            data.labels.push(label);
        }
        if let Some(city) = loc.city {
            data.cities.push(city);
        }
        if let Some(region) = loc.region {
            data.regions.push(region);
        }
    }
    data.labels.sort();
    data.cities.sort();
    data.regions.sort();
    data.labels.dedup();
    data.cities.dedup();
    data.regions.dedup();
    Ok(data)
}

///
/// Perform one or more operations on multiple assets.
///
#[leptos::server(BulkEdit, "/api", "Cbor")]
pub async fn bulk_edit(ops: BulkEditParams) -> Result<u64, ServerFnError> {
    use crate::domain::usecases::edit::{EditAssets, Params};
    use crate::domain::usecases::UseCase;

    let repo = super::ssr::db()?;
    let cache = super::ssr::cache()?;
    let usecase = EditAssets::new(Box::new(repo), Box::new(cache));
    let params = Params {
        assets: ops.assets,
        tag_ops: ops.tag_ops,
        location_ops: ops.location_ops,
        datetime_op: ops.datetime_op,
    };
    let count = usecase
        .call(params)
        .map_err(|e| leptos::ServerFnErrorErr::WrappedServerError(e))?;
    Ok(count)
}

#[component]
pub fn EditPage() -> impl IntoView {
    let (query, set_query, _) = use_local_storage_with_options::<String, JsonSerdeCodec>(
        "edit-query",
        UseStorageOptions::default()
            .initial_value(String::new())
            .delay_during_hydration(true),
    );
    let (sort_order, set_sort_order, _) = use_local_storage_with_options::<String, JsonSerdeCodec>(
        "edit-sort-order",
        UseStorageOptions::default()
            .initial_value("descending")
            .delay_during_hydration(true),
    );
    // page of results to be displayed (1-based)
    let (selected_page, set_selected_page, _) =
        use_local_storage_with_options::<i32, FromToStringCodec>(
            "edit-selected-page",
            UseStorageOptions::default()
                .initial_value(1)
                .delay_during_hydration(true),
        );
    // number of assets to display in a single page of results
    // let page_size = create_rw_signal(18);
    let (page_size, set_page_size, _) = use_local_storage_with_options::<i32, FromToStringCodec>(
        "page-size",
        UseStorageOptions::default()
            .initial_value(18)
            .delay_during_hydration(true),
    );
    let input_ref = NodeRef::<Input>::new();
    let on_change = move |ev: Event| {
        let input = input_ref.get().unwrap();
        ev.stop_propagation();
        set_query.set(input.value());
    };
    // search for assets using the given criteria
    let results = create_resource(
        move || {
            (
                query.get(),
                sort_order.get(),
                selected_page.get(),
                page_size.get(),
            )
        },
        |(query_str, order, page, count)| async move {
            let offset = count * (page - 1);
            let sort_order = SortOrder::from(order.as_str());
            super::scan_assets(
                query_str,
                Some(SortField::Date),
                Some(sort_order),
                Some(count),
                Some(offset),
            )
            .await
        },
    );
    let selected_assets = create_rw_signal::<HashSet<String>>(HashSet::new());
    let submittable = create_memo(move |_| selected_assets.with(|coll| coll.len() > 0));
    let (modal_active, set_modal_active) = create_signal(false);
    let submit = create_action(move |ops: &BulkEditParams| {
        let mut ops = ops.to_owned();
        ops.assets = selected_assets.get().into_iter().collect();
        apply_changes(ops)
    });
    let select_all = create_action(move |ids: &HashSet<String>| {
        let owned = ids.to_owned();
        async move { selected_assets.set(owned) }
    });
    let unselect_all = create_action(move |_input: &()| async move {
        selected_assets.set(HashSet::new());
    });

    view! {
        <nav::NavBar />
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
                                        value=move || query.get()
                                        node_ref=input_ref
                                        on:change=on_change
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
                                <Show
                                    when=move || sort_order.get() == "ascending"
                                    fallback=move || {
                                        view! {
                                            <button
                                                class="button"
                                                on:click=move |_| { set_sort_order.set("ascending".into()) }
                                            >
                                                <span class="icon">
                                                    <i class="fa-solid fa-arrow-up-9-1" aria-hidden="true"></i>
                                                </span>
                                            </button>
                                        }
                                    }
                                >
                                    <button
                                        class="button"
                                        on:click=move |_| {
                                            set_sort_order.set("descending".into())
                                        }
                                    >
                                        <span class="icon">
                                            <i
                                                class="fa-solid fa-arrow-down-1-9"
                                                aria-hidden="true"
                                            ></i>
                                        </span>
                                    </button>
                                </Show>
                            </p>
                        </div>
                    </div>
                    <Transition fallback=move || {
                        view! { "Loading..." }
                    }>
                        {move || {
                            results
                                .get()
                                .map(|result| match result {
                                    Err(err) => {
                                        view! { <span>{move || format!("Error: {}", err)}</span> }
                                            .into_view()
                                    }
                                    Ok(meta) => {
                                        view! {
                                            <paging::PageControls
                                                meta
                                                selected_page
                                                set_selected_page
                                                page_size
                                                set_page_size
                                            />
                                        }
                                            .into_view()
                                    }
                                })
                        }}
                    </Transition>
                    <div class="level-item">
                        <button class="button" on:click=move |_| unselect_all.dispatch(())>
                            <span class="icon">
                                <i class="fa-regular fa-square"></i>
                            </span>
                        </button>
                    </div>
                    <Transition fallback=move || {
                        view! { "..." }
                    }>
                        {move || {
                            results
                                .get()
                                .map(|result| match result {
                                    Err(_) => view! { <span>ERROR</span> }.into_view(),
                                    Ok(meta) => {
                                        let ids = store_value(
                                            meta
                                                .results
                                                .iter()
                                                .map(|r| r.asset_id.clone())
                                                .collect::<HashSet<String>>(),
                                        );
                                        view! {
                                            <div class="level-item">
                                                <button
                                                    class="button"
                                                    on:click=move |_| select_all.dispatch(ids.get_value())
                                                >
                                                    <span class="icon">
                                                        <i class="fa-regular fa-square-check"></i>
                                                    </span>
                                                </button>
                                            </div>
                                        }
                                            .into_view()
                                    }
                                })
                        }}
                    </Transition>
                    <div class="level-item">
                        <div class="field">
                            <p class="control">
                                <Show
                                    when=move || submittable.get()
                                    fallback=|| {
                                        view! {
                                            <button class="button" disabled>
                                                Modify
                                            </button>
                                        }
                                    }
                                >
                                    <input
                                        class="button"
                                        type="submit"
                                        value="Modify"
                                        on:click=move |_| set_modal_active.set(true)
                                    />
                                </Show>
                            </p>
                        </div>
                    </div>
                </div>
            </nav>
        </div>

        <div class="modal" class:is-active=move || modal_active.get()>
            <div class="modal-background"></div>
            <div class="modal-card">
                <EditForm set_modal_active ops_ready=move |ops| submit.dispatch(ops) />
            </div>
        </div>

        <Transition fallback=move || {
            view! { "Loading..." }
        }>
            {move || {
                results
                    .get()
                    .map(|result| match result {
                        Err(err) => {
                            view! { <span>{move || format!("Error: {}", err)}</span> }.into_view()
                        }
                        Ok(meta) => {
                            view! { <ResultsDisplay meta selected_assets /> }
                        }
                    })
            }}
        </Transition>
    }
}

#[component]
fn EditForm<F>(set_modal_active: WriteSignal<bool>, ops_ready: F) -> impl IntoView
where
    F: Fn(BulkEditParams) + Copy + 'static,
{
    // the locations returned from the server are in no particular order
    let location_parts = create_resource(|| (), |_| async move { fetch_location_parts().await });
    let datetime_input_ref: NodeRef<html::Input> = create_node_ref();
    let location_input_ref: NodeRef<html::Input> = create_node_ref();
    let city_input_ref: NodeRef<html::Input> = create_node_ref();
    let region_input_ref: NodeRef<html::Input> = create_node_ref();
    let del_tags = create_rw_signal::<HashSet<String>>(HashSet::new());
    let add_tags = create_rw_signal::<HashSet<String>>(HashSet::new());
    let build_params = move |_| {
        let mut params = BulkEditParams::default();
        // datetime: convert from local to UTC
        let local = chrono::offset::Local::now();
        let datetime_str = format!(
            "{}{}",
            datetime_input_ref.get().unwrap().value(),
            local.offset().to_string()
        );
        if datetime_str.len() > 6 {
            // If not date/time was entered, then the length of the value will
            // be zero, plus the time zone offset added above.
            //
            // Despite formatting the asset datetime with seconds into the input
            // field, sometimes the browser does not display or return the
            // seconds, so we must be flexible here.
            let pattern = if datetime_str.len() == 22 {
                "%Y-%m-%dT%H:%M%z"
            } else {
                "%Y-%m-%dT%H:%M:%S%z"
            };
            if let Ok(datetime) = DateTime::parse_from_str(&datetime_str, pattern) {
                params.datetime_op = Some(DatetimeOperation::Set(datetime.to_utc()));
            }
        }
        // tags to remove
        for name in del_tags.get().into_iter() {
            params.tag_ops.push(TagOperation::Remove(name));
        }
        // tags to add
        for name in add_tags.get().into_iter() {
            params.tag_ops.push(TagOperation::Add(name));
        }
        // location (label, city, region)
        let location_str = location_input_ref.get().unwrap().value();
        if location_str.len() > 0 {
            params
                .location_ops
                .push(LocationOperation::Set(LocationField::Label, location_str));
        }
        let city_str = city_input_ref.get().unwrap().value();
        if city_str.len() > 0 {
            params
                .location_ops
                .push(LocationOperation::Set(LocationField::City, city_str));
        }
        let region_str = region_input_ref.get().unwrap().value();
        if region_str.len() > 0 {
            params
                .location_ops
                .push(LocationOperation::Set(LocationField::Region, region_str));
        }
        ops_ready(params);
    };

    view! {
        <header class="modal-card-head">
            <p class="modal-card-title">Make changes to the selected assets</p>
            <button
                class="delete"
                aria-label="close"
                on:click=move |_| set_modal_active.set(false)
            ></button>
        </header>
        <section class="modal-card-body">
            <div class="mb-2 field">
                <label class="label" style="text-align: left;" for="datetime-input">
                    Set Date and Time
                </label>
                <div class="field-body">
                    <div class="field">
                        <p class="control is-expanded">
                            <input
                                class="input"
                                type="datetime-local"
                                id="datetime-input"
                                node_ref=datetime_input_ref
                            />
                        </p>
                    </div>
                </div>
            </div>

            <TagsEditForm del_tags add_tags />

            <div class="mb-2 field">
                <Transition fallback=move || {
                    view! { "Loading locations..." }
                }>
                    {move || {
                        location_parts
                            .get()
                            .map(|resp| match resp {
                                Err(err) => {
                                    view! { <span>{move || format!("Error: {}", err)}</span> }
                                        .into_view()
                                }
                                Ok(data) => {
                                    let locations = store_value(data);
                                    view! {
                                        <label
                                            class="label"
                                            style="text-align: left;"
                                            for="location-input"
                                        >
                                            Set Location
                                        </label>
                                        <div class="field-body">
                                            <div class="field">
                                                <p class="control is-expanded">
                                                    <input
                                                        class="input"
                                                        type="text"
                                                        id="location-input"
                                                        list="location-labels"
                                                        node_ref=location_input_ref
                                                        placeholder="Description"
                                                    />
                                                    <datalist id="location-labels">
                                                        {move || {
                                                            locations
                                                                .get_value()
                                                                .labels
                                                                .iter()
                                                                .map(|val| {
                                                                    view! { <option value=val></option> }
                                                                })
                                                                .collect::<Vec<_>>()
                                                        }}
                                                    </datalist>
                                                </p>
                                            </div>
                                            <div class="field">
                                                <p class="control is-expanded">
                                                    <input
                                                        class="input"
                                                        type="text"
                                                        id="cities-input"
                                                        list="location-cities"
                                                        node_ref=city_input_ref
                                                        placeholder="City"
                                                    />
                                                    <datalist id="location-cities">
                                                        {move || {
                                                            locations
                                                                .get_value()
                                                                .cities
                                                                .iter()
                                                                .map(|val| {
                                                                    view! { <option value=val></option> }
                                                                })
                                                                .collect::<Vec<_>>()
                                                        }}
                                                    </datalist>
                                                </p>
                                            </div>
                                            <div class="field">
                                                <p class="control is-expanded">
                                                    <input
                                                        class="input"
                                                        type="text"
                                                        id="regions-input"
                                                        list="location-regions"
                                                        node_ref=region_input_ref
                                                        placeholder="Region"
                                                    />
                                                    <datalist id="location-regions">
                                                        {move || {
                                                            locations
                                                                .get_value()
                                                                .regions
                                                                .iter()
                                                                .map(|val| {
                                                                    view! { <option value=val></option> }
                                                                })
                                                                .collect::<Vec<_>>()
                                                        }}
                                                    </datalist>
                                                </p>
                                            </div>
                                        </div>
                                    }
                                        .into_view()
                                }
                            })
                    }}
                </Transition>
            </div>
        </section>
        <footer class="modal-card-foot">
            <div class="buttons">
                <button class="button is-success" on:click=build_params>
                    Apply
                </button>
                <button class="button" on:click=move |_| set_modal_active.set(false)>
                    Cancel
                </button>
            </div>
        </footer>
    }
}

#[component]
fn TagsEditForm(
    del_tags: RwSignal<HashSet<String>>,
    add_tags: RwSignal<HashSet<String>>,
) -> impl IntoView {
    // the tags returned from the server are in no particular order
    let tags = create_resource(
        || (),
        |_| async move {
            let mut results = super::fetch_tags().await;
            if let Ok(data) = results.as_mut() {
                data.sort_by(|a, b| a.label.cmp(&b.label));
            }
            results
        },
    );
    let del_input_ref = NodeRef::<Input>::new();
    let add_input_ref = NodeRef::<Input>::new();

    view! {
        <Transition fallback=move || {
            view! { "Loading..." }
        }>
            {move || {
                tags.get()
                    .map(|resp| match resp {
                        Err(err) => {
                            view! { <span>{move || format!("Error: {}", err)}</span> }.into_view()
                        }
                        Ok(data) => {
                            let tags = store_value(data);
                            view! {
                                <div class="mb-2 field">
                                    <label
                                        class="label"
                                        style="text-align: left;"
                                        for="del-tags-input"
                                    >
                                        Remove Tags
                                    </label>
                                    <div class="field-body">
                                        <div class="field is-expanded">
                                            <div class="field is-grouped">
                                                <p class="control">
                                                    <input
                                                        class="input"
                                                        type="text"
                                                        id="del-tags-input"
                                                        list="del-tag-labels"
                                                        placeholder="Choose tags"
                                                        node_ref=del_input_ref
                                                        on:change=move |ev: Event| {
                                                            let input = del_input_ref.get().unwrap();
                                                            ev.stop_propagation();
                                                            del_tags
                                                                .update(|tags| {
                                                                    tags.insert(input.value());
                                                                });
                                                            input.set_value("");
                                                        }
                                                    />
                                                    <datalist id="del-tag-labels">
                                                        <For
                                                            each=move || tags.get_value()
                                                            key=|t| t.label.clone()
                                                            let:tag
                                                        >
                                                            <option value=tag.label></option>
                                                        </For>
                                                    </datalist>
                                                </p>
                                                <p class="field is-grouped is-grouped-multiline">
                                                    <forms::TagList
                                                        attrs=del_tags.into()
                                                        rm_attr=move |attr| {
                                                            del_tags
                                                                .update(|coll| {
                                                                    coll.remove(&attr);
                                                                });
                                                        }
                                                    />
                                                </p>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                                <div class="mb-2 field">
                                    <label
                                        class="label"
                                        style="text-align: left;"
                                        for="add_tags-input"
                                    >
                                        Add Tags
                                    </label>
                                    <div class="field-body">
                                        <div class="field is-expanded">
                                            <div class="field is-grouped">
                                                <p class="control">
                                                    <input
                                                        class="input"
                                                        type="text"
                                                        id="add_tags-input"
                                                        list="add-tag-labels"
                                                        placeholder="Choose tags"
                                                        node_ref=add_input_ref
                                                        on:change=move |ev: Event| {
                                                            let input = add_input_ref.get().unwrap();
                                                            ev.stop_propagation();
                                                            add_tags
                                                                .update(|tags| {
                                                                    tags.insert(input.value());
                                                                });
                                                            input.set_value("");
                                                        }
                                                    />
                                                    <datalist id="add-tag-labels">
                                                        <For
                                                            each=move || tags.get_value()
                                                            key=|t| t.label.clone()
                                                            let:tag
                                                        >
                                                            <option value=tag.label></option>
                                                        </For>
                                                    </datalist>
                                                </p>
                                                <p class="field is-grouped is-grouped-multiline">
                                                    <forms::TagList
                                                        attrs=add_tags.into()
                                                        rm_attr=move |attr| {
                                                            add_tags
                                                                .update(|coll| {
                                                                    coll.remove(&attr);
                                                                });
                                                        }
                                                    />
                                                </p>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            }
                                .into_view()
                        }
                    })
            }}
        </Transition>
    }
}

#[component]
fn ResultsDisplay(meta: SearchMeta, selected_assets: RwSignal<HashSet<String>>) -> impl IntoView {
    // store the results in the reactive system so the view can be Fn()
    let results = store_value(meta.results);
    let toggle_asset = move |id: &String| {
        selected_assets.update(|list| {
            if list.contains(id) {
                list.remove(id);
            } else {
                list.insert(id.to_owned());
            }
        })
    };
    view! {
        <div class="grid is-col-min-12 padding-2">
            <For each=move || results.get_value() key=|r| r.asset_id.clone() let:elem>
                {move || {
                    let asset = store_value(elem.clone());
                    view! {
                        <div class="cell">
                            <div
                                class="card"
                                class:selected=move || {
                                    selected_assets
                                        .with(|list| list.contains(&asset.get_value().asset_id))
                                }
                                on:click=move |_| toggle_asset(&asset.get_value().asset_id)
                            >
                                <div class="card-image">
                                    <CardFigure asset />
                                </div>
                                <div class="card-content">
                                    <div class="content">
                                        <CardContent
                                            datetime=asset.get_value().datetime
                                            location=asset.get_value().location
                                        />
                                    </div>
                                </div>
                            </div>
                        </div>
                    }
                }}
            </For>
        </div>
    }
}

#[component]
fn CardFigure(asset: StoredValue<SearchResult>) -> impl IntoView {
    view! {
        <figure class="image">
            {move || {
                let filename = store_value(asset.get_value().filename.to_owned());
                if asset.get_value().media_type.starts_with("video/") {
                    let src = format!("/rest/asset/{}", asset.get_value().asset_id);
                    let mut media_type = asset.get_value().media_type.clone();
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
                    let src = format!("/rest/asset/{}", asset.get_value().asset_id);
                    view! {
                        <figcaption>{move || filename.get_value()}</figcaption>
                        <audio controls>
                            <source src=src type=asset.get_value().media_type.clone() />
                        </audio>
                    }
                        .into_view()
                } else {
                    let src = format!("/rest/thumbnail/640/640/{}", asset.get_value().asset_id);
                    view! {
                        <img
                            src=src
                            alt=asset.get_value().filename.clone()
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
fn CardContent(datetime: DateTime<Utc>, location: Option<Location>) -> impl IntoView {
    let datetime = store_value(datetime);
    let location = store_value(location);
    view! {
        <div class="content">
            <time>{move || { datetime.get_value().format("%A %B %e, %Y").to_string() }}</time>
            <Show when=move || location.get_value().is_some() fallback=|| ()>
                <br />
                <span>{move || location.get_value().unwrap().to_string()}</span>
            </Show>
        </div>
    }
}

/// Apply the given operations to the selected assets.
async fn apply_changes(ops: BulkEditParams) {
    if let Err(err) = bulk_edit(ops).await {
        log::error!("bulk edit failed: {err:#?}");
    } else {
        // Force the entire page to reload (only if there were no errors), every
        // single cached resource is now potentially out of date, and Leptos
        // does not give us an easy to to handle this situation.
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let location = document.location().unwrap();
        if let Err(err) = location.reload() {
            log::error!("page reload failed: {err:#?}");
        }
    }
}
