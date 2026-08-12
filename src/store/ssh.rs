//! SSH-backed [`Store`] for a filesystem on another machine.
//!
//! This backend deliberately delegates transport and host verification to the
//! system `ssh` client. It therefore uses the same agent, keys, `~/.ssh/config`,
//! known-hosts policy, ProxyJump settings, and Tailscale connectivity as an
//! interactive SSH session. URLs have the form
//! `ssh://[user@]host/absolute/root`; store keys are paths below that root.

use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};

use super::{Result, Store, StoreError};

const NOT_FOUND: i32 = 44;
const ALREADY_EXISTS: i32 = 45;

/// A remote filesystem store reached through the system OpenSSH client.
#[derive(Clone, Debug)]
pub struct SshStore {
    target: String,
    root: PathBuf,
}

impl SshStore {
    /// Create an SSH store rooted at an absolute remote path.
    pub fn new(target: impl Into<String>, root: impl Into<PathBuf>) -> Result<Self> {
        let target = target.into();
        let root = root.into();
        if target.is_empty() || target.starts_with('-') || target.chars().any(char::is_whitespace) {
            return Err(StoreError::Unsupported("invalid SSH target".into()));
        }
        if !root.is_absolute() {
            return Err(StoreError::Unsupported(
                "SSH store root must be an absolute path".into(),
            ));
        }
        Ok(Self { target, root })
    }

    fn path_for(&self, key: &str) -> Result<PathBuf> {
        if key.is_empty() {
            return Err(StoreError::InvalidKey("empty key".into()));
        }
        let key_path = Path::new(key);
        if key_path.is_absolute()
            || key_path.components().any(|component| {
                !matches!(component, Component::Normal(_))
                    || component.as_os_str().to_str().is_none()
            })
        {
            return Err(StoreError::InvalidKey(key.into()));
        }
        Ok(self.root.join(key_path))
    }

    fn quote(value: &str) -> String {
        format!("'{}'", value.replace('\'', "'\\''"))
    }

    fn command(&self, script: &str) -> Command {
        let mut command = Command::new("ssh");
        command
            .arg("-o")
            .arg("BatchMode=yes")
            .arg("-o")
            .arg("ControlMaster=auto")
            .arg("-o")
            .arg("ControlPersist=60")
            .arg("-o")
            .arg("ControlPath=/tmp/nucleation-ssh-%C")
            .arg(&self.target)
            .arg(script);
        command
    }

    fn run(&self, script: &str) -> Result<std::process::Output> {
        self.command(script)
            .output()
            .map_err(|error| StoreError::Connection(error.to_string()))
    }

    fn run_input(&self, script: &str, bytes: &[u8]) -> Result<std::process::Output> {
        let mut child = self
            .command(script)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| StoreError::Connection(error.to_string()))?;
        child
            .stdin
            .take()
            .ok_or_else(|| StoreError::Io("SSH stdin was unavailable".into()))?
            .write_all(bytes)?;
        child
            .wait_with_output()
            .map_err(|error| StoreError::Connection(error.to_string()))
    }

    fn failed(operation: &str, output: &std::process::Output) -> StoreError {
        let detail = String::from_utf8_lossy(&output.stderr);
        let detail = detail.trim();
        if detail.is_empty() {
            StoreError::Connection(format!("SSH {operation} failed with {}", output.status))
        } else {
            StoreError::Connection(format!("SSH {operation} failed: {detail}"))
        }
    }

    fn write_script(path: &Path, only_if_absent: bool) -> String {
        let path = Self::quote(&path.to_string_lossy());
        let absent = if only_if_absent {
            format!("test ! -e \"$p\" || exit {ALREADY_EXISTS}; lock=\"$p.nuclock\"; mkdir \"$lock\" 2>/dev/null || exit {ALREADY_EXISTS}; trap 'rmdir \"$lock\" 2>/dev/null; rm -f \"$tmp\"' EXIT; test ! -e \"$p\" || exit {ALREADY_EXISTS};")
        } else {
            "trap 'rm -f \"$tmp\"' EXIT;".into()
        };
        format!(
            "set -eu; p={path}; parent=${{p%/*}}; mkdir -p \"$parent\"; tmp=\"$p.nuctmp.$$\"; {absent} cat > \"$tmp\"; mv \"$tmp\" \"$p\"; trap - EXIT; test -z \"${{lock-}}\" || rmdir \"$lock\""
        )
    }
}

