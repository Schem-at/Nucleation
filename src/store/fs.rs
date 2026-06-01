//! Filesystem-backed [`Store`], rooted at a directory. Keys map to paths under
//! the root; path traversal (`..`) is rejected. Native targets only.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use super::{Result, Store, StoreError};

/// Counter for unique temp filenames during atomic writes.
static TMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Prefix marking in-progress temp files; excluded from `list`.
const TMP_PREFIX: &str = ".nuctmp.";

/// A store whose objects are files under `root`. Cloning shares the same root.
#[derive(Clone)]
pub struct FsStore {
    root: PathBuf,
}

impl FsStore {
    /// Create a store rooted at `root`. The directory is created on first write.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Resolve a key to a path under the root, rejecting traversal.
    fn path_for(&self, key: &str) -> Result<PathBuf> {
        if key.is_empty() {
            return Err(StoreError::InvalidKey("empty key".into()));
        }
        let mut path = self.root.clone();
        for comp in key.split('/') {
            if comp.is_empty()
                || comp == "."
                || comp == ".."
                || comp.contains('\\')
                || comp.contains('\0')
            {
                return Err(StoreError::InvalidKey(key.to_string()));
            }
            path.push(comp);
        }
        Ok(path)
    }
}

impl Store for FsStore {
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let path = self.path_for(key)?;
        match std::fs::read(&path) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn put(&self, key: &str, bytes: &[u8]) -> Result<()> {
        let path = self.path_for(key)?;
        let parent = path.parent().unwrap_or(&self.root);
        std::fs::create_dir_all(parent)?;

        // Write to a unique sibling temp file, then atomically rename into place.
        let n = TMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let tmp = parent.join(format!("{TMP_PREFIX}{}.{n}", std::process::id()));
        std::fs::write(&tmp, bytes)?;
        match std::fs::rename(&tmp, &path) {
            Ok(()) => Ok(()),
            Err(e) => {
                let _ = std::fs::remove_file(&tmp);
                Err(e.into())
            }
        }
    }

    fn exists(&self, key: &str) -> Result<bool> {
        Ok(self.path_for(key)?.is_file())
    }

    fn put_if_absent(&self, key: &str, bytes: &[u8]) -> Result<bool> {
        use std::io::Write;
        let path = self.path_for(key)?;
        let parent = path.parent().unwrap_or(&self.root);
        std::fs::create_dir_all(parent)?;
        // `create_new` is O_EXCL: the existence check + create is atomic.
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(mut f) => {
                f.write_all(bytes)?;
                Ok(true)
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => Ok(false),
            Err(e) => Err(e.into()),
        }
    }

    fn delete(&self, key: &str) -> Result<()> {
        let path = self.path_for(key)?;
        match std::fs::remove_file(&path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    fn list(&self, prefix: &str) -> Result<Vec<String>> {
        let mut out = Vec::new();
        if self.root.is_dir() {
            collect_keys(&self.root, &self.root, &mut out)?;
        }
        out.retain(|k| k.starts_with(prefix));
        Ok(out)
    }

    fn health(&self) -> Result<()> {
        std::fs::create_dir_all(&self.root)?;
        Ok(())
    }
}

/// Recursively collect `/`-joined keys for every file under `dir`, relative to
/// `base`, skipping in-progress temp files.
fn collect_keys(base: &Path, dir: &Path, out: &mut Vec<String>) -> Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            collect_keys(base, &path, out)?;
        } else if file_type.is_file() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with(TMP_PREFIX) {
                    continue;
                }
            }
            let rel = path
                .strip_prefix(base)
                .map_err(|e| StoreError::Other(e.to_string()))?;
            let mut key_parts = Vec::new();
            for comp in rel.components() {
                match comp {
                    std::path::Component::Normal(os) => {
                        let s = os
                            .to_str()
                            .ok_or_else(|| StoreError::InvalidKey("non-UTF-8 path".into()))?;
                        key_parts.push(s);
                    }
                    _ => return Err(StoreError::Other("unexpected path component".into())),
                }
            }
            out.push(key_parts.join("/"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fs_store_satisfies_contract() {
        let dir = std::env::temp_dir().join(format!(
            "nucleation-fsstore-test-{}-{}",
            std::process::id(),
            line!()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        crate::store::contract::run_contract(&FsStore::new(&dir));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
