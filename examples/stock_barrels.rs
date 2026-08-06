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

    // `count` is the comparator strength wanted, 1..=15. A comparator reads
    // container fullness: strength s needs roughly s/14 of the container's
    // slots filled, and 15 means every slot full. Filling by slot rather than
    // by stack is what makes "full barrel" mean what the builder meant.
    let slots = if count >= 15 {
        27
    } else {
        (((count as f32 - 1.0) * 27.0 / 14.0).ceil() as usize).max(1)
    };
    for mut entity in barrels {
        let items: Vec<NbtValue> = (0..slots)
            .map(|slot| {
                let mut item = nucleation::nbt::NbtMap::new();
                item.insert("Slot".to_string(), NbtValue::Byte(slot as i8));
                item.insert(
                    "id".to_string(),
                    NbtValue::String("minecraft:redstone".into()),
                );
                item.insert("count".to_string(), NbtValue::Int(64));
                NbtValue::Compound(item)
            })
            .collect();
        let nbt = entity.nbt_mut();
        nbt.insert("Items".to_string(), NbtValue::List(items));
        let position = BlockPosition {
            x: entity.position.0,
            y: entity.position.1,
            z: entity.position.2,
        };
        println!(
            "  {} at {:?} <- {slots} slots of 64 (comparator strength ~{count})",
            entity.id, entity.position
        );
        schematic.set_block_entity(position, entity);
    }

    // Prime the comparators that read those containers.
    //
    // A comparator emits its *stored* OutputSignal, not what its input holds
    // right now — so a door saved with a latched memory cell carries a non-zero
    // signal, and one exported with it zeroed starts unlatched no matter what
    // is in the barrel. Locking a repeater is exactly this: the lock reads the
    // comparator's stored output, so a zeroed one lets the repeater fall open
    // on the first shape update it receives.
    let comparators: Vec<BlockEntity> = schematic
        .get_block_entities_as_list()
        .into_iter()
        .filter(|entity| entity.id.ends_with("comparator"))
        .collect();
    for mut entity in comparators {
        let position = BlockPosition {
            x: entity.position.0,
            y: entity.position.1,
            z: entity.position.2,
        };
        let Some(state) = schematic.get_block(position.x, position.y, position.z) else {
            continue;
        };
        let facing = state
            .properties
            .iter()
            .find(|(k, _)| k == "facing")
            .map(|(_, v)| v.to_string());
        // `getInputSignal` reads the block at FACING.
        let input = match facing.as_deref() {
            Some("north") => (position.x, position.y, position.z - 1),
            Some("south") => (position.x, position.y, position.z + 1),
            Some("west") => (position.x - 1, position.y, position.z),
            Some("east") => (position.x + 1, position.y, position.z),
            _ => continue,
        };
        let stocked = schematic.get_block_entities_as_list().into_iter().any(|other| {
            other.position == input
                && matches!(other.nbt.get("Items"), Some(NbtValue::List(items)) if !items.is_empty())
        });
        if !stocked {
            continue;
        }
        // One item in a 27-slot container is strength 1.
        entity
            .nbt_mut()
            .insert("OutputSignal".to_string(), NbtValue::Int(count as i32));
        println!(
            "  {} at {:?} <- OutputSignal {count}",
            entity.id, entity.position
        );
        schematic.set_block_entity(position, entity);
    }

    std::fs::write(&output, nucleation::litematic::to_litematic(&schematic)?)?;
    println!("wrote {output}");
    Ok(())
}
