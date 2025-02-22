//
// Copyright (c) 2024 Nathan Fiedler
//
use crate::domain::entities::{SortField, SortOrder};
use crate::preso::leptos::{forms, nav, paging, results};
use crate::preso::leptos::{BrowseParams, SearchParams, Season, Year};
use chrono::{Datelike, TimeZone, Utc};
use codee::string::{FromToStringCodec, JsonSerdeCodec};
use html::Div;
use leptos::*;
use leptos_use::on_click_outside;
use leptos_use::storage::{use_local_storage_with_options, UseStorageOptions};
use std::collections::HashSet;

struct SearchParamsBuilder {
    params: SearchParams,
}

impl SearchParamsBuilder {
    fn new() -> Self {
        // default search will show all assets in descending date order
        Self {
            params: SearchParams {
                tags: None,
                locations: None,
                after: None,
                before: Some(chrono::Utc::now()),
                filename: None,
                media_type: None,
                sort_field: Some(SortField::Date),
                sort_order: Some(SortOrder::Descending),
            },
        }
    }

    /// Build the search parameters from all of the available selections.
    fn make(
        tags: HashSet<String>,
        locs: HashSet<String>,
        year: Option<i32>,
        season: Option<Season>,
        media_type: Option<String>,
        order: String,
    ) -> SearchParams {
        let mut builder = SearchParamsBuilder::new();
        builder = builder.tags(tags).locations(locs);
        if let Some(year) = year {
            if let Some(season) = season {
                builder = builder.year_and_season(year, season);
            } else {
                builder = builder.year(year);
            }
        } else if let Some(season) = season {
            builder = builder.season(season);
        }
        if let Some(media_type) = media_type {
            builder = builder.media_type(media_type);
        }
        builder = builder.sort_order(SortOrder::from(order.as_str()));
        builder.build()
    }

    fn sort_order(mut self, order: SortOrder) -> Self {
        self.params.sort_order = Some(order);
        self
    }

    fn tags(mut self, tags: HashSet<String>) -> Self {
        self.params.tags = if tags.is_empty() {
            None
        } else {
            Some(tags.into_iter().collect())
        };
        self
    }

    fn locations(mut self, locations: HashSet<String>) -> Self {
        self.params.locations = if locations.is_empty() {
            None
        } else {
            Some(locations.into_iter().collect())
        };
        self
    }

    fn media_type(mut self, media_type: String) -> Self {
        self.params.media_type = Some(media_type);
        self
    }

    /// Set the year but not the season.
    fn year(mut self, year: i32) -> Self {
        let after = Utc
            .with_ymd_and_hms(year, 1, 1, 0, 0, 0)
            .earliest()
            .unwrap_or_else(Utc::now);
        let before = Utc
            .with_ymd_and_hms(year + 1, 1, 1, 0, 0, 0)
            .earliest()
            .unwrap_or_else(Utc::now);
        self.params.before = Some(before);
        self.params.after = Some(after);
        self
    }

    /// Set the season for the current year.
    fn season(self, season: Season) -> Self {
        let year = Utc::now().year();
        self.year_and_season(year, season)
    }

    /// Set the year and season together.
    fn year_and_season(mut self, year: i32, season: Season) -> Self {
        let (after, before) = match season {
            Season::Winter => (
                Utc.with_ymd_and_hms(year, 1, 1, 0, 0, 0)
                    .earliest()
                    .unwrap_or_else(Utc::now),
                Utc.with_ymd_and_hms(year, 4, 1, 0, 0, 0)
                    .earliest()
                    .unwrap_or_else(Utc::now),
            ),
            Season::Spring => (
                Utc.with_ymd_and_hms(year, 4, 1, 0, 0, 0)
                    .earliest()
                    .unwrap_or_else(Utc::now),
                Utc.with_ymd_and_hms(year, 7, 1, 0, 0, 0)
                    .earliest()
                    .unwrap_or_else(Utc::now),
            ),
            Season::Summer => (
                Utc.with_ymd_and_hms(year, 7, 1, 0, 0, 0)
                    .earliest()
                    .unwrap_or_else(Utc::now),
                Utc.with_ymd_and_hms(year, 10, 1, 0, 0, 0)
                    .earliest()
                    .unwrap_or_else(Utc::now),
            ),
            Season::Fall => (
                Utc.with_ymd_and_hms(year, 10, 1, 0, 0, 0)
                    .earliest()
                    .unwrap_or_else(Utc::now),
                Utc.with_ymd_and_hms(year + 1, 1, 1, 0, 0, 0)
                    .earliest()
                    .unwrap_or_else(Utc::now),
            ),
        };
        self.params.before = Some(before);
        self.params.after = Some(after);
        self
    }

