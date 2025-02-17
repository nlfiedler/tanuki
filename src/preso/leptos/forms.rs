//
// Copyright (c) 2024 Nathan Fiedler
//
use html::Div;
use leptos::ev::Event;
use leptos::html::Input;
use leptos::*;
use leptos_use::on_click_outside;
use std::collections::HashSet;

use crate::domain::entities::Location;

/// Show list of selected attributes as tags/chips.
#[component]
pub fn TagList<F>(attrs: Signal<HashSet<String>>, rm_attr: F) -> impl IntoView
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
pub fn TagsChooser<F>(add_tag: F) -> impl IntoView
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
                                        <label class="label" for="tags-input">
                                            Tags
                                        </label>
                                    </div>
                                    <div class="field-body">
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
                            }
                                .into_view()
                        }
                    })
            }}
        </Transition>
    }
}

#[component]
pub fn LocationsChooser<F>(add_location: F) -> impl IntoView
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
                                        <label class="label" for="locations-input">
                                            Locations
                                        </label>
                                    </div>
                                    <div class="field-body">
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
                            }
                                .into_view()
                        }
                    })
            }}
        </Transition>
    }
}

#[component]
pub fn FullLocationChooser<F>(set_location: F) -> impl IntoView
where
    F: Fn(Location) + Copy + 'static,
{
    use std::str::FromStr;
    // the locations returned from the server are in no particular order
    let locations = create_resource(
        || (),
        |_| async move {
            let mut results = super::fetch_raw_locations().await;
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
        let value = input.value();
        set_location(Location::from_str(&value).unwrap());
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
                                        <label class="label" for="locations-input">
                                            Location
                                        </label>
                                    </div>
                                    <div class="field-body">
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
                                                            let value = loc.to_string();
                                                            view! { <option value=value></option> }
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
    }
    .into_view()
}

#[component]
pub fn TypesChooser<F>(selected_type: Signal<Option<String>>, set_type: F) -> impl IntoView
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
