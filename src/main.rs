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
use log::info;
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

lazy_static! {
    // Path to the database files.
    static ref DB_PATH: PathBuf = {
        dotenv::dotenv().ok();
        let path = env::var("DB_PATH").unwrap_or_else(|_| "tmp/rocksdb".to_owned());
        PathBuf::from(path)
    };
    // Path for uploaded files.
    static ref UPLOAD_PATH: PathBuf = {
        let path = env::var("UPLOAD_PATH").unwrap_or_else(|_| "tmp/uploads".to_owned());
        PathBuf::from(path)
    };
    static ref ASSETS_PATH: PathBuf = {
        let path = env::var("ASSETS_PATH").unwrap_or_else(|_| "tmp/blobs".to_owned());
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

async fn import_asset(mut payload: Multipart) -> Result<HttpResponse, Error> {
    // iterate over multipart stream
    let mut asset_ids: Vec<String> = Vec::new();
    // (expecting only a single item for now)
    while let Ok(Some(mut field)) = payload.try_next().await {
        let disposition = field.content_disposition().unwrap();
        let content_type = field.content_type().to_owned();
        let filename = disposition.get_filename().unwrap();
        let mut filepath = UPLOAD_PATH.clone();
        filepath.push(filename);
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
            // TODO: lots of repeated code here due to moves into blocks
            // TODO: ultimately want to construct most of this one time,
            //       but they need to implement Send trait for safety
            let mut filepath = UPLOAD_PATH.clone();
            let filename = disposition.get_filename().unwrap();
            filepath.push(filename);
            let source = EntityDataSourceImpl::new(DB_PATH.as_path()).unwrap();
            let ctx: Arc<dyn EntityDataSource> = Arc::new(source);
            let records = RecordRepositoryImpl::new(ctx);
            let blobs = BlobRepositoryImpl::new(ASSETS_PATH.as_path());
            let usecase = ImportAsset::new(Box::new(records), Box::new(blobs));
            let params = Params::new(filepath, content_type);
            usecase.call(params)
        })
        .await?;
        asset_ids.push(asset.key);
    }
    let edit_url = format!("/assets/{}/edit", asset_ids[0]);
    Ok(HttpResponse::Found()
        .header(http::header::LOCATION, edit_url)
        .finish())
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
    let ctx: Arc<dyn EntityDataSource> = Arc::new(source);
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
        let filepath = blobs.blob_path(&asset)?;
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
    // TODO: this create_dir_all should probably be done by the repository
    std::fs::create_dir_all("./tmp/rocksdb").unwrap();
    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_owned());
    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_owned());
    let addr = format!("{}:{}", host, port);
    let schema = std::sync::Arc::new(graphql::create_schema());
    info!("listening on http://{}/...", addr);
    HttpServer::new(move || {
        App::new()
            .data(schema.clone())
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
            .route("/thumbnail/{w}/{h}/{id}", web::get().to(get_thumbnail))
            .route("/asset/{id}", web::get().to(raw_asset))
            .route("/asset/{id}", web::head().to(raw_asset))
            .route("/import", web::post().to(import_asset))
            .service(Files::new("/", STATIC_PATH.clone()).index_file("index.html"))
            .default_service(web::get().to(default_index))
    })
    .bind(addr)?
    .run()
    .await
}