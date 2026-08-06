//! `nucleation mesh <in|-> [-o out.glb|-] [--pack P]` — a schematic to a
//! binary glTF, through the same meshing pipeline the 3D view and `render`
//! use. Piped both ways it is one ffmpeg-ish stage:
//!
//! ```text
//! curl -s $URL | nucleation mesh - -o - > build.glb
//! ```
//!
//! The pack defaults to `NUCLEATION_PACK` or the newest installed client
//! jar, exactly like the TUI's GPU view.

use std::path::PathBuf;

use crate::usage_and_exit;

pub(crate) fn mesh_main(args: impl Iterator<Item = String>) {
    let mut input: Option<PathBuf> = None;
    let mut output: Option<PathBuf> = None;
    let mut pack: Option<PathBuf> = None;
    let mut to: Option<String> = None;
    let mut args = args.peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-o" | "--out" => {
                output = Some(PathBuf::from(
                    args.next().unwrap_or_else(|| usage_and_exit()),
                ))
            }
            "--pack" => {
                pack = Some(PathBuf::from(
                    args.next().unwrap_or_else(|| usage_and_exit()),
                ))
            }
            "--to" => to = Some(args.next().unwrap_or_else(|| usage_and_exit())),
            other if other.starts_with("--") => usage_and_exit(),
            other => input = Some(PathBuf::from(other)),
        }
    }
    let Some(input) = input else { usage_and_exit() };
    // No -o: <input stem>.glb beside the input, or stdout when piping in.
    let output = output.unwrap_or_else(|| {
        if super::io::is_stdio(&input) {
            PathBuf::from("-")
        } else {
            input.with_extension("glb")
        }
    });
    let Some(pack) = pack.or_else(super::pack::discover_pack) else {
        eprintln!(
            "no resource pack found — pass --pack <zip or client jar>, set NUCLEATION_PACK, \
             or install Minecraft"
        );
        std::process::exit(2);
    };

    // The container: `--to glb|usdz`, else the output extension, else glb.
    // USDZ is the one macOS Quick Look opens natively — `open build.usdz`
    // is a real 3D viewer with zero extra software.
    let container = to
        .or_else(|| {
            output
                .extension()
                .map(|e| e.to_string_lossy().to_lowercase())
        })
        .unwrap_or_else(|| "glb".to_string());
    if !matches!(container.as_str(), "glb" | "usdz") {
        eprintln!("mesh writes glb or usdz, not {container:?}");
        std::process::exit(2);
    }

    let piping_out = super::io::is_stdio(&output);
    let result = (|| -> Result<(usize, usize), String> {
        let bytes = super::io::read_input(&input)?;
        let manager = nucleation::formats::manager::get_manager();
        let schematic = {
            let manager = manager.lock().map_err(|e| format!("format manager: {e}"))?;
            manager
                .read(&bytes)
                .map_err(|e| format!("{}: unreadable: {e:?}", super::io::display_name(&input)))?
        };
        let source =
            nucleation::meshing::ResourcePackSource::from_file(&pack.display().to_string())
                .map_err(|e| format!("loading resource pack {}: {e:?}", pack.display()))?;
        let mesh = schematic
            .to_mesh(&source, &nucleation::meshing::MeshConfig::default())
            .map_err(|e| format!("meshing: {e:?}"))?;
        let encoded = match container.as_str() {
            "usdz" => mesh
                .to_usdz()
                .map_err(|e| format!("encoding usdz: {e:?}"))?,
            _ => mesh.to_glb().map_err(|e| format!("encoding glb: {e:?}"))?,
        };
        let size = encoded.len();
        super::io::write_output(&output, &encoded)?;
        Ok((bytes.len(), size))
    })();

    match result {
        Ok((in_bytes, out_bytes)) => super::io::status(
            &format!(
                "meshed {} ({in_bytes} bytes) -> {} ({container}, {out_bytes} bytes, pack {})",
                super::io::display_name(&input),
                super::io::display_out(&output),
                pack.file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default()
            ),
            piping_out,
        ),
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(2);
        }
    }
}
