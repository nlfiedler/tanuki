//
// Copyright (c) 2024 Nathan Fiedler
//
use crate::preso::leptos::server::get_count;
use leptos::*;
use leptos_use::{use_color_mode_with_options, ColorMode, UseColorModeOptions, UseColorModeReturn};

#[component]
pub fn NavBar() -> impl IntoView {
    let scheme_dropdown_open = create_rw_signal(false);
    let scheme_dropdown_class = move || {
        if scheme_dropdown_open.get() {
            "dropdown is-active"
        } else {
            "dropdown"
        }
    };
    let UseColorModeReturn { mode, set_mode, .. } =
        use_color_mode_with_options(UseColorModeOptions::default().attribute("data-theme"));

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
                        Home
                    </a>

                    <a class="navbar-item" href="/upload">
                        Upload
                    </a>

                    <a class="navbar-item" href="/pending">
                        Pending
                    </a>
                </div>

                <div class="navbar-end">
                    <div class="navbar-item">
                        <div class=move || scheme_dropdown_class()>
                            <div class="dropdown-trigger">
                                <button
                                    class="button"
                                    on:click=move |_| {
                                        scheme_dropdown_open.update(|v| { *v = !*v })
                                    }
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
                                        class="dropdown-item"
                                        on:click=move |_| {
                                            set_mode.set(ColorMode::Light);
                                            scheme_dropdown_open.set(false)
                                        }
                                    >
                                        <span class="icon">
                                            <i class="fa-solid fa-sun" aria-hidden="true"></i>
                                        </span>
                                        <span>Light</span>
                                    </a>
                                    <a
                                        class="dropdown-item"
                                        on:click=move |_| {
                                            set_mode.set(ColorMode::Dark);
                                            scheme_dropdown_open.set(false)
                                        }
                                    >
                                        <span class="icon">
                                            <i class="fa-solid fa-moon" aria-hidden="true"></i>
                                        </span>
                                        <span>Dark</span>
                                    </a>
                                    <a
                                        class="dropdown-item"
                                        on:click=move |_| {
                                            set_mode.set(ColorMode::Auto);
                                            scheme_dropdown_open.set(false)
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
    let count = create_resource(|| (), |_| async move { get_count().await });

    view! {
        <Transition fallback=move || {
            view! { "Loading..." }
        }>
            {move || {
                count
                    .get()
                    .map(|data| match data.clone() {
                        Err(err) => {
                            view! { <span>{move || format!("Error: {}", err)}</span> }.into_view()
                        }
                        Ok(count) => {
                            view! { <span>{move || format!("{} assets", count)}</span> }.into_view()
                        }
                    })
            }}
        </Transition>
    }
}
