//! Select records before serializing. Only the current matching record is buffered.
use super::scene::{load, value};
use nucleation::nbt::{NbtMap, NbtValue};
use serde_json::{json, Value};
use std::io::{self, Write};
use std::path::PathBuf;

fn compound(map: &NbtMap) -> Value {
    Value::Object(map.iter().map(|(key, v)| (key.clone(), plain(v))).collect())
}
fn plain(tag: &NbtValue) -> Value {
    use NbtValue::*;
    match tag {
        Byte(x) => json!(x),
        Short(x) => json!(x),
        Int(x) => json!(x),
        Long(x) => json!(x),
        Float(x) => json!(x),
        Double(x) => json!(x),
        String(x) => json!(x),
        ByteArray(x) => json!(x),
        IntArray(x) => json!(x),
        LongArray(x) => json!(x),
        List(xs) => Value::Array(xs.iter().map(plain).collect()),
        Compound(map) => compound(map),
    }
}
const HELP: &str = "nucleation query entities|block-entities FILE [--id minecraft:ID] [--nbt] [--limit N]\nWrites one matching record per line (region, id, pos, optional nbt).";
pub(crate) fn query_main(mut args: impl Iterator<Item = String>) {
    if let Err(error) = run(&mut args) {
        eprintln!("{error}");
        std::process::exit(2);
    }
}
fn run(args: &mut impl Iterator<Item = String>) -> Result<(), String> {
    let blocks = match args.next().as_deref() {
        Some("--help") => {
            println!("{HELP}");
            return Ok(());
        }
        Some("entities") => false,
        Some("block-entities" | "tile-entities") => true,
        _ => return Err(HELP.into()),
    };
    let (mut file, mut id, mut nbt, mut limit) = (None, None, false, usize::MAX);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--id" => id = Some(value(args, &arg)?),
            "--limit" => {
                limit = value(args, &arg)?
                    .parse()
                    .map_err(|_| "--limit needs a non-negative integer")?
            }
            "--nbt" => nbt = true,
            s if (!s.starts_with('-') || s == "-") && file.is_none() => {
                file = Some(PathBuf::from(s))
            }
            _ => return Err(HELP.into()),
        }
    }
    let scene = load(&file.ok_or(HELP)?)?;
    let mut out = io::BufWriter::new(io::stdout().lock());
    let mut write_row = |row: Value| -> io::Result<()> {
        serde_json::to_writer(&mut out, &row)?;
        out.write_all(b"\n")
    };
    let result: io::Result<()> = (|| {
        for (name, region) in scene.get_all_regions() {
            if blocks {
                for b in region.block_entities.values() {
                    if limit == 0 {
                        return Ok(());
                    }
                    if id.as_deref().is_some_and(|i| i != b.id.as_str()) {
                        continue;
                    }
                    let mut row = json!({"region":name,"id":b.id.as_str(),"pos":b.position});
                    if nbt {
                        row["nbt"] = compound(&b.nbt)
                    }
                    write_row(row)?;
                    limit -= 1;
                }
            } else {
                for e in &region.entities {
                    if limit == 0 {
                        return Ok(());
                    }
                    if id.as_deref().is_some_and(|i| i != e.id.as_str()) {
                        continue;
                    }
                    let mut row = json!({"region":name,"id":e.id,"pos":e.position});
                    if nbt {
                        row["nbt"] = plain(&NbtValue::from_quartz_nbt(&e.to_nbt()))
                    }
                    write_row(row)?;
                    limit -= 1;
                }
            }
        }
        Ok(())
    })();
    match result.and_then(|_| out.flush()) {
        Err(e) if e.kind() == io::ErrorKind::BrokenPipe => Ok(()),
        other => other.map_err(|e| e.to_string()),
    }
}
