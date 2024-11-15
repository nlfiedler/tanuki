//
// Copyright (c) 2024 Nathan Fiedler
//
use crate::domain::entities::Location;
use crate::preso::leptos::SearchMeta;
use chrono::{DateTime, Utc};
use leptos::*;

#[component]
pub fn ResultsDisplay<F>(meta: SearchMeta, onclick: F) -> impl IntoView
where
    F: Fn(usize) + Copy + 'static,
{
    // store the results in the reactive system so the view can be Fn()
    let results = store_value(meta.results);
    view! {
        <div class="grid is-col-min-16 padding-2">
            <For
                each=move || results.get_value().into_iter().enumerate().map(|(i, r)| store_value((i, r)))
                key=|r| r.get_value().1.asset_id
                let:elem
            >
                <div class="cell">
                    <a on:click=move |_| onclick(elem.get_value().0)>
                        <div class="card">
                            <div class="card-image">
                                <figure class="image">
                                    {move || {
                                        let asset = elem.get_value().1;
                                        if asset.media_type.starts_with("video/") {
                                            let src = format!("/rest/asset/{}", asset.asset_id);
                                            let mut media_type = asset.media_type.clone();
                                            if media_type == "video/quicktime" {
                                                media_type = "video/mp4".into();
                                            }
                                            view! {
                                                <video controls>
                                                    <source src=src type=media_type />
                                                    Bummer, your browser does not support the HTML5
                                                    <code>video</code>
                                                    tag.
                                                </video>
                                            }
                                                .into_view()
                                        } else if asset.media_type.starts_with("audio/") {
                                            let src = format!("/rest/asset/{}", asset.asset_id);
                                            view! {
                                                <figcaption>{move || asset.filename.clone()}</figcaption>
                                                <audio controls>
                                                    <source src=src type=asset.media_type.clone() />
                                                </audio>
                                            }
                                                .into_view()
                                        } else {
                                            let src = format!(
                                                "/rest/thumbnail/960/960/{}",
                                                asset.asset_id,
                                            );
                                            view! {
                                                <img
                                                    src=src
                                                    alt=asset.filename.clone()
                                                    style="max-width: 100%; width: auto; padding: inherit; margin: auto; display: block;"
                                                />
                                            }
                                                .into_view()
                                        }
                                    }}
                                </figure>
                            </div>
                            <div class="card-content">
                                <div class="content">
                                    <CardContent
                                        datetime=elem.get_value().1.datetime
                                        location=elem.get_value().1.location
                                    />
                                </div>
                            </div>
                        </div>
                    </a>
                </div>
            </For>
        </div>
    }
}

#[component]
fn CardContent(datetime: DateTime<Utc>, location: Option<Location>) -> impl IntoView {
    let datetime = store_value(datetime);
    let location = store_value(location);
    view! {
        <div class="content">
            <time>{move || { datetime.get_value().format("%A %B %e, %Y").to_string() }}</time>
            <Show when=move || location.get_value().is_some() fallback=|| ()>
                <br />
                <span>{move || location.get_value().unwrap().to_string()}</span>
            </Show>
        </div>
    }
}
