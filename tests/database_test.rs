//
// Copyright (c) 2019 Nathan Fiedler
//
mod util;

use failure::Error;
use std::fs;
use std::ops::Deref;
use std::thread;
use util::DBPath;
use tanuki::database::*;

// #[test]
// fn test_get_path() {
//     let db_path = DBPath::new("_test_get_path");
//     let dbase = Database::new(&db_path).unwrap();
//     assert_eq!(db_path.as_ref(), dbase.get_path());
// }

#[test]
fn test_backup_restore() {
    let db_path = DBPath::new("_test_backup_restore");
    let dbase = Database::new(&db_path).unwrap();
    assert!(dbase.insert_document(b"charlie", b"localhost").is_ok());

    // backup the database
    let backup_path = DBPath::new("_test_backup_restore_bup");
    dbase.create_backup(&backup_path).unwrap();

    // restore from backup (to a new path)
    let new_path = DBPath::new("_test_backup_restore_new");
    Database::restore_from_backup(&backup_path, &new_path).unwrap();

    // open that new database and verify contents
    let new_base = Database::new(&new_path).unwrap();
    match new_base.get_document(b"charlie") {
        Ok(Some(value)) => assert_eq!(value.deref(), b"localhost"),
        Ok(None) => panic!("get document returned None!"),
        Err(e) => panic!("get document error: {}", e),
    };
    let _ = fs::remove_dir_all(backup_path);
}

#[test]
fn test_insert_document() {
    let db_path = DBPath::new("_test_insert_document");
    let dbase = Database::new(&db_path).unwrap();
    assert!(dbase.insert_document(b"charlie", b"localhost").is_ok());
    assert!(dbase.insert_document(b"charlie", b"remotehost").is_ok());
    match dbase.get_document(b"charlie") {
        Ok(Some(value)) => assert_eq!(value.deref(), b"localhost"),
        Ok(None) => panic!("get document returned None!"),
        Err(e) => panic!("get document error: {}", e),
    }
    // we can update a value using put_document()
    assert!(dbase.put_document(b"charlie", b"remotehost").is_ok());
    match dbase.get_document(b"charlie") {
        Ok(Some(value)) => assert_eq!(value.deref(), b"remotehost"),
        Ok(None) => panic!("get document returned None!"),
        Err(e) => panic!("get document error: {}", e),
    }
}

#[test]
fn test_prefix_counting() {
    let db_path = DBPath::new("_test_prefix_counting");
    let dbase = Database::new(&db_path).unwrap();
    assert!(dbase
        .insert_document(b"punk/cafebabe", b"madoka magic")
        .is_ok());
    assert!(dbase
        .insert_document(b"punk/deadbeef", b"made in abyss")
        .is_ok());
    assert!(dbase
        .insert_document(b"punk/cafed00d", b"houseki no kuni")
        .is_ok());
    assert!(dbase
        .insert_document(b"punk/1badb002", b"eureka seven")
        .is_ok());
    assert!(dbase
        .insert_document(b"punk/abadbabe", b"last exile")
        .is_ok());
    assert!(dbase
        .insert_document(b"kree/cafebabe", b"hibeke! euphonium")
        .is_ok());
    assert!(dbase
        .insert_document(b"kree/deadbeef", b"flip flappers")
        .is_ok());
    assert!(dbase
        .insert_document(b"kree/abadbabe", b"koe no katachi")
        .is_ok());
    assert!(dbase
        .insert_document(b"kree/cafefeed", b"toradora!")
        .is_ok());
    let result = dbase.count_prefix("punk");
    assert!(result.is_ok());
    let count: usize = result.unwrap();
    assert_eq!(count, 5);
    let result = dbase.count_prefix("kree");
    assert!(result.is_ok());
    let count: usize = result.unwrap();
    assert_eq!(count, 4);
}

#[test]
fn test_db_threads_uniq_paths() -> Result<(), Error> {
    let mut children = vec![];
    for ii in 0..10 {
        children.push(thread::spawn(move || {
            // create a clean database for each thread (DBPath creates uniquely
            // named paths each time)
            let db_path = DBPath::new("_test_db_threads_uniq_paths");
            let dbase = Database::new(&db_path).unwrap();
            let key = format!("thread_test_key_{}", ii);
            let result = dbase.insert_document(key.as_bytes(), b"foo bar baz quux");
            assert!(result.is_ok());
        }));
    }
    for child in children {
        let _ = child.join();
    }
    Ok(())
}

// This test is not reliable on Linux systems for some reason.
// #[test]
// fn test_db_threads_one_path() -> Result<(), Error> {
//     let db_path = DBPath::new("_test_db_threads_one_path");
//     let mut children = vec![];
//     for ii in 0..50 {
//         let clone_path = db_path.clone();
//         children.push(thread::spawn(move || {
//             // create a separate instance for each thread
//             let dbase = Database::new(&clone_path).unwrap();
//             let key = format!("thread_test_key_{}", ii);
//             let result = dbase.insert_document(key.as_bytes(), b"foo bar baz quux");
//             assert!(result.is_ok());
//         }));
//     }
//     for child in children {
//         let _ = child.join();
//     }
//     let dbase = Database::new(&db_path).unwrap();
//     let result = dbase.count_prefix("thread_test_key_");
//     assert!(result.is_ok());
//     let count: usize = result.unwrap();
//     assert_eq!(count, 50);
//     Ok(())
// }
