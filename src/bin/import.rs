//
// Copyright (c) 2020 Nathan Fiedler
//
use chrono::prelude::*;
use failure::Error;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::Arc;
use tanuki::data::repositories::{BlobRepositoryImpl, RecordRepositoryImpl};
use tanuki::data::sources::{EntityDataSource, EntityDataSourceImpl};
use tanuki::domain::entities::{Asset, Dimensions};
use tanuki::domain::repositories::{BlobRepository, RecordRepository};

#[derive(Serialize, Deserialize)]
struct ImportAsset {
    key: String,
    doc: ImportDoc,
}

#[derive(Serialize, Deserialize)]
struct ImportDoc {
    checksum: String,
    filename: String,
    filesize: u64,
    mimetype: String,
    tags: Vec<String>,
    import_date: u64,
    caption: Option<String>,
    location: Option<String>,
    user_date: Option<u64>,
    original_date: Option<u64>,
}

impl Into<tanuki::domain::entities::Asset> for ImportAsset {
    fn into(self) -> tanuki::domain::entities::Asset {
        tanuki::domain::entities::Asset {
            key: self.key,
            checksum: self.doc.checksum,
            filename: self.doc.filename,
            byte_length: self.doc.filesize,
            media_type: self.doc.mimetype,
            tags: self.doc.tags,
            import_date: Utc.timestamp((self.doc.import_date / 1000) as i64, 0),
            caption: self.doc.caption,
            location: self
                .doc
                .location
                .and_then(|v| if v.is_empty() { None } else { Some(v) }),
            user_date: self
                .doc
                .user_date
                .map(|d| Utc.timestamp((d / 1000) as i64, 0)),
            original_date: self
                .doc
                .original_date
                .map(|d| Utc.timestamp((d / 1000) as i64, 0)),
            dimensions: None,
        }
    }
}

fn set_dimensions(asset: &mut Asset, blobs: &dyn BlobRepository) {
    if let Ok(filepath) = blobs.blob_path(&asset.key) {
        if asset.media_type.starts_with("image/") {
            if let Ok(dim) = image::image_dimensions(filepath) {
                asset.dimensions = Some(Dimensions(dim.0, dim.1));
            }
        }
    }
}

fn main() -> Result<(), Error> {
    let path = "dump.json";
    let infile = File::open(path)?;
    let reader = BufReader::new(infile);
    let assets: Vec<ImportAsset> = serde_json::from_reader(reader)?;
    println!("# assets loaded: {}", assets.len());
    let blobs_path = Path::new("tmp/assets");
    let blobs = BlobRepositoryImpl::new(blobs_path);
    let converts: Vec<Asset> = assets
        .into_iter()
        .map(|a| {
            let mut convert: Asset = ImportAsset::into(a);
            set_dimensions(&mut convert, &blobs);
            convert
        })
        .collect();
    let source = EntityDataSourceImpl::new("tmp/db2").unwrap();
    let ctx: Arc<dyn EntityDataSource> = Arc::new(source);
    let records = RecordRepositoryImpl::new(ctx);
    for convert in converts.iter() {
        records.put_asset(convert)?;
    }
    println!("# assets converted: {}", converts.len());
    Ok(())
}
