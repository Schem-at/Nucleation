//! Reproduction of a user-reported asymmetry in MCHPRS-driven redstone:
//! two torches placed symmetrically off a wire-junction end up in
//! different lit-states after a single lever toggle.
//!
//! Setup (as the user reported, in Python):
//!   - 5x5 stone-bricks base at y=0
//!   - lever at (-1, 1, 2), facing west
//!   - stone block at (0, 1, 2)
//!   - wire at (1, 1, 2)            (east-west)
//!   - repeater at (2, 1, 2) facing west, delay 1
//!   - wire at (3, 1, 2)            (north-south-west junction)
//!   - wire at (3, 1, 1)            (north-south)
//!   - wire at (3, 1, 3)            (north-south)
//!   - white_concrete at (3, 1, 0) and (3, 1, 4)
//!   - redstone_wall_torch at (4, 1, 0) and (4, 1, 4) facing east, lit
//!
//! Expected: after toggling the lever ON, BOTH torches respond identically
//! (in vanilla, both go OUT because the concrete blocks behind them get
//! powered).

#[cfg(feature = "simulation")]
mod tests {
    use nucleation::simulation::MchprsWorld;
    use nucleation::UniversalSchematic;

    fn build_circuit() -> UniversalSchematic {
        let mut s = UniversalSchematic::new("torch-symmetry".to_string());

        // 1x5 stone-brick floor along z at y=0
        for z in 0..5 {
            s.set_block_from_string(0, 0, z, "minecraft:stone_bricks")
                .unwrap();
        }

        // Central stone block + lever attached to its west face.
        s.set_block_from_string(0, 1, 2, "minecraft:stone_bricks").unwrap();
        s.set_block_from_string(
            -1,
            1,
            2,
            "minecraft:lever[face=wall,facing=west,powered=false]",
        )
        .unwrap();

        // Wires fanning out north and south from the central block.
        s.set_block_from_string(
            0,
            1,
            1,
            "minecraft:redstone_wire[north=side,south=side]",
        )
        .unwrap();
        s.set_block_from_string(
            0,
            1,
            3,
            "minecraft:redstone_wire[north=side,south=side]",
        )
        .unwrap();

        // Concrete blocks the torches sit on top of.
        s.set_block_from_string(0, 1, 0, "minecraft:white_concrete").unwrap();
        s.set_block_from_string(0, 1, 4, "minecraft:white_concrete").unwrap();

        // Both standing torches initially LIT.
        s.set_block_from_string(0, 2, 0, "minecraft:redstone_torch[lit=true]")
            .unwrap();
        s.set_block_from_string(0, 2, 4, "minecraft:redstone_torch[lit=true]")
            .unwrap();

        s
    }

    #[test]
    fn symmetric_torches_must_match_after_lever_toggle() {
        let schematic = build_circuit();
        let mut world = MchprsWorld::new(schematic).expect("build sim world");

        // Toggle the lever ON and let things settle.
        world.on_use_block(nucleation::simulation::BlockPos::new(-1, 1, 2));
        world.tick(20);
        world.flush();
        world.sync_to_schematic();

        let synced = world.get_schematic();
        let south_torch = synced
            .get_block(0, 2, 0)
            .expect("south torch")
            .get_property("lit")
            .map(|s| s.to_string())
            .unwrap_or_default();
        let north_torch = synced
            .get_block(0, 2, 4)
            .expect("north torch")
            .get_property("lit")
            .map(|s| s.to_string())
            .unwrap_or_default();

        let south_wire = world.get_redstone_power(nucleation::simulation::BlockPos::new(0, 1, 1));
        let north_wire = world.get_redstone_power(nucleation::simulation::BlockPos::new(0, 1, 3));

        eprintln!(
            "south wire (0,1,1)={}  torch (0,2,0).lit={:?}",
            south_wire, south_torch
        );
        eprintln!(
            "north wire (0,1,3)={}  torch (0,2,4).lit={:?}",
            north_wire, north_torch
        );

        assert_eq!(
            south_wire, north_wire,
            "wire signals at (0,1,1) and (0,1,3) should match (mirror image)"
        );
        assert_eq!(
            south_torch, north_torch,
            "torches at (0,2,0) and (0,2,4) should be in the same lit state"
        );
    }

