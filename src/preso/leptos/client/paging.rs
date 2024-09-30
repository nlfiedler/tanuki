//
// Copyright (c) 2024 Nathan Fiedler
//
use crate::preso::common::SearchMeta;
use leptos::*;

#[component]
pub fn PageControls(
    meta: SearchMeta,
    selected_page: RwSignal<i32>,
    page_size: RwSignal<i32>,
) -> impl IntoView {
    let dropdown_open = create_rw_signal(false);
    let dropdown_class = move || {
        if dropdown_open.get() {
            "dropdown is-active"
        } else {
            "dropdown"
        }
    };
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
                        <button class="button" on:click=move |_| selected_page.update(|p| *p -= 1)>
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
                        <button class="button" on:click=move |_| selected_page.update(|p| *p += 1)>
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
                    <div class=move || dropdown_class()>
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
                                <a
                                    class="dropdown-item"
                                    on:click=move |_| {
                                        batch(|| {
                                            page_size.set(18);
                                            selected_page.set(1);
                                        });
                                        dropdown_open.set(false)
                                    }
                                >
                                    18
                                </a>
                                <a
                                    class="dropdown-item"
                                    on:click=move |_| {
                                        batch(|| {
                                            page_size.set(36);
                                            selected_page.set(1);
                                        });
                                        dropdown_open.set(false)
                                    }
                                >
                                    36
                                </a>
                                <a
                                    class="dropdown-item"
                                    on:click=move |_| {
                                        batch(|| {
                                            page_size.set(54);
                                            selected_page.set(1);
                                        });
                                        dropdown_open.set(false)
                                    }
                                >
                                    54
                                </a>
                                <a
                                    class="dropdown-item"
                                    on:click=move |_| {
                                        batch(|| {
                                            page_size.set(72);
                                            selected_page.set(1);
                                        });
                                        dropdown_open.set(false)
                                    }
                                >
                                    72
                                </a>
                            </div>
                        </div>
                    </div>
                </p>
            </div>
        </div>
    }
}
