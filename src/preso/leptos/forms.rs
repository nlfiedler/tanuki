//
// Copyright (c) 2024 Nathan Fiedler
//
use leptos::*;
use std::collections::HashSet;

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
