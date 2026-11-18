//! Author a saved scene with the same block descriptors the simulator reads.
use super::geometry::{BrushOptions, Operation};
use nucleation::UniversalSchematic;
use std::path::PathBuf;

pub(crate) fn scene_main(args: impl Iterator<Item = String>) {
    if let Err(error) = run(args) {
        eprintln!("{error}");
        std::process::exit(2);
    }
}

fn run(mut args: impl Iterator<Item = String>) -> Result<(), String> {
    let mut input = None;
    let mut output = None;
    let mut format = None;
    let mut name = None;
    let mut version = None;
    let mut operations = Vec::new();
    let mut brush = BrushOptions::default();
    while let Some(arg) = args.next() {
        if arg == "--help" {
            println!("nucleation scene [input] --set 'X,Y,Z=BLOCK_STATE'... -o scene.litematic\n  --sphere X,Y,Z  --cuboid X,Y,Z:X,Y,Z  --curve '[12*cos(t),4*t,12*sin(t)]' (jq)\n  --block STATE  --radius R  --steps N (global shape settings)\n  --name NAME  --data-version N  --to FORMAT  (- reads/writes a pipe)");
            return Ok(());
        }
        match arg.as_str() {
            "-o" | "--out" => output = Some(PathBuf::from(value(&mut args, &arg)?)),
            "--to" => format = Some(value(&mut args, &arg)?),
            "--name" => name = Some(value(&mut args, &arg)?),
            "--data-version" => {
                version = Some(
                    value(&mut args, &arg)?
                        .parse::<i32>()
                        .map_err(|_| "--data-version needs an integer")?,
                )
            }
            "--set" => {
                let v = value(&mut args, &arg)?;
                let (pos, state) = v.split_once('=').ok_or("--set needs X,Y,Z=BLOCK_STATE")?;
                operations.push(Operation::Set(position(pos)?, state.to_string()));
            }
            "--block" => brush.block = value(&mut args, &arg)?,
            "--radius" => {
                brush.radius = value(&mut args, &arg)?
                    .parse()
                    .map_err(|_| "invalid radius")?
            }
            "--steps" => {
                brush.steps = value(&mut args, &arg)?
                    .parse()
                    .map_err(|_| "invalid steps")?
            }
            "--curve" => operations.push(Operation::Curve(value(&mut args, &arg)?)),
            "--sphere" => operations.push(Operation::Sphere(position(&value(&mut args, &arg)?)?)),
            "--cuboid" => {
                let corners = value(&mut args, &arg)?;
                let (a, b) = corners
                    .split_once(':')
                    .ok_or("--cuboid needs x,y,z:x,y,z")?;
                operations.push(Operation::Cuboid(position(a)?, position(b)?));
            }
            "-" => {
                if input.replace(PathBuf::from(arg)).is_some() {
                    return Err("Only one input scene is allowed".into());
                }
            }
            _ if arg.starts_with('-') => return Err(format!("Unknown scene option {arg}")),
            _ => {
                if input.replace(PathBuf::from(arg)).is_some() {
                    return Err("Only one input scene is allowed".into());
                }
            }
        }
    }
    let output = output.ok_or("scene requires -o OUTPUT; use --help for an example")?;
    let mut scene = if let Some(input) = input {
        load(&input)?
    } else {
        let mut s = UniversalSchematic::new(name.clone().unwrap_or_else(|| "CLI scene".into()));
        s.metadata.mc_version = Some(nucleation::dataconverter::CANONICAL_DATA_VERSION);
        s
    };
    if let Some(name) = name {
        scene.metadata.name = Some(name);
    }
    if let Some(version) = version {
        if version < 0 {
            return Err("Data version must be non-negative".into());
        }
        scene.metadata.mc_version = Some(version);
        scene.metadata.source_data_version = Some(version);
    }
    for operation in operations {
        operation.apply(&mut scene, &brush)?;
    }
    if scene.iter_blocks().all(|(_, s)| s.name == "minecraft:air") {
        return Err("Scene has no blocks; add --set X,Y,Z=STATE".into());
    }
    if super::io::is_stdio(&output) && format.is_none() {
        return Err("stdout needs --to FORMAT".into());
    }
    let manager = nucleation::formats::manager::get_manager();
    let manager = manager.lock().map_err(|e| e.to_string())?;
    let encoded = match format {
        Some(format) => manager.write(&format, &scene, None),
        None => manager.write_auto(&output.display().to_string(), &scene, None),
    }
    .map_err(|e| format!("Writing scene: {e:?}"))?;
    super::io::write_output(&output, &encoded)?;
    super::io::status(
        &format!("Saved scene -> {}", super::io::display_out(&output)),
        super::io::is_stdio(&output),
    );
    Ok(())
}

pub(crate) fn value(
    args: &mut impl Iterator<Item = String>,
    option: &str,
) -> Result<String, String> {
    args.next().ok_or_else(|| format!("{option} needs a value"))
}

pub(crate) fn position(s: &str) -> Result<[i32; 3], String> {
    let v: Vec<i32> = s
        .split(',')
        .map(str::parse)
        .collect::<Result<_, _>>()
        .map_err(|_| format!("Expected integer X,Y,Z, got {s:?}"))?;
    v.try_into()
        .map_err(|_| format!("Expected three coordinates, got {s:?}"))
}

pub(crate) fn load(path: &std::path::Path) -> Result<UniversalSchematic, String> {
    let bytes = super::io::read_input(path)?;
    nucleation::formats::manager::get_manager()
        .lock()
        .map_err(|e| e.to_string())?
        .read(&bytes)
        .map_err(|e| format!("Reading {}: {e:?}", super::io::display_name(path)))
}
