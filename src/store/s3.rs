//! S3-compatible [`Store`] (AWS S3, MinIO, …).
//!
//! Wraps the async `aws-sdk-s3` client with an internal multi-thread tokio
//! runtime so the public surface stays synchronous, matching the rest of the
//! `store` module. Construct from an [`S3Config`] (explicit endpoint /
//! credentials, e.g. for MinIO) or via `s3://bucket/prefix` through
//! [`super::open`], which fills credentials from the environment.

use std::fmt;
use std::io::{Read, Write};
use std::sync::Arc;

use aws_sdk_s3::config::{BehaviorVersion, Credentials, Region};
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::types::{CompletedMultipartUpload, CompletedPart};
use aws_sdk_s3::Client;
use futures_util::TryStreamExt;
use tokio::runtime::Runtime;

use super::{Result, Store, StoreError};

/// S3 multipart minimum part size (except the final part).
const PART_SIZE: usize = 5 * 1024 * 1024;

/// Connection settings for an [`S3Store`].
#[derive(Clone)]
pub struct S3Config {
    /// Bucket name.
    pub bucket: String,
    /// Key prefix prepended to every logical key (normalised to end in `/`).
    pub prefix: String,
    /// Region (defaults to `us-east-1`, which MinIO accepts).
    pub region: Option<String>,
    /// Custom endpoint, e.g. `http://localhost:9000` for MinIO.
    pub endpoint: Option<String>,
    /// Access key id; falls back to `AWS_ACCESS_KEY_ID` when `None`.
    pub access_key: Option<String>,
    /// Secret access key; falls back to `AWS_SECRET_ACCESS_KEY` when `None`.
    pub secret_key: Option<String>,
    /// Use path-style addressing (required by MinIO).
    pub force_path_style: bool,
}

impl S3Config {
    /// Settings for `bucket` with defaults (no prefix, virtual-host style).
    pub fn new(bucket: impl Into<String>) -> Self {
        Self {
            bucket: bucket.into(),
            prefix: String::new(),
            region: None,
            endpoint: None,
            access_key: None,
            secret_key: None,
            force_path_style: false,
        }
    }
}

// Manual Debug so the secret key never appears in logs.
impl fmt::Debug for S3Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("S3Config")
            .field("bucket", &self.bucket)
            .field("prefix", &self.prefix)
            .field("region", &self.region)
            .field("endpoint", &self.endpoint)
            .field("access_key", &self.access_key.as_deref().map(|_| "***"))
            .field("secret_key", &self.secret_key.as_deref().map(|_| "***"))
            .field("force_path_style", &self.force_path_style)
            .finish()
    }
}

/// An S3-compatible object store. Cloning shares the runtime and client.
#[derive(Clone)]
pub struct S3Store {
    rt: Arc<Runtime>,
    client: Client,
    bucket: String,
    prefix: String,
}

impl S3Store {
    /// Connect using `cfg`. Building the client is cheap; no network call
    /// happens until the first operation (use [`Store::health`] to probe).
    pub fn connect(cfg: S3Config) -> Result<Self> {
        let rt = Runtime::new().map_err(|e| StoreError::Connection(e.to_string()))?;

        let region = cfg
            .region
            .clone()
            .unwrap_or_else(|| "us-east-1".to_string());
        let access = cfg
            .access_key
            .clone()
            .or_else(|| std::env::var("AWS_ACCESS_KEY_ID").ok());
        let secret = cfg
            .secret_key
            .clone()
            .or_else(|| std::env::var("AWS_SECRET_ACCESS_KEY").ok());

        let mut builder = aws_sdk_s3::config::Builder::new()
            .behavior_version(BehaviorVersion::latest())
            .region(Region::new(region))
            .force_path_style(cfg.force_path_style);

        if let Some(endpoint) = cfg.endpoint.clone() {
            builder = builder.endpoint_url(endpoint);
        }
        if let (Some(a), Some(s)) = (access, secret) {
            builder =
                builder.credentials_provider(Credentials::new(a, s, None, None, "nucleation"));
        }

        let client = Client::from_conf(builder.build());

        let prefix = normalize_prefix(&cfg.prefix);
        Ok(Self {
            rt: Arc::new(rt),
            client,
            bucket: cfg.bucket,
            prefix,
        })
    }