    fn build(&self) -> SearchParams {
        self.params.clone()
    }
}

#[component]
pub fn HomePage() -> impl IntoView {
    // multiple tags will narrow the search results
    let (selected_tags, set_selected_tags, _) =
        use_local_storage_with_options::<HashSet<String>, JsonSerdeCodec>(
            "home-selected-tags",
            UseStorageOptions::default()
                .initial_value(HashSet::new())
                .delay_during_hydration(true),
        );
    // multiple locations will narrow the search results
    let (selected_locations, set_selected_locations, _) =
        use_local_storage_with_options::<HashSet<String>, JsonSerdeCodec>(
            "home-selected-locations",
            UseStorageOptions::default()
                .initial_value(HashSet::new())
                .delay_during_hydration(true),
        );
    // chosen year by which to narrow results
    let (selected_year, set_selected_year, _) =
        use_local_storage_with_options::<Option<i32>, JsonSerdeCodec>(
            "home-selected-year",
            UseStorageOptions::default()
                .initial_value(None)
                .delay_during_hydration(true),
        );
    // chosen year by which to narrow results
    let (selected_season, set_selected_season, _) =
        use_local_storage_with_options::<Option<Season>, JsonSerdeCodec>(
            "home-selected-season",
            UseStorageOptions::default()
                .initial_value(None)
                .delay_during_hydration(true),
        );
    // chosen media type by which to narrow results
    let (selected_type, set_selected_type, _) =
        use_local_storage_with_options::<Option<String>, JsonSerdeCodec>(
            "home-selected-type",
            UseStorageOptions::default()
                .initial_value(None)
                .delay_during_hydration(true),
        );
    // page of results to be displayed (1-based)
    let (selected_page, set_selected_page, _) =
        use_local_storage_with_options::<i32, FromToStringCodec>(
            "home-selected-page",
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
    let (sort_order, set_sort_order, _) = use_local_storage_with_options::<String, JsonSerdeCodec>(
        "home-sort-order",
        UseStorageOptions::default()
            .initial_value("descending")
            .delay_during_hydration(true),
    );
    // search for assets using the given criteria
    let results = create_resource(
        move || {
            (
                selected_tags.get(),
                selected_locations.get(),
                selected_year.get(),
                selected_season.get(),
                selected_type.get(),
                selected_page.get(),
                page_size.get(),
                sort_order.get(),
            )
        },
        |(tags, locs, year, season, media_type, page, count, order)| async move {
            let params = SearchParamsBuilder::make(tags, locs, year, season, media_type, order);
            let offset = count * (page - 1);
            super::search(params, Some(count), Some(offset)).await
        },
    );
    // begin browsing assets, starting with the chosen asset; the given index is
    // zero-based within the current page of results
    let browse_asset = create_action(move |idx: &usize| {
        let tags = selected_tags.get_untracked();
        let locs = selected_locations.get_untracked();
        let year = selected_year.get_untracked();
        let season = selected_season.get_untracked();
        let media_type = selected_type.get_untracked();
        let order = sort_order.get_untracked();
        let params = SearchParamsBuilder::make(tags, locs, year, season, media_type, order);
        let page = selected_page.get_untracked();
        let count = page_size.get_untracked();
        let offset = count * (page - 1);
        let mut browse = BrowseParams::from(params);
        browse.asset_index = *idx + offset as usize;
        begin_browsing(browse)
    });

    view! {
        <nav::NavBar />
        <div class="container">
            <nav class="level">
                <div class="level-left">
                    <div class="level-item">
                        <forms::TagsChooser add_tag=move |label| {
                            batch(|| {
                                set_selected_tags
                                    .update(|tags| {
                                        tags.insert(label);
                                    });
                                set_selected_page.set(1);
                            })
                        } />
                    </div>
                    <div class="level-item">
                        <forms::LocationsChooser add_location=move |label| {
                            batch(|| {
                                set_selected_locations
                                    .update(|locations| {
                                        locations.insert(label);
                                    });
                                set_selected_page.set(1);
                            })
                        } />
                    </div>
                    <div class="level-item">
                        <div class="field">
                            <YearChooser
                                selected_year
                                set_year=move |value| {
                                    batch(|| {
                                        set_selected_year.set(value);
                                        set_selected_page.set(1);
                                    })
                                }
                            />
                        </div>
                    </div>
                    <div class="level-item">
                        <div class="field">
                            <SeasonChooser
                                selected_season
                                set_season=move |value| {
                                    batch(|| {
                                        if value.is_some() && selected_year.get().is_none() {
                                            let year = Utc::now().year();
                                            set_selected_year.set(Some(year));
                                        }
                                        set_selected_season.set(value);
                                        set_selected_page.set(1);
                                    })
                                }
                            />
                        </div>
                    </div>
                    <div class="level-item">
                        <div class="field">
                            <forms::TypesChooser
                                selected_type
                                set_type=move |value| {
                                    batch(|| {
                                        set_selected_type.set(value);
                                        set_selected_page.set(1);
                                    })
                                }
                            />
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
                </div>
            </nav>
        </div>

        <div class="container my-3">
            <div class="field is-grouped is-grouped-multiline">
                <forms::TagList
                    attrs=selected_tags
                    rm_attr=move |attr| {
                        batch(|| {
                            set_selected_tags
                                .update(|coll| {
                                    coll.remove(&attr);
                                });
                            set_selected_page.set(1);
                        })
                    }
                />
                <forms::TagList
                    attrs=selected_locations
                    rm_attr=move |attr| {
                        batch(|| {
                            set_selected_locations
                                .update(|coll| {
                                    coll.remove(&attr);
                                });
                            set_selected_page.set(1);
                        })
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
                            view! {
                                <results::ResultsDisplay
                                    meta
                                    onclick=move |idx| browse_asset.dispatch(idx)
                                />
                            }
                        }
                    })
            }}
        </Transition>
    }
}

