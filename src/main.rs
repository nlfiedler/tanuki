//
// Copyright (c) 2020 Nathan Fiedler
//
use actix_cors::Cors;
use actix_files::{Files, NamedFile};
use actix_multipart::Multipart;
use actix_web::{http, middleware, web, App, Either, Error, HttpRequest, HttpResponse, HttpServer};
use futures::{StreamExt, TryStreamExt};
use juniper::http::graphiql::graphiql_source;
use juniper::http::GraphQLRequest;
use lazy_static::lazy_static;
use log::{debug, info};
use std::env;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use tanuki::data::repositories::{BlobRepositoryImpl, RecordRepositoryImpl};
use tanuki::data::sources::{EntityDataSource, EntityDataSourceImpl};
use tanuki::domain::repositories::{BlobRepository, RecordRepository};
use tanuki::domain::usecases::import::{ImportAsset, Params};
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

lazy_static! {
    // Path to the database files.
    static ref DB_PATH: PathBuf = {
        dotenv::dotenv().ok();
        let path = env::var("DB_PATH").unwrap_or_else(|_| DEFAULT_DB_PATH.to_owned());
        PathBuf::from(path)
    };
    // Path for uploaded files.
    static ref UPLOAD_PATH: PathBuf = {
        let path = env::var("UPLOAD_PATH").unwrap_or_else(|_| "tmp/uploads".to_owned());
        PathBuf::from(path)
    };
    static ref ASSETS_PATH: PathBuf = {
        let path = env::var("ASSETS_PATH").unwrap_or_else(|_| DEFAULT_ASSETS_PATH.to_owned());
        PathBuf::from(path)
    };
    // Path to the static web files.
    static ref STATIC_PATH: PathBuf = {
        let path = env::var("STATIC_FILES").unwrap_or_else(|_| "./public/".to_owned());
        PathBuf::from(path)
    };
    // Path of the fallback page for web requests.
    static ref DEFAULT_INDEX: PathBuf = {
        let mut path = STATIC_PATH.clone();
        path.push("index.html");
        path
    };
}

async fn import_assets(mut payload: Multipart) -> Result<HttpResponse, Error> {
    // iterate over multipart stream
    let mut asset_ids: Vec<String> = Vec::new();
    while let Ok(Some(mut field)) = payload.try_next().await {
        let disposition = field.content_disposition().unwrap();
        let content_type = field.content_type().to_owned();
        let filename = disposition.get_filename().unwrap();
        let mut filepath = UPLOAD_PATH.clone();
        std::fs::create_dir_all(&filepath)?;
        filepath.push(filename);
        let filepath_clone = filepath.clone();
        // File::create is blocking operation, use threadpool
        let mut f = web::block(|| std::fs::File::create(filepath))
            .await
            .unwrap();
        // each Field is a stream of *Bytes* object
        while let Some(chunk) = field.next().await {
            let data = chunk.unwrap();
            // filesystem operations are blocking, we have to use threadpool
            f = web::block(move || f.write_all(&data).map(|_| f)).await?;
        }
        let asset = web::block(move || {
            let source = EntityDataSourceImpl::new(DB_PATH.as_path()).unwrap();
            let ctx: Arc<dyn EntityDataSource> = Arc::new(source);
            let records = RecordRepositoryImpl::new(ctx);
            let blobs = BlobRepositoryImpl::new(ASSETS_PATH.as_path());
            let usecase = ImportAsset::new(Box::new(records), Box::new(blobs));
            let params = Params::new(filepath_clone, content_type);
            usecase.call(params)
        })
        .await?;
        asset_ids.push(asset.key);
    }
    let body = serde_json::to_string(&asset_ids)?;
    Ok(HttpResponse::Ok().body(body))
}

fn graphiql() -> HttpResponse {
    let html = graphiql_source("/graphql");
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

async fn graphql(
    st: web::Data<Arc<graphql::Schema>>,
    data: web::Json<GraphQLRequest>,
) -> actix_web::Result<HttpResponse> {
    let source = EntityDataSourceImpl::new(DB_PATH.as_path()).unwrap();
    let datasource: Arc<dyn EntityDataSource> = Arc::new(source);
    let ctx = Arc::new(graphql::GraphContext::new(
        datasource,
        Box::new(ASSETS_PATH.clone()),
    ));
    let res = data.execute(&st, &ctx);
    let body = serde_json::to_string(&res)?;
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(body))
}

