//! Store backends exercised against real services spun up by testcontainers.
//!
//! Requires docker. Run with:
//!   cargo test --features store-s3,store-redis,store-pg,store-testkit \
//!     --test store_testcontainers
//!
//! Each backend runs the shared `run_contract` suite — the same assertions the
//! in-crate unit tests use — but against a throwaway container, so CI verifies
//! every backend end-to-end without any pre-provisioned services.

#![cfg(all(
    feature = "store-s3",
    feature = "store-redis",
    feature = "store-pg",
    feature = "store-testkit"
))]

use nucleation::store::contract::run_contract;
use nucleation::store::{PgConfig, PgStore, RedisConfig, RedisStore, S3Config, S3Store};
use testcontainers_modules::testcontainers::core::IntoContainerPort;
use testcontainers_modules::testcontainers::runners::SyncRunner;
use testcontainers_modules::testcontainers::{GenericImage, ImageExt};
use testcontainers_modules::{postgres, redis};

#[test]
fn s3_contract_against_rustfs_container() {
    // RustFS: an S3-compatible server that enforces `If-None-Match` like real
    // S3 (required by `put_if_absent`). MinIO no longer publishes community
    // images. Its default log level prints no readiness line, so wait for the
    // API itself to answer instead of matching log output.
    let node = GenericImage::new("rustfs/rustfs", "1.0.3")
        .with_exposed_port(9000.tcp())
        .with_env_var("RUSTFS_ACCESS_KEY", "rustfsadmin")
        .with_env_var("RUSTFS_SECRET_KEY", "rustfsadmin")
        .start()
        .expect("start RustFS");
    let port = node.get_host_port_ipv4(9000.tcp()).expect("rustfs port");
    let store = S3Store::connect(S3Config {
        bucket: "nucleation-test".to_string(),
        prefix: "tc/".to_string(),
        region: Some("us-east-1".to_string()),
        endpoint: Some(format!("http://127.0.0.1:{port}")),
        access_key: Some("rustfsadmin".to_string()),
        secret_key: Some("rustfsadmin".to_string()),
        force_path_style: true,
    })
    .expect("connect S3");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    while let Err(error) = store.ensure_bucket() {
        assert!(std::time::Instant::now() < deadline, "RustFS never became ready: {error}");
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
    run_contract(&store);
}

#[test]
fn redis_contract_against_container() {
    let node = redis::Redis::default().start().expect("start Redis");
    let port = node.get_host_port_ipv4(6379).expect("redis port");
    let store = RedisStore::connect(RedisConfig {
        url: format!("redis://127.0.0.1:{port}"),
        prefix: "tc:".to_string(),
    })
    .expect("connect Redis");
    run_contract(&store);
}

#[test]
fn pg_contract_against_container() {
    let node = postgres::Postgres::default()
        .start()
        .expect("start Postgres");
    let port = node.get_host_port_ipv4(5432).expect("pg port");
    let store = PgStore::connect(PgConfig {
        url: format!("postgres://postgres:postgres@127.0.0.1:{port}/postgres"),
        table: "nuc_store_tc".to_string(),
        prefix: String::new(),
    })
    .expect("connect Postgres");
    run_contract(&store);
}