    /// Create the bucket if it does not already exist (helper for setup/tests).
    pub fn ensure_bucket(&self) -> Result<()> {
        self.rt.block_on(async {
            match self
                .client
                .create_bucket()
                .bucket(&self.bucket)
                .send()
                .await
            {
                Ok(_) => Ok(()),
                Err(e) => {
                    if let Some(se) = e.as_service_error() {
                        if se.is_bucket_already_owned_by_you() || se.is_bucket_already_exists() {
                            return Ok(());
                        }
                    }
                    Err(StoreError::Connection(e.to_string()))
                }
            }
        })
    }

    fn full(&self, key: &str) -> String {
        format!("{}{}", self.prefix, key)
    }
}

fn normalize_prefix(prefix: &str) -> String {
    if prefix.is_empty() || prefix.ends_with('/') {
        prefix.to_string()
    } else {
        format!("{prefix}/")
    }
}

impl Store for S3Store {
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let full = self.full(key);
        self.rt.block_on(async {
            match self
                .client
                .get_object()
                .bucket(&self.bucket)
                .key(&full)
                .send()
                .await
            {
                Ok(out) => {
                    let data = out
                        .body
                        .collect()
                        .await
                        .map_err(|e| StoreError::Io(e.to_string()))?;
                    Ok(Some(data.into_bytes().to_vec()))
                }
                Err(e) => {
                    if e.as_service_error().map(|se| se.is_no_such_key()) == Some(true) {
                        Ok(None)
                    } else {
                        Err(StoreError::Connection(e.to_string()))
                    }
                }
            }
        })
    }

    fn put(&self, key: &str, bytes: &[u8]) -> Result<()> {
        let full = self.full(key);
        let body = ByteStream::from(bytes.to_vec());
        self.rt.block_on(async {
            self.client
                .put_object()
                .bucket(&self.bucket)
                .key(&full)
                .body(body)
                .send()
                .await
                .map(|_| ())
                .map_err(|e| StoreError::Connection(e.to_string()))
        })
    }

    fn exists(&self, key: &str) -> Result<bool> {
        let full = self.full(key);
        self.rt.block_on(async {
            match self
                .client
                .head_object()
                .bucket(&self.bucket)
                .key(&full)
                .send()
                .await
            {
                Ok(_) => Ok(true),
                Err(e) => {
                    if e.as_service_error().map(|se| se.is_not_found()) == Some(true) {
                        Ok(false)
                    } else {
                        Err(StoreError::Connection(e.to_string()))
                    }
                }
            }
        })
    }

    fn delete(&self, key: &str) -> Result<()> {
        let full = self.full(key);
        self.rt.block_on(async {
            self.client
                .delete_object()
                .bucket(&self.bucket)
                .key(&full)
                .send()
                .await
                .map(|_| ())
                .map_err(|e| StoreError::Connection(e.to_string()))
        })
    }

    fn list(&self, prefix: &str) -> Result<Vec<String>> {
        let s3_prefix = self.full(prefix);
        self.rt.block_on(async {
            let mut keys = Vec::new();
            let mut token: Option<String> = None;
            loop {
                let mut req = self
                    .client
                    .list_objects_v2()
                    .bucket(&self.bucket)
                    .prefix(&s3_prefix);
                if let Some(t) = &token {
                    req = req.continuation_token(t);
                }
                let resp = req
                    .send()
                    .await
                    .map_err(|e| StoreError::Connection(e.to_string()))?;
                for obj in resp.contents() {
                    if let Some(k) = obj.key() {
                        if let Some(logical) = k.strip_prefix(&self.prefix) {
                            keys.push(logical.to_string());
                        }
                    }
                }
                if resp.is_truncated() == Some(true) {
                    token = resp.next_continuation_token().map(|s| s.to_string());
                    if token.is_none() {
                        break;
                    }
                } else {
                    break;
                }
            }
            Ok(keys)
        })
    }

    fn health(&self) -> Result<()> {
        self.rt.block_on(async {
            self.client
                .head_bucket()
                .bucket(&self.bucket)
                .send()
                .await
                .map(|_| ())
                .map_err(|e| StoreError::Connection(e.to_string()))
        })
    }

    /// Native streaming read: pulls the object body chunk-by-chunk instead of
    /// buffering the whole object in memory.
    fn reader(&self, key: &str) -> Result<Box<dyn Read + '_>> {
        let full = self.full(key);
        let body = self.rt.block_on(async {
            match self
                .client
                .get_object()
                .bucket(&self.bucket)
                .key(&full)
                .send()
                .await
            {
                Ok(out) => Ok(out.body),
                Err(e) => {
                    if e.as_service_error().map(|se| se.is_no_such_key()) == Some(true) {
                        Err(StoreError::NotFound(full.clone()))
                    } else {
                        Err(StoreError::Connection(e.to_string()))
                    }
                }
            }
        })?;
        Ok(Box::new(S3Reader {
            rt: self.rt.clone(),
            body,
            leftover: Vec::new(),
            pos: 0,
        }))
    }

    /// Native streaming write: uploads in 5 MiB multipart chunks as data
    /// arrives, so the whole object is never buffered. Small writes fall back
    /// to a single `put_object` on commit.
    fn writer(&self, key: &str) -> Result<Box<dyn Write + '_>> {
        Ok(Box::new(S3Writer {
            rt: self.rt.clone(),
            client: self.client.clone(),
            bucket: self.bucket.clone(),
            key: self.full(key),
            buf: Vec::new(),
            upload_id: None,
            parts: Vec::new(),
            next_part: 1,
            committed: false,
        }))
    }
}

