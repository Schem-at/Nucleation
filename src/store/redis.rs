//! Redis-backed [`Store`]. Objects are Redis string values; keys are namespaced
//! by a prefix. Wraps the async `redis` client with an internal tokio runtime so
//! the public surface stays synchronous.

use std::sync::Arc;

use redis::AsyncCommands;
use tokio::runtime::Runtime;

use super::{Result, Store, StoreError};

/// Connection settings for a [`RedisStore`].
#[derive(Clone, Debug)]
pub struct RedisConfig {
    /// Connection URL, e.g. `redis://localhost:6379/0`.
    pub url: String,
    /// Key prefix prepended to every logical key (normalised to end in `:`).
    pub prefix: String,
}

impl RedisConfig {
    /// Settings for `url` with no prefix.
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            prefix: String::new(),
        }
    }
}

/// A Redis object store. Cloning shares the runtime and multiplexed connection.
#[derive(Clone)]
pub struct RedisStore {
    rt: Arc<Runtime>,
    conn: redis::aio::MultiplexedConnection,
    prefix: String,
}

impl RedisStore {
    /// Connect to Redis. Opens a multiplexed async connection eagerly.
    pub fn connect(cfg: RedisConfig) -> Result<Self> {
        let rt = Runtime::new().map_err(|e| StoreError::Connection(e.to_string()))?;
        let client =
            redis::Client::open(cfg.url).map_err(|e| StoreError::Connection(e.to_string()))?;
        let conn = rt
            .block_on(client.get_multiplexed_async_connection())
            .map_err(|e| StoreError::Connection(e.to_string()))?;
        Ok(Self {
            rt: Arc::new(rt),
            conn,
            prefix: cfg.prefix,
        })
    }

    fn full(&self, key: &str) -> String {
        format!("{}{}", self.prefix, key)
    }
}

impl Store for RedisStore {
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let full = self.full(key);
        let mut conn = self.conn.clone();
        self.rt.block_on(async move {
            let v: Option<Vec<u8>> = conn
                .get(&full)
                .await
                .map_err(|e| StoreError::Connection(e.to_string()))?;
            Ok(v)
        })
    }

    fn put(&self, key: &str, bytes: &[u8]) -> Result<()> {
        let full = self.full(key);
        let mut conn = self.conn.clone();
        self.rt.block_on(async move {
            let _: () = conn
                .set(&full, bytes)
                .await
                .map_err(|e| StoreError::Connection(e.to_string()))?;
            Ok(())
        })
    }

    fn exists(&self, key: &str) -> Result<bool> {
        let full = self.full(key);
        let mut conn = self.conn.clone();
        self.rt.block_on(async move {
            let e: bool = conn
                .exists(&full)
                .await
                .map_err(|e| StoreError::Connection(e.to_string()))?;
            Ok(e)
        })
    }

    fn delete(&self, key: &str) -> Result<()> {
        let full = self.full(key);
        let mut conn = self.conn.clone();
        self.rt.block_on(async move {
            let _: i64 = conn
                .del(&full)
                .await
                .map_err(|e| StoreError::Connection(e.to_string()))?;
            Ok(())
        })
    }

    fn list(&self, prefix: &str) -> Result<Vec<String>> {
        let pattern = format!("{}{}*", self.prefix, prefix);
        let strip_len = self.prefix.len();
        let mut conn = self.conn.clone();
        self.rt.block_on(async move {
            let mut keys = Vec::new();
            let mut cursor: u64 = 0;
            loop {
                let (next, batch): (u64, Vec<String>) = redis::cmd("SCAN")
                    .arg(cursor)
                    .arg("MATCH")
                    .arg(&pattern)
                    .arg("COUNT")
                    .arg(1000)
                    .query_async(&mut conn)
                    .await
                    .map_err(|e| StoreError::Connection(e.to_string()))?;
                for k in batch {
                    keys.push(k[strip_len..].to_string());
                }
                if next == 0 {
                    break;
                }
                cursor = next;
            }
            Ok(keys)
        })
    }

    fn health(&self) -> Result<()> {
        let mut conn = self.conn.clone();
        self.rt.block_on(async move {
            let pong: String = redis::cmd("PING")
                .query_async(&mut conn)
                .await
                .map_err(|e| StoreError::Connection(e.to_string()))?;
            if pong == "PONG" {
                Ok(())
            } else {
                Err(StoreError::Connection(format!(
                    "unexpected PING reply: {pong}"
                )))
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_store() -> Option<RedisStore> {
        let url = std::env::var("NUC_TEST_REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string());
        let cfg = RedisConfig {
            url,
            prefix: format!("nuctest:{}:", std::process::id()),
        };
        let store = RedisStore::connect(cfg).ok()?;
        store.health().ok()?;
        Some(store)
    }

    #[test]
    fn redis_store_satisfies_contract() {
        let Some(store) = test_store() else {
            eprintln!("skipping Redis contract: no server at NUC_TEST_REDIS_URL (default redis://localhost:6379)");
            return;
        };
        crate::store::contract::run_contract(&store);
    }
}
