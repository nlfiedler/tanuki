//
// Copyright (c) 2024 Nathan Fiedler
//
use crate::domain::entities::{AssetInput, Location, SearchResult};
use crate::preso::common::SearchMeta;
use crate::preso::leptos::client::{nav, paging};
use crate::preso::leptos::server::*;
use chrono::{DateTime, TimeDelta, Utc};
use codee::string::{FromToStringCodec, JsonSerdeCodec};
use leptos::ev::Event;
use leptos::html::Input;
use leptos::*;
use leptos_use::storage::{use_local_storage_with_options, UseStorageOptions};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;
use std::str::FromStr;

#[allow(dead_code)]
#[derive(Copy, Clone, Deserialize, Eq, Hash, PartialEq, Serialize)]
enum RecentRange {
    Day,
    Week,
    Month,
    Year,
    All,
}

impl RecentRange {
    fn as_date(&self) -> Option<DateTime<Utc>> {
        let now = Utc::now();
        match *self {
            RecentRange::Day => Some(now - TimeDelta::days(1 as i64)),
            RecentRange::Week => Some(now - TimeDelta::days(7 as i64)),
            RecentRange::Month => Some(now - TimeDelta::days(30 as i64)),
            RecentRange::Year => Some(now - TimeDelta::days(365 as i64)),
            RecentRange::All => None,
        }
    }
}

impl Default for RecentRange {
    fn default() -> Self {
        RecentRange::All
    }
}

impl fmt::Display for RecentRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            RecentRange::Day => write!(f, "Day"),
            RecentRange::Week => write!(f, "Week"),
            RecentRange::Month => write!(f, "Month"),
            RecentRange::Year => write!(f, "Year"),
            RecentRange::All => write!(f, "All"),
        }
    }
}

/// Convert the inputs into a list of `AssetInputId` and send to the server as a
/// bulk update.
async fn save_changes(asset_ids: HashSet<String>, tags: HashSet<String>, location: Location) {
    let tags: Vec<String> = tags.into_iter().collect();
    let location = Some(location);
    let inputs: Vec<AssetInput> = asset_ids
        .iter()
        .map(|id| AssetInput {
            key: id.to_owned(),
            tags: Some(tags.clone()),
            location: location.clone(),
            caption: None,
            datetime: None,
            filename: None,
            media_type: None,
        })
        .collect();
    if let Err(err) = bulk_update(inputs).await {
        log::error!("bulk update failed: {err:#?}");
    } else {
        // Force the entire page to reload (only if there were no errors
        // earlier), every single cached resource is now potentially out of
        // date, and Leptos does not give us an easy to to handle this
        // situation.
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let location = document.location().unwrap();
        if let Err(err) = location.reload() {
            log::error!("page reload failed: {err:#?}");
        }
    }
}