    /// User-reported case: a wire with explicit `east=side, west=side`
    /// (no N/S connections) sits next to a solid block to its north.
    /// In vanilla Minecraft the wire is pointing east/west only, so it
    /// does NOT weakly power the northern neighbour — and a repeater
    /// reading from that block via its rear should stay unpowered.
    #[test]
    fn wire_does_not_power_block_outside_its_visual_direction() {
        let mut s = UniversalSchematic::new("wire-direction".to_string());

        s.set_block_from_string(0, 0, 0, "minecraft:lime_concrete").unwrap();
        s.set_block_from_string(
            0,
            1,
            0,
            "minecraft:lever[face=floor,facing=south,powered=false]",
        )
        .unwrap();
        s.set_block_from_string(1, 0, 0, "minecraft:lime_concrete").unwrap();
        s.set_block_from_string(
            1,
            1,
            0,
            "minecraft:redstone_wire[east=side,west=side]",
        )
        .unwrap();
        s.set_block_from_string(1, 1, 1, "minecraft:lime_concrete").unwrap();
        s.set_block_from_string(1, 0, 2, "minecraft:lime_concrete").unwrap();
        s.set_block_from_string(
            1,
            1,
            2,
            "minecraft:repeater[facing=north,delay=1,powered=false]",
        )
        .unwrap();

        let mut world = MchprsWorld::new(s).expect("build sim world");
        world.on_use_block(nucleation::simulation::BlockPos::new(0, 1, 0));
        world.tick(4);
        world.flush();
        world.sync_to_schematic();

        let synced = world.get_schematic();
        let powered = synced
            .get_block(1, 1, 2)
            .expect("repeater")
            .get_property("powered")
            .map(|s| s.to_string())
            .unwrap_or_default();

        assert_eq!(
            powered, "false",
            "repeater should NOT be powered (wire only points E/W, the block at \
             (1,1,1) sits north of the wire and shouldn't receive weak power); \
             got powered={:?}",
            powered
        );
    }

    /// Companion regression for the orientation half of the same Coalesce
    /// bug: two mirror-image redstone repeaters fed by a shared powered
    /// block must each keep their own `facing` after the simulation runs.
    /// Without preserving the alias's own block on flush, the survivor's
    /// `facing` would clobber the other repeater's orientation.
    #[test]
    fn symmetric_repeaters_keep_facing_after_lever_toggle() {
        let mut s = UniversalSchematic::new("repeater-facing".to_string());

        for z in 0..5 {
            s.set_block_from_string(0, 0, z, "minecraft:stone_bricks")
                .unwrap();
        }
        s.set_block_from_string(0, 1, 2, "minecraft:stone_bricks").unwrap();
        s.set_block_from_string(
            -1,
            1,
            2,
            "minecraft:lever[face=wall,facing=west,powered=false]",
        )
        .unwrap();

        // Mirror-image repeaters; their `facing` points at the host
        // (the input side in MCHPRS convention).
        s.set_block_from_string(
            0,
            1,
            1,
            "minecraft:repeater[facing=south,delay=1,powered=false]",
        )
        .unwrap();
        s.set_block_from_string(
            0,
            1,
            3,
            "minecraft:repeater[facing=north,delay=1,powered=false]",
        )
        .unwrap();

        let mut world = MchprsWorld::new(s).expect("build sim world");
        world.on_use_block(nucleation::simulation::BlockPos::new(-1, 1, 2));
        world.tick(4);
        world.flush();
        world.sync_to_schematic();

        let synced = world.get_schematic();
        let north_facing = synced
            .get_block(0, 1, 1)
            .expect("north repeater")
            .get_property("facing")
            .map(|s| s.to_string())
            .unwrap_or_default();
        let south_facing = synced
            .get_block(0, 1, 3)
            .expect("south repeater")
            .get_property("facing")
            .map(|s| s.to_string())
            .unwrap_or_default();

        assert_eq!(
            north_facing, "south",
            "north repeater (z=1) should still face south; got {:?}",
            north_facing
        );
        assert_eq!(
            south_facing, "north",
            "south repeater (z=3) should still face north; got {:?}",
            south_facing
        );
    }
}
