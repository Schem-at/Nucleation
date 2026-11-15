//! Emit a native-version oracle fixture through the production schematic importer.
//! Usage: export_tick_oracle_fixture INPUT OUTPUT_JSON OUTPUT_SNBT
use quartz_nbt::{NbtCompound, NbtList, NbtTag};
use serde_json::json;

fn snbt(tag: &NbtTag) -> String {
    match tag {
        NbtTag::Byte(v) => format!("{v}b"),
        NbtTag::Short(v) => format!("{v}s"),
        NbtTag::Int(v) => v.to_string(),
        NbtTag::Long(v) => format!("{v}L"),
        NbtTag::Float(v) => format!("{v:?}f"),
        NbtTag::Double(v) => format!("{v:?}d"),
        NbtTag::String(v) => serde_json::to_string(v).unwrap(),
        NbtTag::List(v) => format!("[{}]", v.iter().map(snbt).collect::<Vec<_>>().join(",")),
        NbtTag::Compound(v) => {
            let mut entries = v.inner().iter().collect::<Vec<_>>();
            entries.sort_by_key(|(key, _)| *key);
            format!(
                "{{{}}}",
                entries
                    .into_iter()
                    .map(|(key, value)| format!(
                        "{}:{}",
                        serde_json::to_string(key).unwrap(),
                        snbt(value)
                    ))
                    .collect::<Vec<_>>()
                    .join(",")
            )
        }
        NbtTag::ByteArray(v) => format!(
            "[B;{}]",
            v.iter()
                .map(|v| format!("{v}b"))
                .collect::<Vec<_>>()
                .join(",")
        ),
        NbtTag::IntArray(v) => format!(
            "[I;{}]",
            v.iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(",")
        ),
        NbtTag::LongArray(v) => format!(
            "[L;{}]",
            v.iter()
                .map(|v| format!("{v}L"))
                .collect::<Vec<_>>()
                .join(",")
        ),
    }
}

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    assert_eq!(args.len(), 4, "INPUT OUTPUT_JSON OUTPUT_SNBT");
    let bytes = std::fs::read(&args[1]).expect("read schematic");
    let schematic = nucleation::formats::manager::get_manager()
        .lock()
        .unwrap()
        .read(&bytes)
        .expect("production schematic import");
    let exported = nucleation::formats::gametest::to_gametest_snbt(&schematic);
    let structure = mc_tick::Structure::parse(&exported).expect("parse production export");
    let compound = quartz_nbt::snbt::parse(&exported).expect("parse full exported NBT");
    let blocks = compound.get::<_, &NbtList>("blocks").unwrap();
    let fixture = blocks
        .iter()
        .map(|tag| {
            let NbtTag::Compound(block) = tag else {
                panic!("structure block compound")
            };
            let position = block
                .get::<_, &NbtList>("pos")
                .unwrap()
                .iter()
                .map(|v| match v {
                    NbtTag::Int(n) => *n,
                    _ => panic!("position int"),
                })
                .collect::<Vec<_>>();
            let index = block.get::<_, i32>("state").unwrap() as usize;
            let mut row = json!({"pos":position,"state":structure.palette[index]});
            if let Ok(nbt) = block.get::<_, &NbtCompound>("nbt") {
                let encoded = snbt(&NbtTag::Compound(nbt.clone()));
                assert_eq!(
                    &quartz_nbt::snbt::parse(&encoded).expect("roundtrip block entity"),
                    nbt
                );
                row["nbt"] = encoded.into();
            }
            row
        })
        .collect::<Vec<_>>();
    let (x, y, z) = structure.size;
    let result = json!({"format_version":1,"data_version":structure.data_version,
        "size":[x,y,z],"fixture":fixture,"entity_count":structure.entities.len(),
        "block_entity_count":structure.block_entities.len(),"source_input":args[1]});
    std::fs::write(
        &args[2],
        serde_json::to_string_pretty(&result).unwrap() + "\n",
    )
    .unwrap();
    std::fs::write(&args[3], exported).unwrap();
    println!(
        "{}",
        json!({"blocks":structure.blocks.len(),"entities":structure.entities.len(),
        "block_entities":structure.block_entities.len(),"data_version":structure.data_version})
    );
}
