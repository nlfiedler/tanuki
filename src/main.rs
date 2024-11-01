//
// Copyright (c) 2024 Nathan Fiedler
//
use actix_cors::Cors;
#[cfg(feature = "ssr")]
use actix_files::{Files, NamedFile};
use actix_multipart::Multipart;
#[cfg(feature = "ssr")]
use actix_web::{
    error::InternalError, http::header, http::StatusCode, middleware, web, App, Either, Error,
    HttpMessage, HttpRequest, HttpResponse, HttpServer,
};
use futures::{StreamExt, TryStreamExt};
use juniper::http::graphiql::graphiql_source;
use juniper::http::GraphQLRequest;
use log::error;
use std::collections::HashMap;
use std::env;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, LazyLock};
use tanuki::data::repositories::geo::find_location_repository;
use tanuki::data::repositories::{BlobRepositoryImpl, RecordRepositoryImpl};
use tanuki::data::sources::{EntityDataSource, EntityDataSourceImpl};
use tanuki::domain::repositories::{BlobRepository, RecordRepository};
use tanuki::domain::usecases::UseCase;
use tanuki::preso::graphql;

#[cfg(test)]
static DEFAULT_DB_PATH: &str = "tmp/test/rocksdb";
#[cfg(not(test))]
static DEFAULT_DB_PATH: &str = "tmp/rocksdb";

#[cfg(test)]
static DEFAULT_ASSETS_PATH: &str = "tmp/test/blobs";
#[cfg(not(test))]
static DEFAULT_ASSETS_PATH: &str = "tmp/blobs";

// Path to the database files.
static DB_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    let path = env::var("DB_PATH").unwrap_or_else(|_| DEFAULT_DB_PATH.to_owned());
    PathBuf::from(path)
});

// Path for uploaded files.
static UPLOAD_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    let path = env::var("UPLOAD_PATH").unwrap_or_else(|_| "tmp/uploads".to_owned());
    PathBuf::from(path)
});

static ASSETS_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    let path = env::var("ASSETS_PATH").unwrap_or_else(|_| DEFAULT_ASSETS_PATH.to_owned());
    PathBuf::from(path)
});

// The request body _could_ contain more than one asset, but this endpoint will
// only process a single entity. Returns the newly assigned identifier for the
// updated asset.
#[cfg(feature = "ssr")]
async fn replace_asset(mut payload: Multipart, req: HttpRequest) -> Result<HttpResponse, Error> {
    use tanuki::domain::usecases::replace::{Params, ReplaceAsset};
    let asset_id: String = req.match_info().get("id").unwrap().to_owned();
    // prepare resources for the replace usecase
    let source = EntityDataSourceImpl::new(DB_PATH.as_path())
        .map_err(|e| InternalError::new(e, StatusCode::INTERNAL_SERVER_ERROR))?;
    let ctx: Arc<dyn EntityDataSource> = Arc::new(source);
    let records = Arc::new(RecordRepositoryImpl::new(ctx));
    let blobs = Arc::new(BlobRepositoryImpl::new(ASSETS_PATH.as_path()));
    let geocoder = find_location_repository();
    // process one asset upload and return a list with updated identifier
    let mut asset_ids: Vec<String> = vec![];
    if let Ok(Some(mut field)) = payload.try_next().await {
        let disposition = field.content_disposition();
        let content_type = field
            .content_type()
            .unwrap_or(&mime::APPLICATION_OCTET_STREAM)
            .to_owned();
        let filename = disposition
            .ok_or(actix_web::error::ContentTypeError::ParseError)?
            .get_filename()
            .ok_or(actix_web::error::PayloadError::EncodingCorrupted)?;
        let mut filepath = UPLOAD_PATH.clone();
        filepath.push(filename);
        let filepath_clone = filepath.clone();
        // File operations are blocking, use threadpool
        let mut f = web::block(|| {
            std::fs::create_dir_all(UPLOAD_PATH.as_path())?;
            std::fs::File::create(filepath)
        })
        .await??;
        // each Field is a stream of *Bytes* object
        while let Some(chunk) = field.next().await {
            let data = chunk?;
            // filesystem operations are blocking, we have to use threadpool
            f = web::block(move || f.write_all(&data).map(|_| f)).await??;
        }
        let result = web::block(move || {
            let usecase = ReplaceAsset::new(records, blobs, geocoder);
            let params = Params::new(asset_id, filepath_clone, content_type);
            usecase.call(params)
        })
        .await?;
        match result {
            Ok(asset) => asset_ids.push(asset.key),
            Err(err) => {
                error!("error replacing file: {}", err);
                return Err(InternalError::new(err, StatusCode::INTERNAL_SERVER_ERROR).into());
            }
        }
    }
    let mut output: HashMap<String, Vec<String>> = HashMap::new();
    output.insert("ids".into(), asset_ids);
    let body = serde_json::to_string(&output)?;
    Ok(HttpResponse::Ok().body(body))
}