#[component]
fn YearChooser<F>(selected_year: Signal<Option<i32>>, set_year: F) -> impl IntoView
where
    F: Fn(Option<i32>) + Copy + 'static,
{
    // the years returned from the server are in no particular order
    let years = create_resource(
        || (),
        |_| async move {
            let mut results = super::fetch_years().await;
            if let Ok(data) = results.as_mut() {
                // sort in reverse chronological order for selection convenience
                // (most recent years near the top of the dropdown menu)
                data.sort_by(|a, b| b.value.cmp(&a.value));
                // inject the current year if not already present so that the
                // season selection has something to select when year is unset
                let current_year = Utc::now().year();
                if data.len() > 0 && data[0].value != current_year {
                    data.insert(
                        0,
                        Year {
                            value: current_year,
                            count: 0,
                        },
                    );
                }
            }
            results
        },
    );
    let dropdown_open = create_rw_signal(false);
    let dropdown_ref = create_node_ref::<Div>();
    let _ = on_click_outside(dropdown_ref, move |_| dropdown_open.set(false));

    view! {
        <Transition fallback=move || {
            view! { "Loading..." }
        }>
            {move || {
                years
                    .get()
                    .map(|resp| match resp {
                        Err(err) => {
                            view! { <span>{move || format!("Error: {}", err)}</span> }.into_view()
                        }
                        Ok(data) => {
                            let years = store_value(data);
                            view! {
                                <div
                                    class="dropdown"
                                    class:is-active=move || dropdown_open.get()
                                    node_ref=dropdown_ref
                                >
                                    <div class="dropdown-trigger">
                                        <button
                                            class="button"
                                            on:click=move |_| { dropdown_open.update(|v| { *v = !*v }) }
                                            aria-haspopup="true"
                                            aria-controls="dropdown-menu"
                                        >
                                            <Show
                                                when=move || selected_year.get().is_some()
                                                fallback=move || "Year"
                                            >
                                                {move || selected_year.get().unwrap().to_string()}
                                            </Show>
                                        </button>
                                    </div>
                                    <div class="dropdown-menu" id="dropdown-menu" role="menu">
                                        <div class="dropdown-content">
                                            <a
                                                class="dropdown-item"
                                                on:click=move |_| {
                                                    set_year(None);
                                                    dropdown_open.set(false)
                                                }
                                            >
                                                Any
                                            </a>
                                            <For
                                                each=move || years.get_value()
                                                key=|y| y.value
                                                let:year
                                            >
                                                <a
                                                    class="dropdown-item"
                                                    on:click=move |_| {
                                                        set_year(Some(year.value));
                                                        dropdown_open.set(false)
                                                    }
                                                >
                                                    {move || year.value.to_string()}
                                                </a>
                                            </For>
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
fn SeasonChooser<F>(selected_season: Signal<Option<Season>>, set_season: F) -> impl IntoView
where
    F: Fn(Option<Season>) + Copy + 'static,
{
    let dropdown_open = create_rw_signal(false);
    let dropdown_ref = create_node_ref::<Div>();
    let _ = on_click_outside(dropdown_ref, move |_| dropdown_open.set(false));

    view! {
        <div class="dropdown" class:is-active=move || dropdown_open.get() node_ref=dropdown_ref>
            <div class="dropdown-trigger">
                <button
                    class="button"
                    on:click=move |_| { dropdown_open.update(|v| { *v = !*v }) }
                    aria-haspopup="true"
                    aria-controls="dropdown-menu"
                >
                    <Show when=move || selected_season.get().is_some() fallback=move || "Season">
                        {move || selected_season.get().unwrap().to_string()}
                    </Show>
                </button>
            </div>
            <div class="dropdown-menu" id="dropdown-menu" role="menu">
                <div class="dropdown-content">
                    <a
                        class="dropdown-item"
                        on:click=move |_| {
                            set_season(None);
                            dropdown_open.set(false)
                        }
                    >
                        Any
                    </a>
                    <a
                        class="dropdown-item"
                        on:click=move |_| {
                            set_season(Some(Season::Winter));
                            dropdown_open.set(false)
                        }
                    >
                        {move || Season::Winter.to_string()}
                    </a>
                    <a
                        class="dropdown-item"
                        on:click=move |_| {
                            set_season(Some(Season::Spring));
                            dropdown_open.set(false)
                        }
                    >
                        {move || Season::Spring.to_string()}
                    </a>
                    <a
                        class="dropdown-item"
                        on:click=move |_| {
                            set_season(Some(Season::Summer));
                            dropdown_open.set(false)
                        }
                    >
                        {move || Season::Summer.to_string()}
                    </a>
                    <a
                        class="dropdown-item"
                        on:click=move |_| {
                            set_season(Some(Season::Fall));
                            dropdown_open.set(false)
                        }
                    >
                        {move || Season::Fall.to_string()}
                    </a>
                </div>
            </div>
        </div>
    }
}

/// Set the browse params to begin browsing with the chosen asset. Navigates to
/// the asset details page.
async fn begin_browsing(params: BrowseParams) {
    let (_, set_browse_params, _) = use_local_storage_with_options::<BrowseParams, JsonSerdeCodec>(
        "browse-params",
        UseStorageOptions::default()
            .initial_value(BrowseParams::default())
            .delay_during_hydration(true),
    );
    set_browse_params.set(params);
    let navigate = leptos_router::use_navigate();
    navigate("/browse", Default::default());
}