// Produce a thumbnail for the asset of the requested size.
async fn get_thumbnail(info: web::Path<(u32, u32, String)>) -> HttpResponse {
    let etag: String = format!("{}:{}:{}", info.0, info.1, &info.2);
    let result = web::block(move || {
        let blobs = BlobRepositoryImpl::new(ASSETS_PATH.as_path());
        blobs.thumbnail(info.0, info.1, &info.2)
    })
    .await;
    if let Ok(data) = result {
        HttpResponse::Ok()
            .content_type("image/jpeg")
            .content_length(data.len() as u64)
            .header(http::header::ETAG, etag)
            .body(data)
    } else {
        debug!("get_thumbnail result: {:?}", result);
        HttpResponse::NotFound().finish()
    }
}

// Fetching an asset will either return the file or return a 404.
type AssetResponse = Either<NamedFile, HttpResponse>;

// Return the full asset data and its media type.
async fn raw_asset(info: web::Path<String>) -> actix_web::Result<AssetResponse> {
    let result = web::block(move || {
        let source = EntityDataSourceImpl::new(DB_PATH.as_path()).unwrap();
        let ctx: Arc<dyn EntityDataSource> = Arc::new(source);
        let records = RecordRepositoryImpl::new(ctx);
        records.get_asset(&info)
    })
    .await;
    if let Ok(asset) = result {
        let blobs = BlobRepositoryImpl::new(ASSETS_PATH.as_path());
        // other errors are indeed "internal server errors" so let the default
        // error handler raise those to the client
        let filepath = blobs.blob_path(&asset.key)?;
        let file = NamedFile::open(filepath)?;
        let mime_type: mime::Mime = asset.media_type.parse().unwrap();
        Ok(Either::A(file.set_content_type(mime_type)))
    } else {
        Ok(Either::B(HttpResponse::NotFound().finish()))
    }
}

// All requests that fail to match anything else will be directed to the index
// page, where the client-side code will handle the routing and "page not found"
// error condition.
async fn default_index(_req: HttpRequest) -> actix_web::Result<NamedFile> {
    let file = NamedFile::open(DEFAULT_INDEX.as_path())?;
    Ok(file.use_last_modified(true))
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_owned());
    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_owned());
    let addr = format!("{}:{}", host, port);
    info!("listening on http://{}/...", addr);
    HttpServer::new(|| {
        let schema = std::sync::Arc::new(graphql::create_schema());
        App::new()
            .data(schema)
            .wrap(middleware::Logger::default())
            .wrap(
                // Respond to OPTIONS requests for CORS support, which is common
                // with some GraphQL clients, including the Dart package.
                Cors::new()
                    .allowed_methods(vec!["GET", "POST"])
                    .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
                    .allowed_header(http::header::CONTENT_TYPE)
                    .max_age(3600)
                    .finish(),
            )
            .service(web::resource("/graphql").route(web::post().to(graphql)))
            .service(web::resource("/graphiql").route(web::get().to(graphiql)))
            .route("/api/thumbnail/{w}/{h}/{id}", web::get().to(get_thumbnail))
            .route("/api/asset/{id}", web::get().to(raw_asset))
            .route("/api/asset/{id}", web::head().to(raw_asset))
            .route("/api/import", web::post().to(import_assets))
            .service(Files::new("/", STATIC_PATH.clone()).index_file("index.html"))
            .default_service(web::get().to(default_index))
    })
    .bind(addr)?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{http, test, web, App};

    #[actix_rt::test]
    async fn test_index_get() {
        // arrange
        let mut app =
            test::init_service(App::new().default_service(web::get().to(default_index))).await;
        // act
        let req = test::TestRequest::default().to_request();
        let resp = test::call_service(&mut app, req).await;
        // assert
        assert!(resp.status().is_success());
    }

    #[actix_rt::test]
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
            .header(http::header::CONTENT_TYPE, ct_header)
            .header(http::header::CONTENT_LENGTH, payload.len())
            .set_payload(payload)
            .to_request();
        let resp = test::call_service(&mut app, req).await;
        // assert
        assert!(resp.status().is_success());
    }

    #[actix_rt::test]
    async fn test_get_thumbnail_ok() {
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
        let usecase = ImportAsset::new(Box::new(records), Box::new(blobs));
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
        assert!(resp.headers().contains_key(http::header::ETAG));
        assert_eq!(
            resp.headers().get(http::header::CONTENT_TYPE).unwrap(),
            "image/jpeg"
        );
    }
}
