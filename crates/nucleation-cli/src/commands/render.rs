//! `nucleation render <file> --pack <resourcepack.zip> -o out.png` —
//! a PNG snapshot through nucleation's GPU pipeline. Isometric by default.
//!
//! Compiled behind this crate's `render` feature (which turns on
//! `nucleation/rendering` and its wgpu stack); the stub in `main.rs` explains
//! how to enable it otherwise.

use std::path::PathBuf;

use nucleation::meshing::{MeshConfig, ResourcePackSource};
use nucleation::rendering::{render_meshes_png, RenderConfig};

use crate::usage_and_exit;

pub(crate) fn render_main(args: impl Iterator<Item = String>) {
    let mut input: Option<PathBuf> = None;
    let mut pack: Option<PathBuf> = None;
    let mut output: Option<PathBuf> = None;
    let (mut width, mut height) = (1024u32, 1024u32);
    let mut args = args.peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--pack" => {
                pack = Some(PathBuf::from(
                    args.next().unwrap_or_else(|| usage_and_exit()),
                ))
            }
            "-o" | "--out" => {
                output = Some(PathBuf::from(
                    args.next().unwrap_or_else(|| usage_and_exit()),
                ))
            }
            "--width" => {
                width = args
                    .next()
                    .and_then(|n| n.parse().ok())
                    .unwrap_or_else(|| usage_and_exit())
            }
            "--height" => {
                height = args
                    .next()
                    .and_then(|n| n.parse().ok())
                    .unwrap_or_else(|| usage_and_exit())
            }
            other if other.starts_with("--") => usage_and_exit(),
            other => input = Some(PathBuf::from(other)),
        }
    }
    let (Some(input), Some(pack), Some(output)) = (input, pack, output) else {
        eprintln!(
            "render needs an input file, --pack <resourcepack.zip or client jar>, and -o out.png"
        );
        std::process::exit(2);
    };

    let result = (|| -> Result<usize, String> {
        let bytes = super::io::read_input(&input)?;
        let manager = nucleation::formats::manager::get_manager();
        let schematic = {
            let manager = manager.lock().map_err(|e| format!("format manager: {e}"))?;
            manager
                .read(&bytes)
                .map_err(|e| format!("{}: unreadable: {e:?}", input.display()))?
        };
        let pack = ResourcePackSource::from_file(&pack.display().to_string())
            .map_err(|e| format!("loading resource pack: {e:?}"))?;
        let mesh = schematic
            .to_mesh(&pack, &MeshConfig::default())
            .map_err(|e| format!("meshing: {e:?}"))?;
        let config = RenderConfig {
            width,
            height,
            ..RenderConfig::isometric()
        };
        let png =
            render_meshes_png(&[mesh], &config, None).map_err(|e| format!("rendering: {e:?}"))?;
        let size = png.len();
        super::io::write_output(&output, &png)?;
        Ok(size)
    })();

    let piping_out = super::io::is_stdio(&output);
    match result {
        Ok(size) => super::io::status(
            &format!(
                "rendered {} -> {} ({size} bytes)",
                super::io::display_name(&input),
                super::io::display_out(&output)
            ),
            piping_out,
        ),
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(2);
        }
    }
}
