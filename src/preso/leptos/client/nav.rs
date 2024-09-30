//
// Copyright (c) 2024 Nathan Fiedler
//
use crate::preso::leptos::server::get_count;
use leptos::*;

#[component]
pub fn NavBar() -> impl IntoView {
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
