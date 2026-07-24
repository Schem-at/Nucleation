//! Streaming native video encoding for rendered RGBA animation frames.

use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread::{self, JoinHandle};

use super::RenderError;

const STDERR_LIMIT: usize = 1_048_576;
static TEMP_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoCodec {
    ProRes4444,
    H264,
}

#[derive(Debug, Clone)]
pub struct VideoConfig {
    pub fps: f64,
    pub codec: VideoCodec,
    pub ffmpeg_path: Option<PathBuf>,
}

impl VideoConfig {
    pub fn prores_4444(fps: f64) -> Result<Self, String> {
        Self::new(fps, VideoCodec::ProRes4444)
    }

    pub fn h264(fps: f64) -> Result<Self, String> {
        Self::new(fps, VideoCodec::H264)
    }

    fn new(fps: f64, codec: VideoCodec) -> Result<Self, String> {
        if !fps.is_finite() || fps < 1.0 {
            return Err("video fps must be finite and at least 1.0".to_string());
        }
        Ok(Self {
            fps,
            codec,
            ffmpeg_path: None,
        })
    }

    pub fn set_ffmpeg_path(&mut self, path: impl Into<PathBuf>) {
        self.ffmpeg_path = Some(path.into());
    }

    pub fn validate_output(&self, output: &Path) -> Result<(), String> {
        let extension = output
            .extension()
            .and_then(|value| value.to_str())
            .map(str::to_ascii_lowercase)
            .unwrap_or_default();
        match self.codec {
            VideoCodec::ProRes4444 if extension != "mov" => {
                Err("ProRes 4444 output must use a .mov container".to_string())
            }
            VideoCodec::H264 if extension != "mp4" && extension != "mov" => {
                Err("H.264 output must use a .mp4 or .mov container".to_string())
            }
            _ => Ok(()),
        }
    }

    pub fn ffmpeg_args(&self, width: u32, height: u32, output: &Path) -> Vec<String> {
        let mut args = vec![
            "-y".to_string(),
            "-loglevel".to_string(),
            "error".to_string(),
            "-f".to_string(),
            "rawvideo".to_string(),
            "-pixel_format".to_string(),
            "rgba".to_string(),
            "-video_size".to_string(),
            format!("{width}x{height}"),
            "-framerate".to_string(),
            self.fps.to_string(),
            "-i".to_string(),
            "pipe:0".to_string(),
            "-an".to_string(),
        ];
        match self.codec {
            VideoCodec::ProRes4444 => args.extend(
                [
                    "-c:v",
                    "prores_ks",
                    "-profile:v",
                    "4",
                    "-pix_fmt",
                    "yuva444p10le",
                    "-alpha_bits",
                    "16",
                ]
                .into_iter()
                .map(str::to_string),
            ),
            VideoCodec::H264 => args.extend(
                [
                    "-c:v",
                    "libx264",
                    "-pix_fmt",
                    "yuv420p",
                    "-crf",
                    "18",
                    "-movflags",
                    "+faststart",
                ]
                .into_iter()
                .map(str::to_string),
            ),
        }
        args.push(output.to_string_lossy().into_owned());
        args
    }
}

fn temporary_output_path(output: &Path) -> PathBuf {
    let parent = output.parent().unwrap_or_else(|| Path::new("."));
    let stem = output
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("video");
    let extension = output
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("tmp");
    let id = TEMP_ID.fetch_add(1, Ordering::Relaxed);
    parent.join(format!(
        ".{stem}.nucleation-{}-{id}.{extension}",
        std::process::id()
    ))
}

fn drain_stderr(mut stderr: impl Read) -> (Vec<u8>, bool) {
    let mut captured = Vec::new();
    let mut truncated = false;
    let mut chunk = [0u8; 8192];
    loop {
        match stderr.read(&mut chunk) {
            Ok(0) | Err(_) => break,
            Ok(count) => {
                let remaining = STDERR_LIMIT.saturating_sub(captured.len());
                captured.extend_from_slice(&chunk[..count.min(remaining)]);
                truncated |= count > remaining;
            }
        }
    }
    (captured, truncated)
}

pub(crate) struct VideoEncoder {
    child: Option<Child>,
    stdin: Option<ChildStdin>,
    stderr_thread: Option<JoinHandle<(Vec<u8>, bool)>>,
    expected_frame_bytes: usize,
    output: PathBuf,
    temporary_output: PathBuf,
}

