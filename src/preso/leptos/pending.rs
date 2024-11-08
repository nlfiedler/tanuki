//
// Copyright (c) 2024 Nathan Fiedler
//
use crate::domain::entities::{AssetInput, Location, SearchResult};
use crate::preso::leptos::SearchMeta;
use crate::preso::leptos::{forms, nav, paging};
use chrono::{DateTime, TimeDelta, Utc};
use codee::string::{FromToStringCodec, JsonSerdeCodec};
use leptos::*;
use leptos_use::storage::{use_local_storage_with_options, UseStorageOptions};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;
use std::str::FromStr;

///
/// Search for assets that were recently imported.
///
/// Recently imported assets do not have any tags, location, or caption, and
/// thus are waiting for the user to give them additional details.
///
/// The count indicates how many results to return in a single query,
/// limited to a maximum of 250. Default value is `10`.
///
/// The offset is useful for pagination. Default value is `0`.
///
#[leptos::server]
pub async fn recent(
    since: Option<DateTime<Utc>>,
    count: Option<i32>,
    offset: Option<i32>,
) -> Result<SearchMeta, ServerFnError> {
    use crate::domain::entities::SearchResult;
    use crate::domain::usecases::recent::{Params, RecentImports};
    use crate::domain::usecases::UseCase;

    let repo = super::ssr::db()?;
    let usecase = RecentImports::new(Box::new(repo));
    let params = Params { after_date: since };
    let mut results: Vec<SearchResult> = usecase
        .call(params)
        .map_err(|e| leptos::ServerFnErrorErr::WrappedServerError(e))?;
    let total_count = results.len() as i32;
    let results = super::paginate_vector(&mut results, offset, count);
    let last_page: i32 = if total_count == 0 {
        1
    } else {
        (total_count as u32).div_ceil(count.unwrap_or(10) as u32) as i32
    };
    Ok(SearchMeta {
        results,
        count: total_count,
        last_page,
    })
}

///
/// Update multiple assets with the given values.
///
/// Returns the number of updated assets.
///
#[leptos::server]
pub async fn bulk_update(assets: Vec<AssetInput>) -> Result<i32, ServerFnError> {
    use crate::domain::usecases::update::{Params, UpdateAsset};
    use crate::domain::usecases::UseCase;

    let repo = super::ssr::db()?;
    let usecase = UpdateAsset::new(Box::new(repo));
    for asset in assets.iter() {
        let params: Params = Params::new(asset.clone().into());
        usecase
            .call(params)
            .map_err(|e| leptos::ServerFnErrorErr::WrappedServerError(e))?;
    }
    Ok(assets.len() as i32)
}

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
async fn save_changes(
    asset_ids: HashSet<String>,
    tags: HashSet<String>,
    location: Location,
    datetime: Option<DateTime<Utc>>,
) {
    let tags: Vec<String> = tags.into_iter().collect();
    let location = Some(location);
    let inputs: Vec<AssetInput> = asset_ids
        .iter()
        .map(|id| AssetInput {
            key: id.to_owned(),
            tags: Some(tags.clone()),
            location: location.clone(),
            caption: None,
            datetime,
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
    let datetime_input_ref: NodeRef<html::Input> = create_node_ref();
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
        // datetime: convert from local to UTC
        let local = chrono::offset::Local::now();
        let datetime_str = format!(
            "{}{}",
            datetime_input_ref.get().unwrap().value(),
            local.offset().to_string()
        );
        let datetime = if datetime_str.len() > 6 {
            // need to be flexible with the date/time format
            let pattern = if datetime_str.len() == 22 {
                "%Y-%m-%dT%H:%M%z"
            } else {
                "%Y-%m-%dT%H:%M:%S%z"
            };
            match DateTime::parse_from_str(&datetime_str, pattern) {
                Ok(datetime) => Some(datetime.to_utc()),
                Err(_) => None,
            }
        } else {
            None
        };
        save_changes(
            selected_assets.get(),
            selected_tags.get(),
            selected_location.get(),
            datetime,
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
                        <forms::TagsChooser add_tag=move |label| {
                            set_selected_tags
                                .update(|tags| {
                                    tags.insert(label);
                                })
                        } />
                    </div>
                    <div class="level-item">
                        <forms::FullLocationChooser set_location=move |value| {
                            let location = Location::from_str(&value).unwrap();
                            set_selected_location.set(location);
                        } />
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
                                        node_ref=datetime_input_ref
                                    />
                                </p>
                            </div>
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
                <forms::TagList
                    attrs=selected_tags.into()
                    rm_attr=move |attr| {
                        set_selected_tags
                            .update(|coll| {
                                coll.remove(&attr);
                            });
                    }
                />
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
                    view! {
                        <div class="cell">
                            <div
                                class="card"
                                class:selected=move || {
                                    selected_assets
                                        .with(|list| list.contains(&asset.get_value().asset_id))
                                }
                            >
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
                                    <CardFigure asset />
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
fn CardFigure(asset: StoredValue<SearchResult>) -> impl IntoView {
    view! {
        <figure class="image">
            {move || {
                if asset.get_value().media_type.starts_with("video/") {
                    let src = format!("/rest/asset/{}", asset.get_value().asset_id);
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
                    let src = format!("/rest/asset/{}", asset.get_value().asset_id);
                    view! {
                        <figcaption>{move || asset.get_value().filename}</figcaption>
                        <audio controls>
                            <source src=src type=asset.get_value().media_type />
                        </audio>
                    }
                        .into_view()
                } else {
                    let src = format!("/rest/thumbnail/960/960/{}", asset.get_value().asset_id);
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
