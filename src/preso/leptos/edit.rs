//
// Copyright (c) 2024 Nathan Fiedler
//
use crate::domain::entities::*;
use crate::preso::leptos::{forms, nav};
use crate::preso::leptos::{BulkEditParams, SearchMeta, SearchParams};
use chrono::{DateTime, Local, Utc};
use codee::string::JsonSerdeCodec;
use html::Div;
use leptos::ev::Event;
use leptos::html::Input;
use leptos::*;
use leptos_use::on_click_outside;
use leptos_use::storage::{use_local_storage_with_options, UseStorageOptions};
use std::collections::HashSet;

///
/// Perform one or more operations on multiple assets.
///
#[leptos::server(BulkEdit, "/api", "Cbor")]
pub async fn bulk_edit(ops: BulkEditParams) -> Result<u64, ServerFnError> {
    use crate::domain::usecases::edit::{EditAssets, Params};
    use crate::domain::usecases::UseCase;

    let repo = super::ssr::db()?;
    let usecase = EditAssets::new(Box::new(repo));
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

struct SearchParamsBuilder {
    params: SearchParams,
}

impl SearchParamsBuilder {
    fn new() -> Self {
        // default search will show nothing, but sorting will be by date in
        // descending order until options are provided to change that
        Self {
            params: SearchParams {
                tags: None,
                locations: None,
                after: None,
                before: None,
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

    fn set_after(mut self, after: DateTime<Utc>) -> Self {
        self.params.after = Some(after);
        self
    }

    fn set_before(mut self, before: DateTime<Utc>) -> Self {
        self.params.before = Some(before);
        self
    }

    fn set_media_type(mut self, media_type: String) -> Self {
        self.params.media_type = Some(media_type);
        self
    }

    fn build(&self) -> SearchParams {
        self.params.clone()
    }
}

#[component]
pub fn EditPage() -> impl IntoView {
    let after_date_input_ref: NodeRef<html::Input> = create_node_ref();
    let before_date_input_ref: NodeRef<html::Input> = create_node_ref();
    // multiple tags will _narrow_ the search results
    let (selected_tags, set_selected_tags, _) =
        use_local_storage_with_options::<HashSet<String>, JsonSerdeCodec>(
            "edit-selected-tags",
            UseStorageOptions::default()
                .initial_value(HashSet::new())
                .delay_during_hydration(true),
        );
    // multiple locations will _widen_ the search results
    let (selected_locations, set_selected_locations, _) =
        use_local_storage_with_options::<HashSet<String>, JsonSerdeCodec>(
            "edit-selected-locations",
            UseStorageOptions::default()
                .initial_value(HashSet::new())
                .delay_during_hydration(true),
        );
    // date for which assets must have a "best" date after
    let (after_date, set_after_date, _) =
        use_local_storage_with_options::<Option<DateTime<Utc>>, JsonSerdeCodec>(
            "edit-after-date",
            UseStorageOptions::default()
                .initial_value(None)
                .delay_during_hydration(true),
        );
    // date for which assets must have a "best" date before
    let (before_date, set_before_date, _) =
        use_local_storage_with_options::<Option<DateTime<Utc>>, JsonSerdeCodec>(
            "edit-before-date",
            UseStorageOptions::default()
                .initial_value(None)
                .delay_during_hydration(true),
        );
    // chosen media type by which to narrow results
    let (selected_type, set_selected_type, _) =
        use_local_storage_with_options::<Option<String>, JsonSerdeCodec>(
            "edit-selected-type",
            UseStorageOptions::default()
                .initial_value(None)
                .delay_during_hydration(true),
        );
    // search for assets using the given criteria
    let results = create_resource(
        move || {
            (
                selected_tags.get(),
                selected_locations.get(),
                after_date.get(),
                before_date.get(),
                selected_type.get(),
            )
        },
        |(tags, locs, after, before, media_type)| async move {
            let mut builder = SearchParamsBuilder::new();
            builder = builder.tags(tags).locations(locs);
            if let Some(after) = after {
                builder = builder.set_after(after);
            }
            if let Some(before) = before {
                builder = builder.set_before(before);
            }
            if let Some(media_type) = media_type {
                builder = builder.set_media_type(media_type);
            }
            let params = builder.build();
            super::search(params, Some(100), Some(0)).await
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

    view! {
        <nav::NavBar />
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
                                        });
                                } />
                            </p>
                        </div>
                    </div>
                    <div class="level-item">
                        <div class="field">
                            <p class="control">
                                <LocationsChooser add_location=move |label| {
                                    set_selected_locations
                                        .update(|locations| {
                                            locations.insert(label);
                                        });
                                } />
                            </p>
                        </div>
                    </div>
                    <div class="level-item">
                        <div class="field">
                            <p class="control">
                                <div class="field is-horizontal">
                                    <div class="field-label is-normal">
                                        <label class="label">After</label>
                                    </div>
                                    <div class="field-body">
                                        <div class="field">
                                            <p class="control">
                                                <input
                                                    class="input"
                                                    type="date"
                                                    id="after-input"
                                                    value=move || utc_to_date_str(after_date.get())
                                                    node_ref=after_date_input_ref
                                                    on:change=move |ev: Event| {
                                                        ev.stop_propagation();
                                                        let value = after_date_input_ref.get().unwrap().value();
                                                        set_after_date.set(date_str_to_utc(&value));
                                                    }
                                                />
                                            </p>
                                        </div>
                                    </div>
                                </div>
                            </p>
                        </div>
                    </div>
                    <div class="level-item">
                        <div class="field">
                            <p class="control">
                                <div class="field is-horizontal">
                                    <div class="field-label is-normal">
                                        <label class="label">Before</label>
                                    </div>
                                    <div class="field-body">
                                        <div class="field">
                                            <p class="control">
                                                <input
                                                    class="input"
                                                    type="date"
                                                    id="before-input"
                                                    value=move || utc_to_date_str(before_date.get())
                                                    node_ref=before_date_input_ref
                                                    on:change=move |ev: Event| {
                                                        ev.stop_propagation();
                                                        let value = before_date_input_ref.get().unwrap().value();
                                                        set_before_date.set(date_str_to_utc(&value));
                                                    }
                                                />
                                            </p>
                                        </div>
                                    </div>
                                </div>
                            </p>
                        </div>
                    </div>
                    <div class="level-item">
                        <div class="field">
                            <p class="control">
                                <TypesChooser
                                    selected_type
                                    set_type=move |value| {
                                        set_selected_type.set(value);
                                    }
                                />
                            </p>
                        </div>
                    </div>
                </div>
            </nav>
        </div>

        <div class="container my-3">
            <nav class="level">
                <div class="level-left">
                    <div class="level-item">
                        <div class="field is-grouped is-grouped-multiline">
                            <forms::TagList
                                attrs=selected_tags
                                rm_attr=move |attr| {
                                    set_selected_tags
                                        .update(|coll| {
                                            coll.remove(&attr);
                                        });
                                }
                            />
                            <forms::TagList
                                attrs=selected_locations
                                rm_attr=move |attr| {
                                    set_selected_locations
                                        .update(|coll| {
                                            coll.remove(&attr);
                                        });
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
                <label class="label" style="text-align: left;">
                    Set Date and Time
                </label>
                <div class="field-body">
                    <div class="field">
                        <p class="control is-expanded">
                            <input
                                class="input"
                                type="datetime-local"
                                node_ref=datetime_input_ref
                            />
                        </p>
                    </div>
                </div>
            </div>

            <TagsEditForm del_tags add_tags />

            <div class="mb-2 field">
                <label class="label" style="text-align: left;">
                    Set Location
                </label>
                <div class="field-body">
                    <div class="field">
                        <p class="control is-expanded">
                            <input
                                class="input"
                                type="text"
                                node_ref=location_input_ref
                                placeholder="Description"
                            />
                        </p>
                    </div>
                    <div class="field">
                        <p class="control is-expanded">
                            <input
                                class="input"
                                type="text"
                                node_ref=city_input_ref
                                placeholder="City"
                            />
                        </p>
                    </div>
                    <div class="field">
                        <p class="control is-expanded">
                            <input
                                class="input"
                                type="text"
                                node_ref=region_input_ref
                                placeholder="Region"
                            />
                        </p>
                    </div>
                </div>
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
                                    <label class="label" style="text-align: left;">
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
                                    <label class="label" style="text-align: left;">
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
                    let src = format!("/rest/thumbnail/400/400/{}", asset.get_value().asset_id);
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

#[component]
fn TagsChooser<F>(add_tag: F) -> impl IntoView
where
    F: Fn(String) + Copy + 'static,
{
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
            let mut results = super::fetch_all_locations().await;
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
fn TypesChooser<F>(selected_type: Signal<Option<String>>, set_type: F) -> impl IntoView
where
    F: Fn(Option<String>) + Copy + 'static,
{
    // the media types returned from the server are in no particular order
    let types = create_resource(
        || (),
        |_| async move {
            let mut results = super::fetch_types().await;
            if let Ok(data) = results.as_mut() {
                data.sort_by(|a, b| a.label.cmp(&b.label));
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
                types
                    .get()
                    .map(|resp| match resp {
                        Err(err) => {
                            view! { <span>{move || format!("Error: {}", err)}</span> }.into_view()
                        }
                        Ok(data) => {
                            let types = store_value(data);
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
                                                when=move || selected_type.get().is_some()
                                                fallback=move || "Media Type"
                                            >
                                                {move || selected_type.get().unwrap()}
                                            </Show>
                                        </button>
                                    </div>
                                    <div class="dropdown-menu" id="dropdown-menu" role="menu">
                                        <div class="dropdown-content">
                                            <a
                                                class="dropdown-item"
                                                on:click=move |_| {
                                                    set_type(None);
                                                    dropdown_open.set(false)
                                                }
                                            >
                                                Any
                                            </a>
                                            <For
                                                each=move || types.get_value()
                                                key=|t| t.clone().label
                                                let:type_
                                            >
                                                {move || {
                                                    let type_ = store_value(type_.clone());
                                                    view! {
                                                        <a
                                                            class="dropdown-item"
                                                            on:click=move |_| {
                                                                set_type(Some(type_.get_value().label));
                                                                dropdown_open.set(false)
                                                            }
                                                        >
                                                            {move || type_.get_value().label}
                                                        </a>
                                                    }
                                                }}
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

/// Convert a DateTime<Utc> to a date in string format, assuming local time zone.
fn utc_to_date_str(datetime: Option<DateTime<Utc>>) -> String {
    match datetime {
        Some(dt) => {
            // this is quite complicated for some reason
            let local_now = Local::now();
            let naive_utc = dt.naive_utc();
            let datetime_local =
                DateTime::<Local>::from_naive_utc_and_offset(naive_utc, local_now.offset().clone());
            datetime_local.naive_local().format("%Y-%m-%d").to_string()
        }
        None => "".into(),
    }
}

/// Convert a string like "2003-08-30" to a DateTime<Utc>, assuming that the
/// input was for the local time zone. If the value cannot be parsed then `None`
/// is returned.
fn date_str_to_utc(value: &str) -> Option<DateTime<Utc>> {
    let local = chrono::offset::Local::now();
    let datetime_str = format!("{}T00:00{}", value, local.offset().to_string());
    match DateTime::parse_from_str(&datetime_str, "%Y-%m-%dT%H:%M%z") {
        Ok(datetime) => Some(datetime.to_utc()),
        Err(_err) => {
            // log::error!("datetime parse error: {:?}; input: {}", err, value);
            None
        }
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