impl VideoEncoder {
    pub(crate) fn start(
        config: &VideoConfig,
        width: u32,
        height: u32,
        output: &Path,
    ) -> Result<Self, RenderError> {
        config
            .validate_output(output)
            .map_err(RenderError::VideoEncode)?;
        let expected_frame_bytes = (width as usize)
            .checked_mul(height as usize)
            .and_then(|pixels| pixels.checked_mul(4))
            .ok_or_else(|| RenderError::VideoEncode("video dimensions overflow".to_string()))?;
        let executable = config
            .ffmpeg_path
            .clone()
            .unwrap_or_else(|| PathBuf::from("ffmpeg"));
        let temporary_output = temporary_output_path(output);
        let mut child = Command::new(&executable)
            .args(config.ffmpeg_args(width, height, &temporary_output))
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| {
                RenderError::VideoEncode(format!(
                    "failed to start FFmpeg at '{}': {error}. Install ffmpeg or set an explicit encoder path",
                    executable.display()
                ))
            })?;
        let Some(stdin) = child.stdin.take() else {
            let _ = child.kill();
            let _ = child.wait();
            return Err(RenderError::VideoEncode(
                "FFmpeg stdin was unavailable".to_string(),
            ));
        };
        let Some(stderr) = child.stderr.take() else {
            let _ = child.kill();
            let _ = child.wait();
            return Err(RenderError::VideoEncode(
                "FFmpeg stderr was unavailable".to_string(),
            ));
        };
        let stderr_thread = thread::spawn(move || drain_stderr(stderr));
        Ok(Self {
            child: Some(child),
            stdin: Some(stdin),
            stderr_thread: Some(stderr_thread),
            expected_frame_bytes,
            output: output.to_path_buf(),
            temporary_output,
        })
    }

    fn take_stderr(&mut self) -> String {
        let Some(thread) = self.stderr_thread.take() else {
            return String::new();
        };
        let Ok((bytes, truncated)) = thread.join() else {
            return "FFmpeg stderr reader panicked".to_string();
        };
        let mut text = String::from_utf8_lossy(&bytes).trim().to_string();
        if truncated {
            text.push_str(" [stderr truncated]");
        }
        text
    }

    fn abort(&mut self) -> String {
        self.stdin.take();
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        let stderr = self.take_stderr();
        let _ = fs::remove_file(&self.temporary_output);
        stderr
    }

    pub(crate) fn write_frame(&mut self, rgba: &[u8]) -> Result<(), RenderError> {
        if rgba.len() != self.expected_frame_bytes {
            return Err(RenderError::VideoEncode(format!(
                "RGBA frame has {} bytes; expected {}",
                rgba.len(),
                self.expected_frame_bytes
            )));
        }
        let result = self
            .stdin
            .as_mut()
            .ok_or_else(|| RenderError::VideoEncode("FFmpeg stdin is closed".to_string()))?
            .write_all(rgba);
        if let Err(error) = result {
            let stderr = self.abort();
            let suffix = (!stderr.is_empty())
                .then(|| format!(": {stderr}"))
                .unwrap_or_default();
            return Err(RenderError::VideoEncode(format!(
                "failed to stream frame to FFmpeg: {error}{suffix}"
            )));
        }
        Ok(())
    }

    pub(crate) fn finish(mut self) -> Result<(), RenderError> {
        self.stdin.take();
        let status = self
            .child
            .take()
            .expect("video encoder child exists until finish")
            .wait()
            .map_err(|error| {
                RenderError::VideoEncode(format!("failed waiting for FFmpeg: {error}"))
            })?;
        let stderr = self.take_stderr();
        if !status.success() {
            let _ = fs::remove_file(&self.temporary_output);
            return Err(RenderError::VideoEncode(format!(
                "FFmpeg exited with {status}: {stderr}"
            )));
        }
        fs::rename(&self.temporary_output, &self.output).map_err(|error| {
            let _ = fs::remove_file(&self.temporary_output);
            RenderError::VideoEncode(format!(
                "failed to publish encoded video '{}' to '{}': {error}",
                self.temporary_output.display(),
                self.output.display()
            ))
        })
    }
}

impl Drop for VideoEncoder {
    fn drop(&mut self) {
        if self.child.is_some() || self.stderr_thread.is_some() {
            self.abort();
        }
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    fn test_dir(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "nucleation-video-{name}-{}-{}",
            std::process::id(),
            TEMP_ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        path
    }

    fn executable(path: &Path, body: &str) {
        fs::write(path, body).unwrap();
        let mut permissions = fs::metadata(path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions).unwrap();
    }

    #[test]
    fn drains_large_stderr_and_replaces_output_transactionally() {
        let dir = test_dir("drain");
        let script = dir.join("fake-ffmpeg");
        executable(
            &script,
            "#!/usr/bin/env bash\nhead -c 262144 /dev/zero >&2\ncat >/dev/null\nout=${!#}\nprintf encoded > \"$out\"\n",
        );
        let output = dir.join("video.mp4");
        fs::write(&output, b"old").unwrap();
        let mut config = VideoConfig::h264(1.0).unwrap();
        config.set_ffmpeg_path(&script);

        let mut encoder = VideoEncoder::start(&config, 1, 1, &output).unwrap();
        encoder.write_frame(&[0, 0, 0, 0]).unwrap();
        encoder.finish().unwrap();

        assert_eq!(fs::read(&output).unwrap(), b"encoded");
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 2);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn encoder_failure_preserves_existing_output_and_cleans_temporary_file() {
        let dir = test_dir("failure");
        let script = dir.join("fake-ffmpeg");
        executable(
            &script,
            "#!/usr/bin/env bash\nhead -c 262144 /dev/zero >&2\ncat >/dev/null\nexit 42\n",
        );
        let output = dir.join("video.mp4");
        fs::write(&output, b"old").unwrap();
        let mut config = VideoConfig::h264(1.0).unwrap();
        config.set_ffmpeg_path(&script);

        let mut encoder = VideoEncoder::start(&config, 1, 1, &output).unwrap();
        encoder.write_frame(&[0, 0, 0, 0]).unwrap();
        let error = encoder.finish().unwrap_err().to_string();

        assert!(error.contains("42"), "{error}");
        assert!(error.contains("stderr truncated") || error.len() > 1_000);
        assert_eq!(fs::read(&output).unwrap(), b"old");
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 2);
        fs::remove_dir_all(dir).unwrap();
    }
}
