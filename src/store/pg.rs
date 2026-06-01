//! Postgres-backed [`Store`]. Objects live as rows in a blob table
//! (`key TEXT PRIMARY KEY, data BYTEA`). Wraps async `tokio-postgres` with an
//! internal tokio runtime so the public surface stays synchronous.

use std::sync::Arc;

use tokio::runtime::Runtime;
use tokio_postgres::{Client, NoTls};

use super::{Result, Store, StoreError};

/// Connection settings for a [`PgStore`].
#[derive(Clone, Debug)]
pub struct PgConfig {
    /// libpq URL, e.g. `postgres://user:pass@localhost:5432/db`.
    pub url: String,
    /// Blob table name (validated to `[A-Za-z0-9_]+`).
    pub table: String,
    /// Key prefix prepended to every logical key.
    pub prefix: String,
}

impl PgConfig {
    /// Settings for `url` storing blobs in `table`, no prefix.
    pub fn new(url: impl Into<String>, table: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            table: table.into(),
            prefix: String::new(),
        }
    }
}

/// A Postgres blob store. Cloning shares the runtime and client.
#[derive(Clone)]
pub struct PgStore {
    rt: Arc<Runtime>,
    client: Arc<Client>,
    table: String,
    prefix: String,
}

impl PgStore {
    /// Connect and ensure the blob table exists.
    pub fn connect(cfg: PgConfig) -> Result<Self> {
        if cfg.table.is_empty()
            || !cfg
                .table
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_')
        {
            return Err(StoreError::InvalidKey(format!(
                "invalid table name: {:?}",
                cfg.table
            )));
        }

        let rt = Runtime::new().map_err(|e| StoreError::Connection(e.to_string()))?;
        let (client, connection) = rt
            .block_on(tokio_postgres::connect(&cfg.url, NoTls))
            .map_err(|e| StoreError::Connection(e.to_string()))?;
        // Drive the connection on the runtime for the life of the store.
        rt.spawn(async move {
            let _ = connection.await;
        });

        let store = Self {
            rt: Arc::new(rt),
            client: Arc::new(client),
            table: cfg.table,
            prefix: cfg.prefix,
        };
        store.ensure_table()?;
        Ok(store)
    }

    fn ensure_table(&self) -> Result<()> {
        let sql = format!(
            "CREATE TABLE IF NOT EXISTS {} (key TEXT PRIMARY KEY, data BYTEA NOT NULL)",
            self.table
        );
        self.rt
            .block_on(self.client.execute(&sql, &[]))
            .map(|_| ())
            .map_err(|e| StoreError::Connection(e.to_string()))
    }

    fn full(&self, key: &str) -> String {
        format!("{}{}", self.prefix, key)
    }
}

impl Store for PgStore {
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let full = self.full(key);
        let sql = format!("SELECT data FROM {} WHERE key = $1", self.table);
        self.rt.block_on(async {
            let row = self
                .client
                .query_opt(&sql, &[&full])
                .await
                .map_err(|e| StoreError::Connection(e.to_string()))?;
            Ok(row.map(|r| r.get::<_, Vec<u8>>(0)))
        })
    }

    fn put(&self, key: &str, bytes: &[u8]) -> Result<()> {
        let full = self.full(key);
        let data = bytes.to_vec();
        let sql = format!(
            "INSERT INTO {} (key, data) VALUES ($1, $2) \
             ON CONFLICT (key) DO UPDATE SET data = EXCLUDED.data",
            self.table
        );
        self.rt.block_on(async {
            self.client
                .execute(&sql, &[&full, &data])
                .await
                .map(|_| ())
                .map_err(|e| StoreError::Connection(e.to_string()))
        })
    }

    fn exists(&self, key: &str) -> Result<bool> {
        let full = self.full(key);
        let sql = format!("SELECT 1 FROM {} WHERE key = $1", self.table);
        self.rt.block_on(async {
            let row = self
                .client
                .query_opt(&sql, &[&full])
                .await
                .map_err(|e| StoreError::Connection(e.to_string()))?;
            Ok(row.is_some())
        })
    }

    fn delete(&self, key: &str) -> Result<()> {
        let full = self.full(key);
        let sql = format!("DELETE FROM {} WHERE key = $1", self.table);
        self.rt.block_on(async {
            self.client
                .execute(&sql, &[&full])
                .await
                .map(|_| ())
                .map_err(|e| StoreError::Connection(e.to_string()))
        })
    }

    fn list(&self, prefix: &str) -> Result<Vec<String>> {
        let full_prefix = self.full(prefix);
        let strip_len = self.prefix.len();
        let sql = format!("SELECT key FROM {} WHERE starts_with(key, $1)", self.table);
        self.rt.block_on(async {
            let rows = self
                .client
                .query(&sql, &[&full_prefix])
                .await
                .map_err(|e| StoreError::Connection(e.to_string()))?;
            Ok(rows
                .iter()
                .map(|r| r.get::<_, String>(0)[strip_len..].to_string())
                .collect())
        })
    }

    fn health(&self) -> Result<()> {
        self.rt
            .block_on(self.client.execute("SELECT 1", &[]))
            .map(|_| ())
            .map_err(|e| StoreError::Connection(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_store() -> Option<PgStore> {
        let url = std::env::var("NUC_TEST_PG_URL").unwrap_or_else(|_| {
            "postgres://postgres:postgres@localhost:5432/nucleation".to_string()
        });
        // Unique table per process so concurrent runs don't collide.
        let cfg = PgConfig {
            url,
            table: format!("nuc_store_test_{}", std::process::id()),
            prefix: String::new(),
        };
        let store = PgStore::connect(cfg).ok()?;
        store.health().ok()?;
        Some(store)
    }

    #[test]
    fn pg_store_satisfies_contract() {
        let Some(store) = test_store() else {
            eprintln!("skipping Postgres contract: no server at NUC_TEST_PG_URL");
            return;
        };
        crate::store::contract::run_contract(&store);
        // Drop the per-process test table.
        let sql = format!("DROP TABLE IF EXISTS {}", store.table);
        let _ = store.rt.block_on(store.client.execute(&sql, &[]));
    }
}