#[cfg(feature = "ssr")]
async fn import_assets(mut payload: Multipart) -> Result<HttpResponse, Error> {
    use tanuki::domain::usecases::import::{ImportAsset, Params};
    // prepare resources for the import usecase
    let source = EntityDataSourceImpl::new(DB_PATH.as_path())
        .map_err(|e| InternalError::new(e, StatusCode::INTERNAL_SERVER_ERROR))?;
    let ctx: Arc<dyn EntityDataSource> = Arc::new(source);
    let records = Arc::new(RecordRepositoryImpl::new(ctx));
    let blobs = Arc::new(BlobRepositoryImpl::new(ASSETS_PATH.as_path()));
    let geocoder = find_location_repository();
    // iterate over multipart stream
    let mut asset_ids: Vec<String> = Vec::new();
    while let Ok(Some(mut field)) = payload.try_next().await {
        let disposition = field.content_disposition();
        let content_type = field
            .content_type()
            .unwrap_or(&mime::APPLICATION_OCTET_STREAM)
            .to_owned();
        let filename = disposition
            .ok_or(actix_web::error::ContentTypeError::ParseError)?
            .get_filename()
            .ok_or(actix_web::error::PayloadError::EncodingCorrupted)?;
        let mut filepath = UPLOAD_PATH.clone();
        filepath.push(filename);
        let filepath_clone = filepath.clone();
        // File operations are blocking, use threadpool
        let mut f = web::block(|| {
            std::fs::create_dir_all(UPLOAD_PATH.as_path())?;
            std::fs::File::create(filepath)
        })
        .await??;
        // each Field is a stream of *Bytes* object
        while let Some(chunk) = field.next().await {
            let data = chunk?;
            // filesystem operations are blocking, we have to use threadpool
            f = web::block(move || f.write_all(&data).map(|_| f)).await??;
        }
        let records_1 = records.clone();
        let blobs_1 = blobs.clone();
        let geocoder_1 = geocoder.clone();
        let result = web::block(move || {
            let usecase = ImportAsset::new(records_1, blobs_1, geocoder_1);
            let params = Params::new(filepath_clone, content_type);
            usecase.call(params)
        })
        .await?;
        match result {
            Ok(asset) => asset_ids.push(asset.key),
            Err(err) => {
                error!("error importing file: {}", err);
                return Err(InternalError::new(err, StatusCode::INTERNAL_SERVER_ERROR).into());
            }
        }
    }
    let mut output: HashMap<String, Vec<String>> = HashMap::new();
    output.insert("ids".into(), asset_ids);
    let body = serde_json::to_string(&output)?;
    Ok(HttpResponse::Ok().body(body))
}

#[cfg(feature = "ssr")]
async fn graphiql() -> actix_web::Result<HttpResponse> {
    let html = graphiql_source("/graphql", None);
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html))
}

#[cfg(feature = "ssr")]
async fn graphql(
    st: web::Data<(Arc<graphql::Schema>, leptos::LeptosOptions)>,
    data: web::Json<GraphQLRequest>,
) -> actix_web::Result<HttpResponse> {
    let source = EntityDataSourceImpl::new(DB_PATH.as_path()).unwrap();
    let datasource: Arc<dyn EntityDataSource> = Arc::new(source);
    let ctx = Arc::new(graphql::GraphContext::new(
        datasource,
        Box::new(ASSETS_PATH.clone()),
    ));
    let res = data.execute(&st.0, &ctx).await;
    let body = serde_json::to_string(&res)?;
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(body))
}

