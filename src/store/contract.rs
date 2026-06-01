//! Backend-agnostic behavioural contract for [`Store`](super::Store).
//!
//! Every backend runs the *same* assertions through [`run_contract`], so a new
//! backend can't silently drift from the documented semantics. Service-backed
//! backends (S3/Redis/Postgres) call this through their own (possibly skipped)
//! tests.

use std::io::{Read, Write};

use super::{Store, StoreError};

/// Exercise the full `Store` contract against a fresh, empty `store`.
///
/// Panics (failing the calling test) on the first violated expectation.
pub fn run_contract(store: &dyn Store) {
    health_reports_usable(store);
    put_get_roundtrip(store);
    put_overwrites(store);
    get_missing_is_none(store);
    exists_tracks_presence(store);
    delete_is_idempotent(store);
    list_returns_keys_under_prefix(store);
    reader_streams_bytes(store);
    reader_missing_is_not_found(store);
    writer_commits_on_flush(store);
    put_if_absent_is_atomic(store);
    list_paginated_walks_keyset(store);
}

fn health_reports_usable(store: &dyn Store) {
    store
        .health()
        .expect("health() should succeed on a usable store");
}

fn put_get_roundtrip(store: &dyn Store) {
    let key = "contract/roundtrip";
    store.put(key, b"hello").expect("put");
    assert_eq!(
        store.get(key).expect("get"),
        Some(b"hello".to_vec()),
        "get must return exactly what put stored"
    );
    store.delete(key).expect("cleanup");
}

fn put_overwrites(store: &dyn Store) {
    let key = "contract/overwrite";
    store.put(key, b"first").expect("put first");
    store.put(key, b"second").expect("put second");
    assert_eq!(
        store.get(key).expect("get"),
        Some(b"second".to_vec()),
        "put must overwrite an existing key"
    );
    store.delete(key).expect("cleanup");
}

fn get_missing_is_none(store: &dyn Store) {
    assert_eq!(
        store.get("contract/definitely-absent").expect("get"),
        None,
        "get on a missing key must be Ok(None)"
    );
}

fn exists_tracks_presence(store: &dyn Store) {
    let key = "contract/exists";
    assert!(!store.exists(key).expect("exists absent"));
    store.put(key, b"x").expect("put");
    assert!(store.exists(key).expect("exists present"));
    store.delete(key).expect("delete");
    assert!(!store.exists(key).expect("exists after delete"));
}

fn delete_is_idempotent(store: &dyn Store) {
    let key = "contract/idempotent-delete";
    store.put(key, b"x").expect("put");
    store.delete(key).expect("first delete");
    store
        .delete(key)
        .expect("deleting a missing key must succeed");
    assert_eq!(store.get(key).expect("get"), None);
}

fn list_returns_keys_under_prefix(store: &dyn Store) {
    store.put("list/p/1", b"a").expect("put");
    store.put("list/p/2", b"b").expect("put");
    store.put("list/q/1", b"c").expect("put");

    let mut under_p = store.list("list/p/").expect("list");
    under_p.sort();
    assert_eq!(
        under_p,
        vec!["list/p/1".to_string(), "list/p/2".to_string()]
    );

    let all = store.list("list/").expect("list all");
    assert!(all.contains(&"list/q/1".to_string()));
    assert_eq!(all.len(), 3, "prefix list must include every match");

    for k in ["list/p/1", "list/p/2", "list/q/1"] {
        store.delete(k).expect("cleanup");
    }
}

fn reader_streams_bytes(store: &dyn Store) {
    let key = "contract/reader";
    let data: Vec<u8> = (0..4096).map(|i| (i % 251) as u8).collect();
    store.put(key, &data).expect("put");

    let mut out = Vec::new();
    store
        .reader(key)
        .expect("reader")
        .read_to_end(&mut out)
        .expect("read_to_end");
    assert_eq!(out, data, "reader must yield the stored bytes");
    store.delete(key).expect("cleanup");
}

fn reader_missing_is_not_found(store: &dyn Store) {
    match store.reader("contract/absent-reader") {
        Err(StoreError::NotFound(_)) => {}
        Err(other) => panic!("reader on missing key must be NotFound, got {other:?}"),
        Ok(_) => panic!("reader on missing key must error, got Ok"),
    }
}

fn writer_commits_on_flush(store: &dyn Store) {
    let key = "contract/writer";
    let data: Vec<u8> = (0..2048).map(|i| (i % 97) as u8).collect();
    {
        let mut w = store.writer(key).expect("writer");
        w.write_all(&data).expect("write_all");
        w.flush().expect("flush");
    }
    assert_eq!(
        store.get(key).expect("get"),
        Some(data),
        "bytes written through writer must be readable after flush"
    );
    store.delete(key).expect("cleanup");
}

fn put_if_absent_is_atomic(store: &dyn Store) {
    let key = "contract/cas";
    assert!(
        store.put_if_absent(key, b"first").expect("cas first"),
        "first put_if_absent must win"
    );
    assert!(
        !store.put_if_absent(key, b"second").expect("cas second"),
        "put_if_absent on an existing key must return false"
    );
    assert_eq!(
        store.get(key).expect("get"),
        Some(b"first".to_vec()),
        "put_if_absent must not overwrite the existing value"
    );
    store.delete(key).expect("cleanup");
}

fn list_paginated_walks_keyset(store: &dyn Store) {
    let keys: Vec<String> = (0..5).map(|i| format!("page/{i}")).collect();
    for k in &keys {
        store.put(k, b"x").expect("put");
    }

    // First page of 2: sorted, with a cursor signalling more.
    let (p1, n1) = store.list_paginated("page/", None, 2).expect("page1");
    assert_eq!(p1.len(), 2, "page honours the limit");
    assert!(
        p1.windows(2).all(|w| w[0] < w[1]),
        "page is sorted ascending"
    );
    assert!(n1.is_some(), "more keys remain after the first page");

    // The next page starts strictly after the cursor (no overlap).
    let (p2, _) = store
        .list_paginated("page/", n1.as_deref(), 2)
        .expect("page2");
    assert!(
        p2.iter().all(|k| k > p1.last().unwrap()),
        "page 2 is strictly after page 1"
    );

    // Walking to exhaustion visits every key exactly once.
    let mut seen = Vec::new();
    let mut cursor: Option<String> = None;
    loop {
        let (page, next) = store
            .list_paginated("page/", cursor.as_deref(), 2)
            .expect("walk");
        seen.extend(page);
        match next {
            Some(c) => cursor = Some(c),
            None => break,
        }
    }
    seen.sort();
    assert_eq!(seen, keys, "pagination must cover every key exactly once");

    for k in &keys {
        store.delete(k).expect("cleanup");
    }
}
