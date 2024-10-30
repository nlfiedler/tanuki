//
// Copyright (c) 2024 Nathan Fiedler
//
use crate::preso::leptos::nav;
use leptos::html::Div;
use leptos::*;
use leptos_use::{use_drop_zone_with_options, UseDropZoneOptions, UseDropZoneReturn};
use web_sys::wasm_bindgen::JsCast;

///
/// Import files in the `uploads` directory as if uplaoded via the browser.
///
#[leptos::server]
pub async fn ingest() -> Result<u32, ServerFnError> {
    use crate::data::repositories::geo::find_location_repository;
    use crate::domain::usecases::ingest::{IngestAssets, Params};
    use crate::domain::usecases::UseCase;
    use std::path::PathBuf;
    use std::sync::Arc;

    let repo = super::ssr::db()?;
    let blobs = super::ssr::blobs()?;
    let geocoder = find_location_repository();
    let usecase = IngestAssets::new(Arc::new(repo), Arc::new(blobs), geocoder);
    let path = std::env::var("UPLOAD_PATH").unwrap_or_else(|_| "tmp/uploads".to_owned());
    let uploads_path = PathBuf::from(path);
    let params = Params::new(uploads_path);
    let count = usecase
        .call(params)
        .map_err(|e| leptos::ServerFnErrorErr::WrappedServerError(e))?;
    Ok(count as u32)
}

#[component]
pub fn UploadPage() -> impl IntoView {
    // boolean indicating if user is dragging files over the drop zone
    let (is_over, set_is_over) = create_signal(false);
    // drop zone element and destructured/renamed fields from the helper
    let drop_zone_el = create_node_ref::<Div>();
    let UseDropZoneReturn {
        is_over_drop_zone: _ignored,
        files: dropped_files,
    } = use_drop_zone_with_options(
        drop_zone_el,
        UseDropZoneOptions::default()
            .on_enter(move |_| set_is_over.set(true))
            .on_drop(move |_| set_is_over.set(false))
            .on_leave(move |_| set_is_over.set(false)),
    );
    // files selected from the file input element
    let (selected_files, set_selected_files) = create_signal::<Vec<web_sys::File>>(vec![]);
    // combination of the dropped files and those from the file selector
    let all_files = create_memo(move |_| {
        with!(|dropped_files, selected_files| {
            let mut copy: Vec<web_sys::File> = dropped_files.iter().cloned().collect();
            for select in selected_files {
                copy.push(select.to_owned());
            }
            copy.sort_by(|a, b| a.name().cmp(&b.name()));
            copy.dedup_by(|a, b| a.name() == b.name());
            copy
        })
    });
    let has_files = move || all_files.get().len() > 0;
    let drop_style = move || {
        format!(
            "border-style: dashed; min-height: 14em; {}",
            if is_over.get() {
                "border-color: green;"
            } else {
                ""
            }
        )
    };
    // import any files in the 'uploads' directory
    let import_action = create_action(move |_input: &()| import_files());
    // indicates number of files uploaded so far as percentage
    let (progress, set_progress) = create_signal(0);
    // upload the files asynchronously and update the progress
    let upload_action =
        create_action(move |_input: &()| upload_files(all_files.get(), set_progress));
    // merge chosen files with any that were selected previously
    let files_selected = move |ev| {
        let mut selected = file_event_to_file_vec(ev);
        let mut copy = selected_files.get();
        copy.append(&mut selected);
        copy.sort_by(|a, b| a.name().cmp(&b.name()));
        copy.dedup_by(|a, b| a.name() == b.name());
        set_selected_files.set(copy);
    };

    view! {
        <nav::NavBar />

        <div class="container">
            <nav class="level">
                <div class="level-left">
                    <div class="level-item">
                        <button
                            class="button"
                            class:is-loading=move || import_action.pending().get()
                            disabled=move || import_action.pending().get()
                            on:click=move |_| import_action.dispatch(())
                        >
                            Import
                        </button>
                        <div class="block ml-2">
                            <p class="subtitle is-5">from the <code>uploads</code>directory</p>
                        </div>
                    </div>
                </div>

                <div class="level-right">
                    <div class="level-item">
                        <div class="file">
                            <label class="file-label">
                                <input
                                    class="file-input"
                                    type="file"
                                    id="file-input"
                                    multiple
                                    name="uploads"
                                    disabled=move || upload_action.pending().get()
                                    on:input=files_selected
                                />
                                <span class="file-cta">
                                    <span class="file-icon">
                                        <i class="fas fa-upload"></i>
                                    </span>
                                    <span class="file-label">Choose Files</span>
                                </span>
                            </label>
                        </div>
                    </div>
                    <div class="level-item">
                        <button
                            class="button"
                            disabled=move || {
                                has_files() == false || upload_action.pending().get()
                            }
                            on:click=move |_| upload_action.dispatch(())
                        >
                            Start Upload
                        </button>
                    </div>
                </div>
            </nav>
            <Show
                when=move || upload_action.pending().get()
                fallback=|| view! { <progress class="mt-4 progress" value="0" max="100" /> }
            >
                <progress class="mt-4 progress" value=progress.get() max="100">
                    progress.get().to_string()
                </progress>
            </Show>
        </div>

        <section class="section">
            <p>You can drop files into the drop zone below.</p>
            <div node_ref=drop_zone_el class="content" style=drop_style>
                <Show when=move || has_files() fallback=|| view! {}>
                    <table class="table is-fullwidth">
                        <thead>
                            <tr>
                                <th>File</th>
                                <th>Type</th>
                                <th>Size</th>
                                <th>Date</th>
                            </tr>
                        </thead>
                        <tbody>
                            <For each=move || all_files.get() key=|f| f.name() let:file>
                                <tr>
                                    <td>{file.name()}</td>
                                    <td>{file.type_()}</td>
                                    <td>{file.size()}</td>
                                    <td>{format_time(file.last_modified())}</td>
                                </tr>
                            </For>
                        </tbody>
                        <tfoot>
                            <tr>
                                <th>File</th>
                                <th>Type</th>
                                <th>Size</th>
                                <th>Date</th>
                            </tr>
                        </tfoot>
                    </table>
                </Show>
            </div>
        </section>
    }
}