// Produce a thumbnail for the asset of the requested size.
#[cfg(feature = "ssr")]
async fn get_thumbnail(req: HttpRequest) -> actix_web::Result<HttpResponse> {
    // => /rest/thumbnail/{w}/{h}/{id}
    let width: u32 = req.match_info().get("w").unwrap().parse().unwrap();
    let height: u32 = req.match_info().get("h").unwrap().parse().unwrap();
    let identifier: String = req.match_info().get("id").unwrap().to_owned();
    let etag_value = format!("{}:{}:{}", width, height, &identifier);
    let etag: header::EntityTag = header::EntityTag::new_strong(etag_value);
    if none_match(&etag, &req) {
        let result = web::block(move || {
            let blobs = BlobRepositoryImpl::new(ASSETS_PATH.as_path());
            blobs.thumbnail(width, height, &identifier)
        })
        .await?;
        match result {
            Ok(data) => Ok(HttpResponse::Ok()
                .content_type("image/jpeg")
                .append_header((header::CONTENT_LENGTH, data.len() as u64))
                .append_header((header::ETAG, etag))
                .body(data)),
            Err(err) => {
                error!("get_thumbnail result: {}", err);
                Ok(HttpResponse::NotFound().finish())
            }
        }
    } else {
        Ok(HttpResponse::NotModified().finish())
    }
}

// Returns true if `req` does not have an `If-None-Match` header matching `etag`.
#[cfg(feature = "ssr")]
fn none_match(etag: &header::EntityTag, req: &HttpRequest) -> bool {
    match req.get_header::<header::IfNoneMatch>() {
        Some(header::IfNoneMatch::Any) => false,
        Some(header::IfNoneMatch::Items(ref items)) => {
            for item in items {
                if item.weak_eq(etag) {
                    return false;
                }
            }
            true
        }
        None => true,
    }
}

// Fetching an asset will either return the file or return a 404.
#[cfg(feature = "ssr")]
type AssetResponse = Either<NamedFile, HttpResponse>;

// Return the full asset data and its media type.
#[cfg(feature = "ssr")]
async fn raw_asset(info: web::Path<String>) -> actix_web::Result<AssetResponse> {
    let result = web::block(move || {
        let source = EntityDataSourceImpl::new(DB_PATH.as_path()).unwrap();
        let ctx: Arc<dyn EntityDataSource> = Arc::new(source);
        let records = RecordRepositoryImpl::new(ctx);
        records.get_asset(&info)
    })
    .await?;
    if let Ok(asset) = result {
        let blobs = BlobRepositoryImpl::new(ASSETS_PATH.as_path());
        if let Ok(filepath) = blobs.blob_path(&asset.key) {
            // the browser uses whatever name is given here, despite the
            // `download` attribute on the a href element
            let file = std::fs::File::open(filepath)?;
            let named_file = NamedFile::from_file(file, asset.filename)?;
            let mime_type: mime::Mime = asset.media_type.parse().unwrap();
            Ok(Either::Left(named_file.set_content_type(mime_type)))
        } else {
            Ok(Either::Right(HttpResponse::InternalServerError().finish()))
        }
    } else {
        Ok(Either::Right(HttpResponse::NotFound().finish()))
    }
}

#[cfg(feature = "ssr")]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use leptos::*;
    use leptos_actix::{generate_route_list, LeptosRoutes};
    use tanuki::preso::leptos::*;

    let conf = get_configuration(None).await.unwrap();
    let addr = conf.leptos_options.site_addr;
    let routes = generate_route_list(App);

    dotenv::dotenv().ok();
    env_logger::init();
    HttpServer::new(move || {
        let leptos_options = &conf.leptos_options;
        let site_root = &leptos_options.site_root;
        let schema = Arc::new(graphql::create_schema());
        App::new()
            .app_data(web::Data::new((schema, leptos_options.to_owned())))
            .wrap(middleware::Logger::default())
            .wrap(
                // Respond to OPTIONS requests for CORS support, which is common
                // with some GraphQL clients, including the Dart package.
                Cors::default()
                    .allow_any_origin()
                    .allowed_methods(vec!["GET", "POST"])
                    .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
                    .allowed_header(header::CONTENT_TYPE)
                    .max_age(3600),
            )
            // serve up the compiled static assets
            .service(
                Files::new("/pkg", format!("{site_root}/pkg"))
                    .use_etag(true)
                    .use_last_modified(true),
            )
            // serve up the raw static assets
            .service(
                Files::new("/assets", site_root)
                    .use_etag(true)
                    .use_last_modified(true),
            )
            .service(web::resource("/graphql").route(web::post().to(graphql)))
            .service(web::resource("/graphiql").route(web::get().to(graphiql)))
            .service(favicon)
            // use a different path than /api which Leptos uses by default
            .route("/rest/thumbnail/{w}/{h}/{id}", web::get().to(get_thumbnail))
            .route("/rest/asset/{id}", web::get().to(raw_asset))
            .route("/rest/asset/{id}", web::head().to(raw_asset))
            .route("/rest/import", web::post().to(import_assets))
            .route("/rest/replace/{id}", web::post().to(replace_asset))
            .leptos_routes(leptos_options.to_owned(), routes.to_owned(), App)
    })
    .bind(addr)?
    .run()
    .await
}

