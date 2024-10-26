//
// Copyright (c) 2024 Nathan Fiedler
//
use crate::preso::common::SearchMeta;
use html::Div;
use leptos::*;
use leptos_use::on_click_outside;

#[component]
pub fn PageControls(
    meta: SearchMeta,
    selected_page: Signal<i32>,
    set_selected_page: WriteSignal<i32>,
    page_size: Signal<i32>,
    set_page_size: WriteSignal<i32>,
) -> impl IntoView {
    let dropdown_open = create_rw_signal(false);
    let dropdown_ref = create_node_ref::<Div>();
    let _ = on_click_outside(dropdown_ref, move |_| dropdown_open.set(false));

    view! {
        <div class="level-item">
            <div class="field">
                <p class="control">
                    <Show
                        when=move || (selected_page.get() > 1)
                        fallback=move || {
                            view! {
                                <button class="button" disabled>
                                    <span class="icon">
                                        <i class="fas fa-angle-left"></i>
                                    </span>
                                </button>
                            }
                        }
                    >
                        <button
                            class="button"
                            on:click=move |_| set_selected_page.update(|p| *p -= 1)
                        >
                            <span class="icon">
                                <i class="fas fa-angle-left"></i>
                            </span>
                        </button>
                    </Show>
                </p>
            </div>
        </div>
        <div class="level-item">
            <span>{{ move || format!("Page {} of {}", selected_page.get(), meta.last_page) }}</span>
        </div>
        <div class="level-item">
            <div class="field">
                <p class="control">
                    <Show
                        when=move || (selected_page.get() < meta.last_page)
                        fallback=move || {
                            view! {
                                <button class="button" disabled>
                                    <span class="icon">
                                        <i class="fas fa-angle-right"></i>
                                    </span>
                                </button>
                            }
                        }
                    >
                        <button
                            class="button"
                            on:click=move |_| set_selected_page.update(|p| *p += 1)
                        >
                            <span class="icon">
                                <i class="fas fa-angle-right"></i>
                            </span>
                        </button>
                    </Show>
                </p>
            </div>
        </div>
        <div class="level-item">
            <div class="field">
                <p class="control">
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
                                <span class="icon">
                                    <i class="fa-solid fa-expand" aria-hidden="true"></i>
                                </span>
                            </button>
                        </div>
                        <div class="dropdown-menu" id="dropdown-menu" role="menu">
                            <div class="dropdown-content">
                                <For each=move || [18, 36, 54, 72].iter() key=|i| *i let:size>
                                    <a
                                        class=if page_size.get() == *size {
                                            "dropdown-item is-active"
                                        } else {
                                            "dropdown-item"
                                        }
                                        on:click=move |_| {
                                            batch(|| {
                                                set_page_size.set(*size);
                                                set_selected_page.set(1);
                                            });
                                            dropdown_open.set(false)
                                        }
                                    >
                                        {move || size.to_string()}
                                    </a>
                                </For>
                            </div>
                        </div>
                    </div>
                </p>
            </div>
        </div>
    }
}
