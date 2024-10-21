//
// Copyright (c) 2024 Nathan Fiedler
//
use crate::domain::entities::{Location, SortField, SortOrder};
use crate::preso::common::{SearchMeta, SearchParams, Season, Year};
use crate::preso::leptos::client::{nav, paging};
use crate::preso::leptos::server::*;
use chrono::{DateTime, Datelike, TimeZone, Utc};
use codee::string::{FromToStringCodec, JsonSerdeCodec};
use leptos::ev::Event;
use leptos::html::Input;
use leptos::*;
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

    /// Set the year but not the season.
    fn set_year(mut self, year: i32) -> Self {
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
    fn set_season(self, season: Season) -> Self {
        let year = Utc::now().year();
        self.set_year_and_season(year, season)
    }

    /// Set the year and season together.
    fn set_year_and_season(mut self, year: i32, season: Season) -> Self {
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
    // multiple tags will _narrow_ the search results
    let (selected_tags, set_selected_tags, _) =
        use_local_storage_with_options::<HashSet<String>, JsonSerdeCodec>(
            "home-selected-tags",
            UseStorageOptions::default()
                .initial_value(HashSet::new())
                .delay_during_hydration(true),
        );
    // multiple locations will _widen_ the search results
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
    // search for assets using the given criteria
    let results = create_resource(
        move || {
            (
                selected_tags.get(),
                selected_locations.get(),
                selected_year.get(),
                selected_season.get(),
                selected_page.get(),
                page_size.get(),
            )
        },
        |(tags, locs, year, season, page, count)| async move {
            let mut builder = SearchParamsBuilder::new();
            builder = builder.tags(tags).locations(locs);
            if let Some(year) = year {
                if let Some(season) = season {
                    builder = builder.set_year_and_season(year, season);
                } else {
                    builder = builder.set_year(year);
                }
            } else if let Some(season) = season {
                builder = builder.set_season(season);
            }
            let params = builder.build();
            let offset = count * (page - 1);
            search(params, Some(count), Some(offset)).await
        },
    );

    view! {
        <nav::NavBar />
        <div class="container">
            <nav class="level">
                <div class="level-left">
                    <div class="level-item">
                        <div class="field">
                            <p class="control">
                                <TagsChooser add_tag=move |label| {
                                    batch(|| {
                                        set_selected_tags
                                            .update(|tags| {
                                                tags.insert(label);
                                            });
                                        set_selected_page.set(1);
                                    })
                                } />
                            </p>
                        </div>
                    </div>
                    <div class="level-item">
                        <div class="field">
                            <p class="control">
                                <LocationsChooser add_location=move |label| {
                                    batch(|| {
                                        set_selected_locations
                                            .update(|locations| {
                                                locations.insert(label);
                                            });
                                        set_selected_page.set(1);
                                    })
                                } />
                            </p>
                        </div>
                    </div>
                    <div class="level-item">
                        <div class="field">
                            <p class="control">
                                <YearChooser
                                    selected_year
                                    set_year=move |value| {
                                        batch(|| {
                                            set_selected_year.set(value);
                                            set_selected_page.set(1);
                                        })
                                    }
                                />
                            </p>
                        </div>
                    </div>
                    <div class="level-item">
                        <div class="field">
                            <p class="control">
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
                            </p>
                        </div>
                    </div>
                </div>

                <div class="level-right">
                    <div class="level-item">
                        <Transition fallback=move || {
                            view! { "..." }
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
                                                <span>{move || format!("{} results", meta.count)}</span>
                                            }
                                                .into_view()
                                        }
                                    })
                            }}
                        </Transition>
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
                <TagList
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
                <TagList
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
                            view! { <ResultsDisplay meta /> }
                        }
                    })
            }}
        </Transition>
    }
}