#[component]
pub fn PendingPage() -> impl IntoView {
    // range of time for which to find recent imports
    let (selected_range, set_selected_range, _) =
        use_local_storage_with_options::<RecentRange, JsonSerdeCodec>(
            "pending-selected-range",
            UseStorageOptions::default()
                .initial_value(RecentRange::All)
                .delay_during_hydration(true),
        );
    // page of results to be displayed (1-based)
    let (selected_page, set_selected_page, _) =
        use_local_storage_with_options::<i32, FromToStringCodec>(
            "pending-selected-page",
            UseStorageOptions::default()
                .initial_value(1)
                .delay_during_hydration(true),
        );
    // number of assets to display in a single page of results
    let (page_size, set_page_size, _) = use_local_storage_with_options::<i32, FromToStringCodec>(
        "page-size",
        UseStorageOptions::default()
            .initial_value(18)
            .delay_during_hydration(true),
    );
    // search for recent imports within the given time range
    let results = create_resource(
        move || (selected_range.get(), selected_page.get(), page_size.get()),
        |(range, page, count)| async move {
            let offset = count * (page - 1);
            recent(range.as_date(), Some(count), Some(offset)).await
        },
    );
    let selected_assets = create_rw_signal::<HashSet<String>>(HashSet::new());
    let (selected_tags, set_selected_tags) = create_signal::<HashSet<String>>(HashSet::new());
    let (selected_location, set_selected_location) = create_signal(Location::default());
    let submittable = create_memo(move |_| {
        with!(|selected_assets, selected_tags, selected_location| {
            // a location is not really considered "set" unless the label is
            // defined, as many assets will have geocoded location data at the
            // time of import
            (selected_tags.len() > 0 || selected_location.label.is_some())
                && selected_assets.len() > 0
        })
    });
    // compile the set of asset inputs and send to the server
    let save_action = create_action(move |_input: &()| {
        save_changes(
            selected_assets.get(),
            selected_tags.get(),
            selected_location.get(),
        )
    });

    view! {
        <nav::NavBar />
        <div class="container mb-4">
            <nav class="level">
                <div class="level-left">
                    <div class="level-item">
                        <RangeSelector selected_range set_selected_range />
                    </div>
                    <div class="level-item">
                        <PendingCount results />
                    </div>
                </div>

                <div class="level-right">
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
                </div>
            </nav>
        </div>

        <div class="container">
            <nav class="level">
                <div class="level-left">
                    <div class="level-item">
                        <div class="field">
                            <p class="control">
                                <TagsChooser add_tag=move |label| {
                                    set_selected_tags
                                        .update(|tags| {
                                            tags.insert(label);
                                        })
                                } />
                            </p>
                        </div>
                    </div>
                    <div class="level-item">
                        <div class="field">
                            <p class="control">
                                <LocationsChooser set_location=move |value| {
                                    let location = Location::from_str(&value).unwrap();
                                    set_selected_location.set(location);
                                } />
                            </p>
                        </div>
                    </div>
                </div>

                <div class="level-right">
                    <div class="level-item">
                        <Show
                            when=move || submittable.get()
                            fallback=|| {
                                view! {
                                    <button class="button" disabled>
                                        Save
                                    </button>
                                }
                            }
                        >
                            <input
                                class="button"
                                type="submit"
                                value="Save"
                                on:click=move |_| save_action.dispatch(())
                            />
                        </Show>
                    </div>
                </div>
            </nav>
        </div>

        <div class="container mt-3 mb-3">
            <div class="field is-grouped is-grouped-multiline">
                {move || {
                    selected_tags
                        .get()
                        .into_iter()
                        .map(move |attr| {
                            let stored_attr = store_value(attr);
                            view! {
                                <div class="control">
                                    <div class="tags has-addons">
                                        <a class="tag">{move || stored_attr.get_value()}</a>
                                        <a
                                            class="tag is-delete"
                                            on:click=move |_| {
                                                stored_attr
                                                    .with_value(|attr| {
                                                        set_selected_tags
                                                            .update(|coll| {
                                                                coll.remove(attr);
                                                            })
                                                    })
                                            }
                                        ></a>
                                    </div>
                                </div>
                            }
                        })
                        .collect::<Vec<_>>()
                }}
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
fn PendingCount(
    results: Resource<(RecentRange, i32, i32), Result<SearchMeta, ServerFnError>>,
) -> impl IntoView {
    // must use Suspense or Transition when waiting for a resouce
    view! {
        <Transition fallback=move || {
            view! { "..." }
        }>
            {move || {
                results
                    .get()
                    .map(|result| match result {
                        Err(err) => {
                            view! { <span>{move || format!("Error: {}", err)}</span> }.into_view()
                        }
                        Ok(meta) => {
                            view! {
                                <span>{move || format!("Pending items: {}", meta.count)}</span>
                            }
                                .into_view()
                        }
                    })
            }}
        </Transition>
    }
}

#[component]
fn RangeSelector(
    selected_range: Signal<RecentRange>,
    set_selected_range: WriteSignal<RecentRange>,
) -> impl IntoView {
    let elements = store_value(vec![
        RecentRange::All,
        RecentRange::Year,
        RecentRange::Month,
        RecentRange::Week,
        RecentRange::Day,
    ]);
    // be sure to access the signal inside the view!
    view! {
        <div class="field is-grouped">
            <For each=move || elements.get_value() key=|t| t.clone() let:range>
                <Show
                    when=move || range == selected_range.get()
                    fallback=move || {
                        view! {
                            <p class="control">
                                <button
                                    class="button"
                                    on:click=move |_| set_selected_range.set(range)
                                >
                                    {move || {
                                        view! { {move || { range.to_string() }} }
                                    }}
                                </button>
                            </p>
                        }
                    }
                >
                    <p class="control">
                        <button class="button is-active">
                            {move || {
                                view! { {move || { range.to_string() }} }
                            }}
                        </button>
                    </p>
                </Show>
            </For>
        </div>
    }
}

#[component]
fn TagsChooser<F>(add_tag: F) -> impl IntoView
where
    F: Fn(String) + Copy + 'static,
{
    // the tags returned from the server are in no particular order
    let tags = create_resource(
        || (),
        |_| async move {
            let mut results = fetch_tags().await;
            if let Ok(data) = results.as_mut() {
                data.sort_by(|a, b| a.label.cmp(&b.label));
            }
            results
        },
    );

    let input_ref = NodeRef::<Input>::new();
    //
    // n.b. on:change is called under several conditions:
    // - user selects one of the available datalist options
    // - user types some text and presses the Enter key
    // - user types some text and moves the focus
    //
    let on_change = move |ev: Event| {
        let input = input_ref.get().unwrap();
        ev.stop_propagation();
        add_tag(input.value());
        input.set_value("");
    };

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
                                <div class="field is-horizontal">
                                    <div class="field-label is-normal">
                                        <label class="label">Tags</label>
                                    </div>
                                    <div class="field-body">
                                        <div class="field">
                                            <p class="control">
                                                <input
                                                    class="input"
                                                    type="text"
                                                    id="tags-input"
                                                    list="tag-labels"
                                                    placeholder="Choose tags"
                                                    node_ref=input_ref
                                                    on:change=on_change
                                                />
                                                <datalist id="tag-labels">
                                                    <For
                                                        each=move || tags.get_value()
                                                        key=|t| t.label.clone()
                                                        let:tag
                                                    >
                                                        <option value=tag.label></option>
                                                    </For>
                                                </datalist>
                                            </p>
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
fn LocationsChooser<F>(set_location: F) -> impl IntoView
where
    F: Fn(String) + Copy + 'static,
{
    // the locations returned from the server are in no particular order
    let locations = create_resource(
        || (),
        |_| async move {
            let mut results = fetch_raw_locations().await;
            if let Ok(data) = results.as_mut() {
                data.sort_by(|a, b| a.label.cmp(&b.label));
            }
            results
        },
    );

    let input_ref = NodeRef::<Input>::new();
    //
    // n.b. on:change is called under several conditions:
    // - user selects one of the available datalist options
    // - user types some text and presses the Enter key
    // - user types some text and moves the focus
    //
    let on_change = move |ev: Event| {
        let input = input_ref.get().unwrap();
        ev.stop_propagation();
        set_location(input.value());
    };

    view! {
        <Transition fallback=move || {
            view! { "Loading..." }
        }>
            {move || {
                locations
                    .get()
                    .map(|resp| match resp {
                        Err(err) => {
                            view! { <span>{move || format!("Error: {}", err)}</span> }.into_view()
                        }
                        Ok(data) => {
                            let locations = store_value(data);
                            view! {
                                <div class="field is-horizontal">
                                    <div class="field-label is-normal">
                                        <label class="label">Location</label>
                                    </div>
                                    <div class="field-body">
                                        <div class="field">
                                            <p class="control">
                                                <input
                                                    class="input"
                                                    type="text"
                                                    id="locations-input"
                                                    list="location-labels"
                                                    placeholder="Choose location"
                                                    node_ref=input_ref
                                                    on:change=on_change
                                                />
                                                <datalist id="location-labels">
                                                    {move || {
                                                        locations
                                                            .get_value()
                                                            .iter()
                                                            .map(|loc| {
                                                                let desc = loc.to_string();
                                                                view! { <option value=desc></option> }
                                                            })
                                                            .collect::<Vec<_>>()
                                                    }}
                                                </datalist>
                                            </p>
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
    .into_view()
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
        <div class="grid is-col-min-16 padding-2">
            <For each=move || results.get_value() key=|r| r.asset_id.clone() let:elem>
                {move || {
                    let asset = store_value(elem.clone());
                    let card_style = if selected_assets.with(|list| list.contains(&elem.asset_id)) {
                        "border: medium solid hsl(217, 71%, 53%);"
                    } else {
                        ""
                    };
                    view! {
                        <div class="cell">
                            <div class="card" style=card_style>
                                <header class="card-header">
                                    <p class="card-header-title">
                                        {move || asset.get_value().filename}
                                    </p>
                                    <button class="card-header-icon">
                                        <span class="icon">
                                            <i class="fas fa-angle-down"></i>
                                        </span>
                                    </button>
                                </header>
                                <div
                                    class="card-image"
                                    on:click=move |_| toggle_asset(&asset.get_value().asset_id)
                                >
                                    <figure class="image">
                                        {move || {
                                            if asset.get_value().media_type.starts_with("video/") {
                                                let src = format!(
                                                    "/rest/asset/{}",
                                                    asset.get_value().asset_id,
                                                );
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
                                            } else if asset.get_value().media_type.starts_with("audio/")
                                            {
                                                let src = format!(
                                                    "/rest/asset/{}",
                                                    asset.get_value().asset_id,
                                                );
                                                view! {
                                                    <figcaption>{move || asset.get_value().filename}</figcaption>
                                                    <audio controls>
                                                        <source src=src type=asset.get_value().media_type />
                                                    </audio>
                                                }
                                                    .into_view()
                                            } else {
                                                let src = format!(
                                                    "/rest/thumbnail/960/960/{}",
                                                    asset.get_value().asset_id,
                                                );
                                                view! {
                                                    <img
                                                        src=src
                                                        alt=asset.get_value().filename.clone()
                                                        style="max-width: 100%; width: auto;"
                                                    />
                                                }
                                                    .into_view()
                                            }
                                        }}
                                    </figure>
                                </div>
                                <div class="card-content">
                                    <div class="content">
                                        <CardContent asset />
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
fn CardContent(asset: StoredValue<SearchResult>) -> impl IntoView {
    view! {
        <div class="content">
            <time>{move || { asset.get_value().datetime.format("%A %B %e, %Y").to_string() }}</time>
            <Show when=move || asset.get_value().location.is_some() fallback=|| ()>
                <br />
                <span>{move || asset.get_value().location.unwrap().to_string()}</span>
            </Show>
        </div>
    }
}
