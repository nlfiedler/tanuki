//
// Copyright (c) 2024 Nathan Fiedler
//
use crate::preso::leptos::{nav, paging, results};
use codee::string::{FromToStringCodec, JsonSerdeCodec};
use leptos::ev::Event;
use leptos::html::Input;
use leptos::*;
use leptos_use::storage::{use_local_storage_with_options, UseStorageOptions};

#[component]
pub fn SearchPage() -> impl IntoView {
    let (query, set_query, _) = use_local_storage_with_options::<String, JsonSerdeCodec>(
        "search-query",
        UseStorageOptions::default()
            .initial_value(String::new())
            .delay_during_hydration(true),
    );
    // page of results to be displayed (1-based)
    let (selected_page, set_selected_page, _) =
        use_local_storage_with_options::<i32, FromToStringCodec>(
            "search-selected-page",
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
        move || (query.get(), selected_page.get(), page_size.get()),
        |(query_str, page, count)| async move {
            let offset = count * (page - 1);
            super::scan_assets(query_str, None, None, Some(count), Some(offset)).await
        },
    );

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
                                        style="max-width: 400%; width: 400%;"
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
                            view! { <results::ResultsDisplay meta /> }
                        }
                    })
            }}
        </Transition>
    }
}
