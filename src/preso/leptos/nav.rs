//
// Copyright (c) 2024 Nathan Fiedler
//
use crate::preso::leptos::get_count;
use leptos::html::Div;
use leptos::prelude::*;
use leptos_use::{
    on_click_outside, use_color_mode_with_options, ColorMode, UseColorModeOptions,
    UseColorModeReturn,
};

#[component]
pub fn NavBar() -> impl IntoView {
    let dropdown_open = RwSignal::new(false);
    let dropdown_ref: NodeRef<Div> = NodeRef::new();
    let _ = on_click_outside(dropdown_ref, move |_| dropdown_open.set(false));
    let UseColorModeReturn { mode, set_mode, .. } = use_color_mode_with_options(
        UseColorModeOptions::default()
            .attribute("data-theme")
            .cookie_enabled(true),
    );

    view! {
        <nav class="navbar" role="navigation" aria-label="main navigation">
            <div class="navbar-brand">
                <img class="navbar-item" src="/assets/tanuki.png" width="80" height="80" />
                <a
                    role="button"
                    class="navbar-burger"
                    aria-label="menu"
                    aria-expanded="false"
                    data-target="navbarBasicExample"
                >
                    <span aria-hidden="true"></span>
                    <span aria-hidden="true"></span>
                    <span aria-hidden="true"></span>
                    <span aria-hidden="true"></span>
                </a>
            </div>

            <div id="navbarBasicExample" class="navbar-menu">
                <div class="navbar-start">
                    <a class="navbar-item" href="/">
                        Browse
                    </a>

                    <a class="navbar-item" href="/search">
                        Search
                    </a>

                    <a class="navbar-item" href="/upload">
                        Upload
                    </a>

                    <a class="navbar-item" href="/pending">
                        Pending
                    </a>

                    <a class="navbar-item" href="/edit">
                        Edit
                    </a>
                </div>

                <div class="navbar-end">
                    <div class="navbar-item">
                        <div
                            class="dropdown is-right"
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
                                        <i
                                            class=move || {
                                                if mode.get() == ColorMode::Dark {
                                                    "fa-solid fa-moon"
                                                } else {
                                                    "fa-solid fa-sun"
                                                }
                                            }
                                            aria-hidden="true"
                                        ></i>
                                    </span>
                                </button>
                            </div>
                            <div class="dropdown-menu" id="dropdown-menu" role="menu">
                                <div class="dropdown-content">
                                    <a
                                        class=move || {
                                            if mode.get() == ColorMode::Light {
                                                "dropdown-item is-active"
                                            } else {
                                                "dropdown-item"
                                            }
                                        }
                                        on:click=move |_| {
                                            set_mode.set(ColorMode::Light);
                                            dropdown_open.set(false)
                                        }
                                    >
                                        <span class="icon">
                                            <i class="fa-solid fa-sun" aria-hidden="true"></i>
                                        </span>
                                        <span>Light</span>
                                    </a>
                                    <a
                                        class=move || {
                                            if mode.get() == ColorMode::Dark {
                                                "dropdown-item is-active"
                                            } else {
                                                "dropdown-item"
                                            }
                                        }
                                        on:click=move |_| {
                                            set_mode.set(ColorMode::Dark);
                                            dropdown_open.set(false)
                                        }
                                    >
                                        <span class="icon">
                                            <i class="fa-solid fa-moon" aria-hidden="true"></i>
                                        </span>
                                        <span>Dark</span>
                                    </a>
                                    <a
                                        class=move || {
                                            if mode.get() == ColorMode::Auto {
                                                "dropdown-item is-active"
                                            } else {
                                                "dropdown-item"
                                            }
                                        }
                                        on:click=move |_| {
                                            set_mode.set(ColorMode::Auto);
                                            dropdown_open.set(false)
                                        }
                                    >
                                        <span class="icon">
                                            <i class="fa-solid fa-desktop" aria-hidden="true"></i>
                                        </span>
                                        <span>System</span>
                                    </a>
                                </div>
                            </div>
                        </div>
                    </div>
                    <div class="navbar-item">
                        <AssetCount />
                    </div>
                </div>
            </div>
        </nav>
    }
}

#[component]
fn AssetCount() -> impl IntoView {
    let count = Resource::new(|| (), |_| async move { get_count().await });

    view! {
        <Transition fallback=move || {
            view! { "Loading..." }
        }>
            <span>
                {move || {
                    let count = count.get().and_then(Result::ok).unwrap_or_default();
                    format!("{} assets", count)
                }}
            </span>
        </Transition>
    }
}
