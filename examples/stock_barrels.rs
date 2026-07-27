//! Put items into a schematic's barrels, and write it back out.
//!
//!     cargo run --example stock_barrels -- <in.litematic> <out.litematic> [count]
//!
//! The vault door reads a barrel with a comparator to hold a repeater locked.
//! The litematic as exported has that barrel empty, so the comparator reads 0,
//! the repeater never locks, and the door's memory cell starts in the wrong
//! state. Whether that is an export artefact or how the build was saved, the
//! way to find out is to stock the barrel and see whether the door comes to
//! life — so this does exactly that, on the litematic, keeping it the one
//! source of truth rather than hand-editing the .snbt downstream of it.
//!
//! `count` items land in slot 0. A comparator reads container *fullness*, so
//! one item in a 27-slot barrel is signal strength 1 — enough to lock a
//! repeater, which is all the mechanism needs.
use nucleation::block_entity::BlockEntity;
use nucleation::block_position::BlockPosition;
use nucleation::nbt::NbtValue;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let (Some(input), Some(output)) = (args.next(), args.next()) else {
        eprintln!("usage: stock_barrels <in.litematic> <out.litematic> [count]");
        std::process::exit(2);
    };
    let count: i8 = args.next().and_then(|v| v.parse().ok()).unwrap_or(1);

    let data = std::fs::read(&input)?;
    let mut schematic = nucleation::litematic::from_litematic(&data)?;

    let barrels: Vec<BlockEntity> = schematic
        .get_block_entities_as_list()
        .into_iter()
        .filter(|entity| entity.id.ends_with("barrel") || entity.id.ends_with("chest"))
        .collect();

    if barrels.is_empty() {
        eprintln!("no containers in {input}");
        std::process::exit(1);
    }

    for mut entity in barrels {
        let mut item = nucleation::nbt::NbtMap::new();
        item.insert("Slot".to_string(), NbtValue::Byte(0));
        item.insert("id".to_string(), NbtValue::String("minecraft:redstone".into()));
        item.insert("count".to_string(), NbtValue::Int(count as i32));
        let nbt = entity.nbt_mut();
        nbt.insert("Items".to_string(), NbtValue::List(vec![NbtValue::Compound(item)]));
        let position = BlockPosition {
            x: entity.position.0,
            y: entity.position.1,
            z: entity.position.2,
        };
        println!(
            "  {} at {:?} <- {count} x minecraft:redstone",
            entity.id, entity.position
        );
        schematic.set_block_entity(position, entity);
    }

    std::fs::write(&output, nucleation::litematic::to_litematic(&schematic)?)?;
    println!("wrote {output}");
    Ok(())
}