/// Sync [`Read`] adapter over the async S3 `ByteStream`.
struct S3Reader {
    rt: Arc<Runtime>,
    body: ByteStream,
    leftover: Vec<u8>,
    pos: usize,
}

impl Read for S3Reader {
    fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
        if self.pos >= self.leftover.len() {
            match self.rt.block_on(self.body.try_next()) {
                Ok(Some(chunk)) => {
                    self.leftover = chunk.to_vec();
                    self.pos = 0;
                }
                Ok(None) => return Ok(0),
                Err(e) => return Err(std::io::Error::other(e.to_string())),
            }
        }
        if self.leftover.is_empty() {
            return Ok(0);
        }
        let n = out.len().min(self.leftover.len() - self.pos);
        out[..n].copy_from_slice(&self.leftover[self.pos..self.pos + n]);
        self.pos += n;
        Ok(n)
    }
}

/// Streaming multipart [`Write`] adapter for S3.
struct S3Writer {
    rt: Arc<Runtime>,
    client: Client,
    bucket: String,
    key: String,
    buf: Vec<u8>,
    upload_id: Option<String>,
    parts: Vec<CompletedPart>,
    next_part: i32,
    committed: bool,
}

impl S3Writer {
    fn ensure_multipart(&mut self) -> std::io::Result<String> {
        if let Some(id) = &self.upload_id {
            return Ok(id.clone());
        }
        let id = self
            .rt
            .block_on(
                self.client
                    .create_multipart_upload()
                    .bucket(&self.bucket)
                    .key(&self.key)
                    .send(),
            )
            .map_err(|e| std::io::Error::other(e.to_string()))?
            .upload_id
            .ok_or_else(|| std::io::Error::other("S3 returned no upload id"))?;
        self.upload_id = Some(id.clone());
        Ok(id)
    }

    fn upload_one(&mut self, bytes: Vec<u8>) -> std::io::Result<()> {
        let upload_id = self.ensure_multipart()?;
        let part_number = self.next_part;
        self.next_part += 1;
        let etag = self
            .rt
            .block_on(
                self.client
                    .upload_part()
                    .bucket(&self.bucket)
                    .key(&self.key)
                    .upload_id(&upload_id)
                    .part_number(part_number)
                    .body(ByteStream::from(bytes))
                    .send(),
            )
            .map_err(|e| std::io::Error::other(e.to_string()))?
            .e_tag;
        self.parts.push(
            CompletedPart::builder()
                .part_number(part_number)
                .set_e_tag(etag)
                .build(),
        );
        Ok(())
    }