#[cfg(not(any(feature = "ssr", feature = "csr")))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
    // see optional feature `csr` instead
}

#[cfg(all(not(feature = "ssr"), feature = "csr"))]
pub fn main() {
    // a client-side main function is required for using `trunk serve`
    // prefer using `cargo leptos serve` instead
    // to run: `trunk serve --open --features csr`
    use crate::preso::leptos::*;
    console_error_panic_hook::set_once();
    leptos::mount_to_body(App);
}

#[cfg(feature = "ssr")]
#[actix_web::get("favicon.ico")]
async fn favicon(
    st: web::Data<(Arc<graphql::Schema>, leptos::LeptosOptions)>,
) -> actix_web::Result<actix_files::NamedFile> {
    let site_root = &st.1.site_root;
    Ok(actix_files::NamedFile::open(format!(
        "{site_root}/favicon.ico"
    ))?)
}

#[cfg(feature = "ssr")]
#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{http, test, web, App};
    use base64::{engine::general_purpose, Engine as _};
    use tanuki::data::repositories::geo::DummyLocationRepository;
    use tanuki::domain::usecases::checksum_file;

    #[actix_web::test]
    async fn test_import_assets_ok() {
        let boundary = "----WebKitFormBoundary0gYa4NfETro6nMot";
        // arrange
        let mut app =
            test::init_service(App::new().route("/import", web::post().to(import_assets))).await;
        // act
        let ct_header = format!("multipart/form-data; boundary={}", boundary);
        let filename = "./tests/fixtures/fighting_kittens.jpg";
        let raw_file = std::fs::read(filename).unwrap();
        let mut payload: Vec<u8> = Vec::new();
        let mut boundary_before = String::from("--");
        boundary_before.push_str(boundary);
        boundary_before.push_str("\r\nContent-Disposition: form-data;");
        boundary_before.push_str(r#" name="asset"; filename="kittens.jpg""#);
        boundary_before.push_str("\r\nContent-Type: image/jpeg\r\n\r\n");
        payload.write(boundary_before.as_bytes()).unwrap();
        payload.write(&raw_file).unwrap();
        let mut boundary_after = String::from("\r\n--");
        boundary_after.push_str(boundary);
        boundary_after.push_str("--\r\n");
        payload.write(boundary_after.as_bytes()).unwrap();
        let req = test::TestRequest::with_uri("/import")
            .method(http::Method::POST)
            .append_header((header::CONTENT_TYPE, ct_header))
            .append_header((header::CONTENT_LENGTH, payload.len()))
            .set_payload(payload)
            .to_request();
        let mut resp: HashMap<String, Vec<String>> =
            test::call_and_read_body_json(&mut app, req).await;
        // assert
        let ids: Vec<String> = resp.remove("ids").unwrap();
        assert_eq!(ids.len(), 1);
        // should be one identifier that is base64 encoded and the path and filename
        // will change over time so can only really check the extension
        let decoded = general_purpose::STANDARD.decode(&ids[0]).unwrap();
        assert!(decoded.ends_with(b".jpg"));
    }

    #[actix_web::test]
    async fn test_replace_asset_ok() {
        use tanuki::domain::usecases::import::{ImportAsset, Params};
        // arrange
        let src_filename = "./tests/fixtures/f1t.jpg";
        let mut filepath = UPLOAD_PATH.clone();
        std::fs::create_dir_all(&filepath).unwrap();
        filepath.push("f1t.jpg");
        std::fs::copy(src_filename, &filepath).unwrap();
        let source = EntityDataSourceImpl::new(DB_PATH.as_path()).unwrap();
        let ctx: Arc<dyn EntityDataSource> = Arc::new(source);
        let blobs = BlobRepositoryImpl::new(ASSETS_PATH.as_path());
        let records = RecordRepositoryImpl::new(ctx);
        if let Ok(Some(asset)) = records.get_asset_by_digest(
            "sha256-c52b9501d1037c50c8d20969a36a888b71310ff90ee557f813330144d8377b18",
        ) {
            // clean up previous test runs
            records.delete_asset(&asset.key).unwrap();
        }
        let geocoder = DummyLocationRepository::new();
        let usecase = ImportAsset::new(Arc::new(records), Arc::new(blobs), Arc::new(geocoder));
        let params = Params::new(filepath, mime::IMAGE_JPEG);
        let asset = usecase.call(params).unwrap();
        let blobs = BlobRepositoryImpl::new(ASSETS_PATH.as_path());
        let blob_path = blobs.blob_path(&asset.key).unwrap();
        let digest = checksum_file(&blob_path).unwrap();
        assert_eq!(
            digest,
            "sha256-5514da7cbe82ef4a0c8dd7c025fba78d8ad085b47ae8cee74fb87705b3d0a630"
        );
        // act
        let mut app =
            test::init_service(App::new().route("/replace/{id}", web::post().to(replace_asset)))
                .await;
        let boundary = "----WebKitFormBoundary0gYa4NfETro6nMot";
        let ct_header = format!("multipart/form-data; boundary={}", boundary);
        let filename = "./tests/fixtures/f2t.jpg";
        let raw_file = std::fs::read(filename).unwrap();
        let mut payload: Vec<u8> = Vec::new();
        let mut boundary_before = String::from("--");
        boundary_before.push_str(boundary);
        boundary_before.push_str("\r\nContent-Disposition: form-data;");
        boundary_before.push_str(r#" name="asset"; filename="f2t.jpg""#);
        boundary_before.push_str("\r\nContent-Type: image/jpeg\r\n\r\n");
        payload.write(boundary_before.as_bytes()).unwrap();
        payload.write(&raw_file).unwrap();
        let mut boundary_after = String::from("\r\n--");
        boundary_after.push_str(boundary);
        boundary_after.push_str("--\r\n");
        payload.write(boundary_after.as_bytes()).unwrap();
        let uri = format!("/replace/{}", asset.key);
        let req = test::TestRequest::with_uri(&uri)
            .method(http::Method::POST)
            .append_header((header::CONTENT_TYPE, ct_header))
            .append_header((header::CONTENT_LENGTH, payload.len()))
            .set_payload(payload)
            .to_request();
        let mut resp: HashMap<String, Vec<String>> =
            test::call_and_read_body_json(&mut app, req).await;
        // assert
        let ids: Vec<String> = resp.remove("ids").unwrap();
        assert_eq!(ids.len(), 1);
        // should be one identifier that is base64 encoded and the path and filename
        // will change over time so can only really check the extension
        let decoded = general_purpose::STANDARD.decode(&ids[0]).unwrap();
        assert!(decoded.ends_with(b".jpg"));
        let blob_path = blobs.blob_path(&ids[0]).unwrap();
        let digest = checksum_file(&blob_path).unwrap();
        assert_eq!(
            digest,
            "sha256-c52b9501d1037c50c8d20969a36a888b71310ff90ee557f813330144d8377b18"
        );
    }

    #[actix_web::test]
    async fn test_get_thumbnail_ok() {
        use tanuki::domain::usecases::import::{ImportAsset, Params};
        // arrange
        let src_filename = "./tests/fixtures/dcp_1069.jpg";
        let mut filepath = UPLOAD_PATH.clone();
        std::fs::create_dir_all(&filepath).unwrap();
        filepath.push("dcp_1069.jpg");
        std::fs::copy(src_filename, &filepath).unwrap();
        let source = EntityDataSourceImpl::new(DB_PATH.as_path()).unwrap();
        let ctx: Arc<dyn EntityDataSource> = Arc::new(source);
        let records = RecordRepositoryImpl::new(ctx);
        let blobs = BlobRepositoryImpl::new(ASSETS_PATH.as_path());
        let geocoder = DummyLocationRepository::new();
        let usecase = ImportAsset::new(Arc::new(records), Arc::new(blobs), Arc::new(geocoder));
        let params = Params::new(filepath, mime::IMAGE_JPEG);
        let asset = usecase.call(params).unwrap();
        let mut app = test::init_service(
            App::new().route("/thumbnail/{w}/{h}/{id}", web::get().to(get_thumbnail)),
        )
        .await;
        // act
        let uri = format!("/thumbnail/128/128/{}", asset.key);
        let req = test::TestRequest::with_uri(&uri).to_request();
        let resp = test::call_service(&mut app, req).await;
        // assert
        assert!(resp.status().is_success());
        assert!(resp.headers().contains_key(header::ETAG));
        assert_eq!(
            resp.headers().get(header::CONTENT_TYPE).unwrap(),
            "image/jpeg"
        );
        let etag = resp.headers().get(header::ETAG).unwrap();

        // assert the etag/if-none-match functionality
        let req = test::TestRequest::with_uri(&uri)
            .append_header((header::ETAG, etag.to_str().unwrap()))
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        assert!(resp.status().is_success());
    }
}