/// Ingest the files in the `uploads` directory and navigate to pending page
/// upon completion.
async fn import_files() {
    if let Err(err) = ingest().await {
        log::error!("file import failed: {err:#?}");
    }
    // navigate to the pending page for convenience and consistency with the
    // upload button
    let navigate = leptos_router::use_navigate();
    navigate("/pending", Default::default());
}

/// Upload the given files, updating the progress for each one. Navigates
/// to the pending page on completion.
async fn upload_files(files: Vec<web_sys::File>, set_progress: WriteSignal<i32>) {
    let num_files = files.len();
    for (idx, file) in files.into_iter().enumerate() {
        let form_data = web_sys::FormData::new().unwrap();
        let filename = file.name();
        form_data
            .set_with_blob_and_filename("asset", &file, &filename)
            .unwrap();
        let res = gloo::net::http::Request::post("/rest/import")
            .body(form_data)
            .unwrap()
            .send()
            .await;
        if let Err(err) = res {
            log::error!("file upload failed: {err:#?}");
        }
        set_progress.set(((idx + 1) * 100 / num_files) as i32);
    }
    // since we are unable to reset the drop zone with the current API, simply
    // send the browser to the pending page immediately
    let navigate = leptos_router::use_navigate();
    navigate("/pending", Default::default());
}

/// Convert the FileList from the given event into a Vec<File> type.
fn file_event_to_file_vec(ev: leptos::ev::Event) -> Vec<web_sys::File> {
    let files = ev
        .target()
        .unwrap()
        .unchecked_ref::<web_sys::HtmlInputElement>()
        .files()
        .unwrap();
    let mut results: Vec<web_sys::File> = Vec::new();
    for idx in 0..files.length() {
        results.push(files.item(idx).unwrap());
    }
    results
}

/// Format the JavaScript time to something sensible.
fn format_time(time: f64) -> String {
    use chrono::{DateTime, Utc};
    let seconds = time / 1000.0;
    let dt: DateTime<Utc> = DateTime::from_timestamp(seconds as i64, 0).unwrap_or(Utc::now());
    dt.format("%Y-%m-%d").to_string()
}
