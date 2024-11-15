//
// Copyright (c) 2024 Nathan Fiedler
//
use crate::domain::entities::{SortField, SortOrder};
use crate::preso::leptos::BrowseParams;
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
    let (sort_order, set_sort_order, _) = use_local_storage_with_options::<String, JsonSerdeCodec>(
        "search-sort-order",
        UseStorageOptions::default()
            .initial_value("descending")
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
    // begin browsing assets, starting with the chosen asset; the given index is
    // zero-based within the current page of results
    let browse_asset = create_action(move |idx: &usize| {
        let page = selected_page.get_untracked();
        let count = page_size.get_untracked();
        let offset = count * (page - 1);
        let mut browse = BrowseParams::default();
        let order = sort_order.get_untracked();
        browse.sort_field = Some(SortField::Date);
        browse.sort_order = Some(SortOrder::from(order.as_str()));
        browse.query = Some(query.get_untracked());
        browse.asset_index = *idx + offset as usize;
        begin_browsing(browse)
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
    navigate("/asset", Default::default());
}
