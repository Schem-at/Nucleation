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
use testcontainers_modules::testcontainers::runners::SyncRunner;
use testcontainers_modules::{minio, postgres, redis};

#[test]
fn s3_contract_against_minio_container() {
    let node = minio::MinIO::default().start().expect("start MinIO");
    let port = node.get_host_port_ipv4(9000).expect("minio port");
    let store = S3Store::connect(S3Config {
        bucket: "nucleation-test".to_string(),
        prefix: "tc/".to_string(),
        region: Some("us-east-1".to_string()),
        endpoint: Some(format!("http://127.0.0.1:{port}")),
        access_key: Some("minioadmin".to_string()),
        secret_key: Some("minioadmin".to_string()),
        force_path_style: true,
    })
    .expect("connect S3");
    store.ensure_bucket().expect("ensure bucket");
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