#[component]
fn ResultsDisplay(meta: SearchMeta) -> impl IntoView {
    // store the results in the reactive system so the view can be Fn()
    let results = store_value(meta.results);
    view! {
        <div class="grid is-col-min-16 padding-2">
            <For each=move || results.get_value() key=|r| r.asset_id.clone() let:asset>
                <div class="cell">
                    <a href=format!("/asset/{}", asset.asset_id)>
                        <div class="card">
                            <div class="card-image">
                                <figure class="image">
                                    {move || {
                                        let filename = store_value(asset.filename.to_owned());
                                        if asset.media_type.starts_with("video/") {
                                            let src = format!("/rest/asset/{}", asset.asset_id);
                                            let mut media_type = asset.media_type.clone();
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
                                        } else if asset.media_type.starts_with("audio/") {
                                            let src = format!("/rest/asset/{}", asset.asset_id);
                                            view! {
                                                <figcaption>{move || filename.get_value()}</figcaption>
                                                <audio controls>
                                                    <source src=src type=asset.media_type.clone() />
                                                </audio>
                                            }
                                                .into_view()
                                        } else {
                                            let src = format!(
                                                "/rest/thumbnail/960/960/{}",
                                                asset.asset_id,
                                            );
                                            view! {
                                                <img
                                                    src=src
                                                    alt=asset.filename.clone()
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
                                    <CardContent datetime=asset.datetime location=asset.location />
                                </div>
                            </div>
                        </div>
                    </a>
                </div>
            </For>
        </div>
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

/// Show list of selected attributes as tags/chips.
#[component]
fn TagList<F>(attrs: Signal<HashSet<String>>, rm_attr: F) -> impl IntoView
where
    F: Fn(String) + Copy + 'static,
{
    // be sure to access the signal inside the view!
    view! {
        {move || {
            attrs
                .get()
                .into_iter()
                .map(move |attr| {
                    let attr = store_value(attr);
                    view! {
                        <div class="control">
                            <div class="tags has-addons">
                                <a class="tag">{move || attr.get_value()}</a>
                                <a
                                    class="tag is-delete"
                                    on:click=move |_| { rm_attr(attr.get_value()) }
                                ></a>
                            </div>
                        </div>
                    }
                })
                .collect::<Vec<_>>()
        }}
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
fn LocationsChooser<F>(add_location: F) -> impl IntoView
where
    F: Fn(String) + Copy + 'static,
{
    // the locations returned from the server are in no particular order
    let locations = create_resource(
        || (),
        |_| async move {
            let mut results = fetch_all_locations().await;
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
        add_location(input.value());
        input.set_value("");
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
                                        <label class="label">Locations</label>
                                    </div>
                                    <div class="field-body">
                                        <div class="field">
                                            <p class="control">
                                                <input
                                                    class="input"
                                                    type="text"
                                                    id="locations-input"
                                                    list="location-labels"
                                                    placeholder="Choose locations"
                                                    node_ref=input_ref
                                                    on:change=on_change
                                                />
                                                <datalist id="location-labels">
                                                    <For
                                                        each=move || locations.get_value()
                                                        key=|l| l.label.clone()
                                                        let:loc
                                                    >
                                                        <option value=loc.label></option>
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
fn YearChooser<F>(selected_year: Signal<Option<i32>>, set_year: F) -> impl IntoView
where
    F: Fn(Option<i32>) + Copy + 'static,
{
    // the years returned from the server are in no particular order
    let years = create_resource(
        || (),
        |_| async move {
            let mut results = fetch_years().await;
            if let Ok(data) = results.as_mut() {
                // sort in reverse chronological order for selection convenience
                // (most recent years near the top of the dropdown menu)
                data.sort_by(|a, b| b.value.cmp(&a.value));
                // inject the current year if not already present so that the
                // season selection has something to select when year is unset
                let current_year = Utc::now().year();
                if data[0].value != current_year {
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
    let dropdown_class = move || {
        if dropdown_open.get() {
            "dropdown is-active"
        } else {
            "dropdown"
        }
    };

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
                                <div class=move || dropdown_class()>
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
    let dropdown_class = move || {
        if dropdown_open.get() {
            "dropdown is-active"
        } else {
            "dropdown"
        }
    };

    view! {
        <div class=move || dropdown_class()>
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