    fn commit(&mut self) -> std::io::Result<()> {
        if self.committed {
            return Ok(());
        }
        if self.upload_id.is_none() {
            // Small object — single PUT.
            let body = ByteStream::from(std::mem::take(&mut self.buf));
            self.rt
                .block_on(
                    self.client
                        .put_object()
                        .bucket(&self.bucket)
                        .key(&self.key)
                        .body(body)
                        .send(),
                )
                .map_err(|e| std::io::Error::other(e.to_string()))?;
        } else {
            if !self.buf.is_empty() {
                let last = std::mem::take(&mut self.buf);
                self.upload_one(last)?;
            }
            let completed = CompletedMultipartUpload::builder()
                .set_parts(Some(self.parts.clone()))
                .build();
            let upload_id = self.upload_id.clone().unwrap();
            self.rt
                .block_on(
                    self.client
                        .complete_multipart_upload()
                        .bucket(&self.bucket)
                        .key(&self.key)
                        .upload_id(upload_id)
                        .multipart_upload(completed)
                        .send(),
                )
                .map_err(|e| std::io::Error::other(e.to_string()))?;
        }
        self.committed = true;
        Ok(())
    }
}

impl Write for S3Writer {
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
        self.buf.extend_from_slice(data);
        while self.buf.len() >= PART_SIZE {
            let part: Vec<u8> = self.buf.drain(..PART_SIZE).collect();
            self.upload_one(part)?;
        }
        Ok(data.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.commit()
    }
}

impl Drop for S3Writer {
    fn drop(&mut self) {
        if !self.committed {
            let _ = self.commit();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};

    /// Build a test store against a local MinIO, or `None` if unreachable.
    fn test_store() -> Option<S3Store> {
        let endpoint = std::env::var("NUC_TEST_S3_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:9000".to_string());
        let cfg = S3Config {
            bucket: "nucleation-test".to_string(),
            prefix: format!("t{}/", std::process::id()),
            region: Some("us-east-1".to_string()),
            endpoint: Some(endpoint),
            access_key: Some("minioadmin".to_string()),
            secret_key: Some("minioadmin".to_string()),
            force_path_style: true,
        };
        let store = S3Store::connect(cfg).ok()?;
        store.ensure_bucket().ok()?;
        store.health().ok()?;
        Some(store)
    }

    #[test]
    fn s3_store_satisfies_contract() {
        let Some(store) = test_store() else {
            eprintln!(
                "skipping S3 contract: no MinIO at NUC_TEST_S3_ENDPOINT (default http://localhost:9000)"
            );
            return;
        };
        crate::store::contract::run_contract(&store);
    }

    #[test]
    fn s3_streaming_multipart_roundtrip() {
        let Some(store) = test_store() else {
            eprintln!("skipping S3 streaming: no MinIO");
            return;
        };
        // 11 MiB so the multipart writer flushes 2 full parts + a remainder.
        let data: Vec<u8> = (0..11 * 1024 * 1024).map(|i| (i % 256) as u8).collect();
        let key = "stream/big.bin";
        {
            let mut w = store.writer(key).expect("writer");
            w.write_all(&data).expect("write_all");
            w.flush().expect("flush");
        }
        let mut out = Vec::new();
        store
            .reader(key)
            .expect("reader")
            .read_to_end(&mut out)
            .expect("read_to_end");
        assert_eq!(out.len(), data.len(), "streamed length must match");
        assert_eq!(out, data, "streamed bytes must match");
        store.delete(key).expect("cleanup");
    }

    #[test]
    fn debug_redacts_secret() {
        let cfg = S3Config {
            secret_key: Some("supersecret".to_string()),
            ..S3Config::new("b")
        };
        let dbg = format!("{cfg:?}");
        assert!(!dbg.contains("supersecret"), "secret must be redacted");
    }
}
