//! `nucleation port` — every `.snbt` under the given paths becomes a
//! self-contained `.litematic` test under `--out`, mirroring relative paths.

use std::path::{Path, PathBuf};

use crate::commands::test::{discover, display_path, load_structure};
use crate::usage_and_exit;

pub(crate) fn port_main(args: impl Iterator<Item = String>) {
    let mut paths: Vec<PathBuf> = Vec::new();
    let mut specs: Option<PathBuf> = None;
    let mut out: Option<PathBuf> = None;
    let mut max_ticks: u64 = 400;
    let mut args = args.peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--path" => paths.push(PathBuf::from(
                args.next().unwrap_or_else(|| usage_and_exit()),
            )),
            "--specs" => {
                specs = Some(PathBuf::from(
                    args.next().unwrap_or_else(|| usage_and_exit()),
                ))
            }
            "--out" => {
                out = Some(PathBuf::from(
                    args.next().unwrap_or_else(|| usage_and_exit()),
                ))
            }
            "--max-ticks" => {
                max_ticks = args
                    .next()
                    .and_then(|n| n.parse().ok())
                    .unwrap_or_else(|| usage_and_exit())
            }
            other if other.starts_with("--") => usage_and_exit(),
            other => paths.push(PathBuf::from(other)),
        }
    }
    let Some(out) = out else { usage_and_exit() };
    if paths.is_empty() {
        usage_and_exit();
    }

    let (mut ported, mut skipped, mut failed) = (0usize, 0usize, 0usize);
    for (root, file) in discover(&paths, "") {
        if !file.extension().is_some_and(|e| e == "snbt") {
            continue;
        }
        let rel = file.strip_prefix(&root).unwrap_or(&file).to_path_buf();
        let shown = display_path(&file);
        // The probe builds real simulations; a structure the engine panics on
        // (an unsupported entity) must fail its own row, not the whole run.
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            port_one(&root, &file, specs.as_deref(), &out, &rel, max_ticks)
        }))
        .unwrap_or_else(|panic| {
            let why = panic
                .downcast_ref::<String>()
                .map(String::as_str)
                .or_else(|| panic.downcast_ref::<&str>().copied())
                .unwrap_or("the harness panicked");
            Err(why.to_string())
        });
        match outcome {
            Ok(source) => {
                ported += 1;
                println!("ported  {shown}  ({source})");
            }
            Err(why) if why.starts_with("skipped") => {
                skipped += 1;
                println!("skipped {shown}  ({why})");
            }
            Err(why) => {
                failed += 1;
                eprintln!("FAILED  {shown}  {why}");
            }
        }
    }
    println!(
        "\n{ported} ported, {skipped} skipped, {failed} failed → {}",
        out.display()
    );
    std::process::exit(i32::from(failed > 0));
}

/// Convert one structure. `Ok` names the descriptor's source; `Err` beginning
/// with `skipped` is a non-error (nothing to port).
fn port_one(
    root: &Path,
    file: &Path,
    specs: Option<&Path>,
    out: &Path,
    rel: &Path,
    max_ticks: u64,
) -> Result<String, String> {
    let what = file
        .file_stem()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let text =
        std::fs::read_to_string(file).map_err(|e| format!("reading {}: {e}", file.display()))?;

    // The output carrier needs a UniversalSchematic; only the modern
    // `data:`-flavor imports to one. The old engine flavor pairs with
    // sidecar descriptors already and needs no porting.
    let mut schematic =
        nucleation::formats::structure_snbt::from_structure_snbt(text.as_bytes())
            .map_err(|_| "skipped: engine-flavor snbt, keep its sidecar descriptor".to_string())?;

    // A hand-written descriptor wins; the synthesizer is the fallback.
    let sidecar = file.with_extension("").with_extension("test.json");
    let mirrored = specs.map(|s| {
        s.join(file.strip_prefix(root).unwrap_or(file))
            .with_extension("")
            .with_extension("test.json")
    });
    let (spec, source) = if sidecar.is_file() {
        (
            std::fs::read_to_string(&sidecar).map_err(|e| e.to_string())?,
            "hand-written sidecar",
        )
    } else if let Some(mirrored) = mirrored.filter(|m| m.is_file()) {
        (
            std::fs::read_to_string(&mirrored).map_err(|e| e.to_string())?,
            "hand-written spec",
        )
    } else {
        let structure = load_structure(file)?;
        // Probe: what does the engine refuse to simulate? Those blocks get
        // asserted inert in the synthesized spec — machinery blocks the
        // engine models keep their behaviour, terrain and mob-test dressing
        // do nothing. A machine gutted by that goes honestly red on its
        // accept claim rather than lying green.
        let mut inert: Vec<String> = Vec::new();
        for _ in 0..4 {
            match mc_test::try_build_sim(
                &structure,
                mc_test::mc_tick::Pos::new(0, 0, 0),
                mc_test::SettleMode::Placement,
                &[],
                &inert,
                None,
                &what,
            ) {
                Ok(_) => break,
                Err(report) => {
                    // "...simulated as nothing: a, b[c=d], e" — take the names.
                    let Some(list) = report.rsplit("simulated as nothing: ").next() else {
                        break;
                    };
                    let before = inert.len();
                    for descriptor in list.split(", ") {
                        let name = descriptor.split('[').next().unwrap_or(descriptor).trim();
                        if name.starts_with("minecraft:")
                            && !inert.iter().any(|n| n == name)
                            && name != "minecraft:test_block"
                        {
                            inert.push(name.to_string());
                        }
                    }
                    if inert.len() == before {
                        break;
                    }
                }
            }
        }
        // The litematic carrier compacts to the non-air bounding box; the
        // spec pre-shifts its positions and records the original origin so
        // absolute-position-hashed update order stays what it was — a
        // one-block shift phase-shifts observer chains.
        let mut min = (i32::MAX, i32::MAX, i32::MAX);
        for (pos, entry) in &structure.blocks {
            let name = structure.palette[*entry]
                .split('[')
                .next()
                .unwrap_or_default();
            if name != "minecraft:air" {
                min = (min.0.min(pos.x), min.1.min(pos.y), min.2.min(pos.z));
            }
        }
        let shift = if min.0 == i32::MAX { (0, 0, 0) } else { min };
        let spec = mc_test::synthesize_block_based(&structure, &what, max_ticks, &inert, shift)
            .ok_or(
                "skipped: no expressible test-block claim and no hand-written spec".to_string(),
            )?;
        (spec, "synthesized from test blocks")
    };

    schematic.metadata.embedded_test = Some(spec.trim().to_string());
    let bytes = nucleation::formats::litematic::to_litematic(&schematic)
        .map_err(|e| format!("writing litematic: {e:?}"))?;
    let dest = out.join(rel).with_extension("litematic");
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(&dest, bytes).map_err(|e| format!("writing {}: {e}", dest.display()))?;
    Ok(source.to_string())
}
