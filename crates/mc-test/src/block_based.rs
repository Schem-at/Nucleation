//! Synthesize a scenario from a `minecraft:block_based` gametest structure.
//!
//! Modern gametest suites (lithium's, vanilla's own since 1.21.5) carry their
//! assertions *inside* the structure: a `test_block` in `start` mode pulses
//! redstone when the test begins, and the test passes when every `accept`
//! test_block is powered before the run's tick budget ends. That is a
//! machine-readable claim, so nobody should have to re-author it by hand.
//!
//! The synthesized descriptor leans on the engine's own `TestAccept`
//! behaviour (an accept-mode test_block latches to `fired=true` on its first
//! neighbour signal, recorded as a block change):
//!
//! - each `start` block becomes a `redstone_block` at tick 0 and stays one —
//!   the wiki-documented behaviour ("emits a constant redstone signal,
//!   similarly to a block of redstone, when the test starts"),
//! - each `accept` block must latch to `fired=true` at least once — an
//!   [`events`](crate::Case::events) claim on the accept cell itself, so a
//!   transient counts no matter which tick it lands on,
//! - a run-length check pins the horizon at `max_ticks`.

use mc_tick::{Pos, Structure};

/// Synthesize a single-case suite from `structure`'s test blocks.
///
/// `None` when the structure carries no test blocks: a spec with no
/// expressible claim would pass by asserting nothing, and a vacuous green is
/// worse than an unported row. `what` labels the case; `max_ticks` is the
/// tick budget from the corpus's `test_instance` JSON; `extra_inert` carries
/// blocks a probe found unmodelled.
///
/// `shift` is the translation the *carrier* will apply to the build — a
/// litematic compacts to its non-air bounding box — so every emitted
/// position is pre-shifted, and the case's `origin` records where the build
/// originally sat. Same-tick update order hashes absolute positions, and a
/// build shifted by one block phase-shifts its observer chains; `origin` is
/// the designed compensation.
pub fn synthesize_block_based(
    structure: &Structure,
    what: &str,
    max_ticks: u64,
    extra_inert: &[String],
    shift: (i32, i32, i32),
) -> Option<String> {
    let mut starts: Vec<Pos> = Vec::new();
    let mut accepts: Vec<Pos> = Vec::new();
    for (pos, entry) in &structure.blocks {
        let descriptor = structure.palette.get(*entry).map(String::as_str).unwrap_or_default();
        if !descriptor.starts_with("minecraft:test_block") {
            continue;
        }
        if descriptor.contains("mode=start") {
            starts.push(*pos);
        } else if descriptor.contains("mode=accept") {
            accepts.push(*pos);
        }
        // `log` and `fail` modes assert nothing a synthesized pass needs.
    }
    if starts.is_empty() && accepts.is_empty() {
        return None;
    }

    // A start test_block "emits a constant redstone signal, similarly to a
    // block of redstone, when the test starts" (minecraft.wiki/w/Test_Block)
    // — constant, not a pulse. Becoming a redstone block at tick 0 and
    // staying one IS the vanilla behaviour; lithium's item_sorter depends on
    // it (its gated input hopper drains the feed chest for the whole run).
    let mut actions = Vec::new();
    for start in &starts {
        actions.push(serde_json::json!({
            "tick": 0,
            "place": [start.x - shift.0, start.y - shift.1, start.z - shift.2],
            "state": "minecraft:redstone_block"
        }));
    }

    // The vanilla pass condition, headless: every accept block must latch to
    // the engine's `fired=true` variant (the `TestAccept` behaviour records
    // it as a block change). No `after` guard: anything the emulated pulse
    // can power, the game's own start pulse powers too, so a latch during
    // the pulse window is vanilla behaviour rather than a leak.
    let mut events = Vec::new();
    for accept in &accepts {
        events.push(serde_json::json!({
            "kind": "block-changed",
            "pos": [accept.x - shift.0, accept.y - shift.1, accept.z - shift.2],
            "to": "minecraft:test_block[fired=true]",
        }));
    }
    if events.is_empty() {
        // Start blocks but nothing to accept: the only claim would be the
        // pulse's own block changes, and a spec that cannot fail is refused.
        return None;
    }

    let inert: Vec<String> = extra_inert.to_vec();

    let suite = serde_json::json!({
        "name": format!(
            "{what}: block-based auto-port — {} start pulse(s) emulated, {} accept(s) must latch",
            starts.len(),
            events.len(),
        ),
        "origin": [shift.0, shift.1, shift.2],
        "seed": 0,
        // The lithium test environment: random_tick_speed 3.
        "random_ticks": 3,
        "inert": inert,
        // Vanilla runs 10 setup ticks before a block-based test starts;
        // placement transients (observer pulses) play out off the record.
        "setup": 10,
        "actions": actions,
        "checks": [
            // A run-length pin only: `at_least: 0` never fails, it just makes
            // the case tick to the corpus's budget. The events are the claim.
            { "tick": max_ticks, "expect": "changes", "at_least": 0 }
        ],
        "events": events,
    });
    Some(suite.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// start → dust → accept. The synthesized case must pulse the start cell
    /// and demand the accept latches to `fired=true`.
    const RIG: &str = r#"{DataVersion: 4325, size: [4, 1, 1], entities: [], blocks: [{pos: [0, 0, 0], state: 0}, {pos: [1, 0, 0], state: 1}, {pos: [2, 0, 0], state: 2}], palette: [{Name: "minecraft:test_block", Properties: {mode: "start"}}, {Name: "minecraft:redstone_wire", Properties: {east: "side", north: "none", power: "0", south: "none", west: "side"}}, {Name: "minecraft:test_block", Properties: {mode: "accept"}}]}"#;

    #[test]
    fn a_block_based_rig_synthesizes_a_runnable_case() {
        let structure = Structure::parse(RIG).expect("parses");
        let spec = synthesize_block_based(&structure, "rig", 20, &["minecraft:sponge".to_string()], (0, 0, 0))
            .expect("test blocks found");
        let cases = crate::parse_suite(&spec, "rig").expect("the synthesized spec must parse");
        assert_eq!(cases.len(), 1);
        let case = &cases[0];
        assert_eq!(case.inert, vec!["minecraft:sponge"], "test_block needs no assertion now");
        assert_eq!(case.setup, 10, "vanilla's setup_ticks, honoured");
        assert_eq!(case.actions.len(), 1, "one start: a constant signal, never cleared");
        assert_eq!(case.events.len(), 1, "one accept, one latch claim");
        assert_eq!(case.events[0].pos, Some([2, 0, 0]), "the claim sits on the accept itself");
        assert_eq!(case.events[0].to.as_deref(), Some("minecraft:test_block[fired=true]"));
        assert_eq!(case.events[0].after, None, "a latch during the pulse window is vanilla");
        assert_eq!(case.checks[0].tick, 20);
    }

    #[test]
    fn a_plain_structure_synthesizes_nothing() {
        let plain = r#"{DataVersion: 4325, size: [1, 1, 1], entities: [], blocks: [{pos: [0, 0, 0], state: 0}], palette: [{Name: "minecraft:stone"}]}"#;
        let structure = Structure::parse(plain).expect("parses");
        assert!(synthesize_block_based(&structure, "plain", 20, &[], (0, 0, 0)).is_none());
    }

    /// The rig's synthesized spec is not just well-formed: run, it passes —
    /// the pulse crosses the dust and the accept really latches.
    #[test]
    fn the_synthesized_rig_actually_passes_its_own_run() {
        let structure = Structure::parse(RIG).expect("parses");
        let spec =
            synthesize_block_based(&structure, "rig", 20, &[], (0, 0, 0)).expect("test blocks found");
        let cases = crate::parse_suite(&spec, "rig").expect("parses");
        crate::run(&structure, &cases[0], None).expect("the synthesized claim must hold");
    }

    /// A start with no accepts: the only claim would be the pulse's own
    /// changes, and a spec that cannot fail is refused.
    #[test]
    fn a_start_with_no_accepts_synthesizes_nothing() {
        let rig = r#"{DataVersion: 4325, size: [2, 1, 1], entities: [], blocks: [{pos: [0, 0, 0], state: 0}], palette: [{Name: "minecraft:test_block", Properties: {mode: "start"}}]}"#;
        let structure = Structure::parse(rig).expect("parses");
        assert!(synthesize_block_based(&structure, "rig", 20, &[], (0, 0, 0)).is_none());
    }
}