impl Store for SshStore {
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let path = Self::quote(&self.path_for(key)?.to_string_lossy());
        let output = self.run(&format!("test -f {path} || exit {NOT_FOUND}; cat {path}"))?;
        match output.status.code() {
            Some(0) => Ok(Some(output.stdout)),
            Some(NOT_FOUND) => Ok(None),
            _ => Err(Self::failed("read", &output)),
        }
    }

    fn put(&self, key: &str, bytes: &[u8]) -> Result<()> {
        let output = self.run_input(&Self::write_script(&self.path_for(key)?, false), bytes)?;
        if output.status.success() {
            Ok(())
        } else {
            Err(Self::failed("write", &output))
        }
    }

    fn exists(&self, key: &str) -> Result<bool> {
        let path = Self::quote(&self.path_for(key)?.to_string_lossy());
        let output = self.run(&format!("test -f {path}"))?;
        match output.status.code() {
            Some(0) => Ok(true),
            Some(1) => Ok(false),
            _ => Err(Self::failed("exists", &output)),
        }
    }

    fn delete(&self, key: &str) -> Result<()> {
        let path = Self::quote(&self.path_for(key)?.to_string_lossy());
        let output = self.run(&format!("rm -f -- {path}"))?;
        if output.status.success() {
            Ok(())
        } else {
            Err(Self::failed("delete", &output))
        }
    }

    fn list(&self, prefix: &str) -> Result<Vec<String>> {
        let root = Self::quote(&self.root.to_string_lossy());
        let output = self.run(&format!("test ! -d {root} || find {root} -type f -print"))?;
        if !output.status.success() {
            return Err(Self::failed("list", &output));
        }
        let root = format!("{}/", self.root.to_string_lossy().trim_end_matches('/'));
        let mut keys = String::from_utf8(output.stdout)
            .map_err(|_| StoreError::Io("remote path is not UTF-8".into()))?
            .lines()
            .filter_map(|path| path.strip_prefix(&root))
            .filter(|key| !key.split('/').any(|part| part.starts_with(".nuctmp.")))
            .filter(|key| key.starts_with(prefix))
            .map(str::to_string)
            .collect::<Vec<_>>();
        keys.sort();
        Ok(keys)
    }

    fn health(&self) -> Result<()> {
        let root = Self::quote(&self.root.to_string_lossy());
        let output = self.run(&format!("mkdir -p {root} && test -d {root} -a -w {root}"))?;
        if output.status.success() {
            Ok(())
        } else {
            Err(Self::failed("health check", &output))
        }
    }

    fn put_if_absent(&self, key: &str, bytes: &[u8]) -> Result<bool> {
        let output = self.run_input(&Self::write_script(&self.path_for(key)?, true), bytes)?;
        match output.status.code() {
            Some(0) => Ok(true),
            Some(ALREADY_EXISTS) => Ok(false),
            _ => Err(Self::failed("conditional write", &output)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requires_absolute_root_and_safe_keys() {
        assert!(SshStore::new("host", "relative").is_err());
        let store = SshStore::new("user@host", "/srv/builds").unwrap();
        assert!(store.path_for("schematics/a.schem").is_ok());
        assert!(store.path_for("../escape").is_err());
        assert!(store.path_for("/escape").is_err());
    }

    #[test]
    fn shell_quote_handles_apostrophes_without_interpolation() {
        assert_eq!(SshStore::quote("a'b"), "'a'\\''b'");
    }
}
