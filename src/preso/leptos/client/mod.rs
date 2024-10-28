//
// Copyright (c) 2024 Nathan Fiedler
//
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

mod asset;
mod edit;
mod forms;
mod home;
mod nav;
mod paging;
mod pending;
mod upload;

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/tanuki.css" />
        <Stylesheet href="/assets/fontawesome/css/all.min.css" />
        <Title text="Tanuki" />
        <Router>
            <main>
                <Routes>
                    <Route path="" view=home::HomePage />
                    <Route path="/upload" view=upload::UploadPage />
                    <Route path="/pending" view=pending::PendingPage />
                    <Route path="/edit" view=edit::EditPage />
                    <Route path="/asset/:id" view=asset::AssetPage />
                    <Route path="/*any" view=NotFound />
                </Routes>
            </main>
        </Router>
    }
}

/// 404 - Not Found
#[component]
fn NotFound() -> impl IntoView {
    // set an HTTP status code 404 this is feature gated because it can only be
    // done during initial server-side rendering if you navigate to the 404 page
    // subsequently, the status code will not be set because there is not a new
    // HTTP request to the server
    #[cfg(feature = "ssr")]
    {
        // this can be done inline because it's synchronous if it were async,
        // we'd use a server function
        let resp = expect_context::<leptos_actix::ResponseOptions>();
        resp.set_status(actix_web::http::StatusCode::NOT_FOUND);
    }

    view! {
        <nav::NavBar />
        <section class="section">
            <h1 class="title">Page not found</h1>
            <h2 class="subtitle">This is not the page you are looking for.</h2>
            <div class="content">
                <p>Try using the navigation options above.</p>
            </div>
        </section>
    }
}
