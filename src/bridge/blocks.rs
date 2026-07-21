//! Block metadata queries: per-block facts, tags, kinds, shape variants and
//! block-state enumeration over the built-in block table. Bridge-native
//! surface (no old `ffi/*.rs` counterpart) fronting
//! [`crate::blockpedia::facts_json`].

#[diplomat::bridge]
pub mod ffi {
    use super::super::shared::ffi::NucleationError;
    use diplomat_runtime::DiplomatWrite;
    use std::fmt::Write;

    /// Namespace for read-only queries over the built-in block table
    /// (Java 26.2 blocks + official semantics extracted from the vanilla
    /// jars: definition kinds, base-block links, block tags, model
    /// geometry). All list results are JSON array strings sorted by id;
    /// block/tag/kind arguments accept both `minecraft:`-prefixed and
    /// short forms (`minecraft:oak_stairs` / `oak_stairs`).
    #[diplomat::opaque]
    pub struct Blocks;

    impl Blocks {
        /// Full facts for one block as a JSON object:
        /// `{id, kind, base_block, tags: [...], full_cube, transparent,
        /// color: [r, g, b] | null, properties: {name: [values...]},
        /// default_state: {name: value}}`. `kind` is the official
        /// definition kind (`minecraft:stair`, plain full blocks are
        /// `minecraft:block`); `base_block` is the block this one is a
        /// shape variant of (or `null`); `color` is the texture-derived
        /// average RGB. Errors with `NotFound` for unknown ids.
        pub fn get_json(id: &DiplomatStr, out: &mut DiplomatWrite) -> Result<(), NucleationError> {
            let id = std::str::from_utf8(id).map_err(|_| NucleationError::InvalidArgument)?;
            let json = crate::blockpedia::facts_json::block_facts_json(id)
                .ok_or(NucleationError::NotFound)?;
            let _ = write!(out, "{json}");
            Ok(())
        }

        /// All known block ids as a sorted JSON array string.
        pub fn ids_json(out: &mut DiplomatWrite) {
            let _ = write!(
                out,
                "{}",
                crate::blockpedia::facts_json::all_block_ids_json()
            );
        }

        /// Ids of every block carrying the vanilla block tag, as a sorted
        /// Blocks whose measured texture color is within `max_distance`
        /// (Oklab; ~0.05 = same color family, ~0.15 = generous) of the given
        /// RGB, as a JSON array of `{"id", "color": [r,g,b], "distance"}`
        /// sorted nearest-first. Blocks without color data never match.
        pub fn by_color_json(
            r: u8,
            g: u8,
            b: u8,
            max_distance: f32,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            let target = crate::blockpedia::ExtendedColorData::from_rgb(r, g, b);
            let mut hits: Vec<(f32, &str, [u8; 3])> = crate::blockpedia::all_blocks()
                .filter_map(|f| {
                    let c = f.extras.color.as_ref()?.to_extended();
                    let d = c.distance_oklab(&target);
                    (d <= max_distance).then_some((d, f.id, c.rgb))
                })
                .collect();
            hits.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
            let rows: Vec<serde_json::Value> = hits
                .into_iter()
                .map(|(d, id, rgb)| {
                    serde_json::json!({"id": id, "color": rgb, "distance": (d * 1000.0).round() / 1000.0})
                })
                .collect();
            let _ = write!(out, "{}", serde_json::to_string(&rows).unwrap_or_default());
            Ok(())
        }

        /// JSON array string (`[]` for unknown tags). Accepts
        /// `minecraft:wool` and short `wool` forms, including nested paths
        /// like `mineable/pickaxe`.
        pub fn by_tag_json(
            tag: &DiplomatStr,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            let tag = std::str::from_utf8(tag).map_err(|_| NucleationError::InvalidArgument)?;
            let _ = write!(
                out,
                "{}",
                crate::blockpedia::facts_json::block_ids_by_tag_json(tag)
            );
            Ok(())
        }

        /// Ids of every block of the given official definition kind
        /// (`minecraft:stair`, `minecraft:slab`, `minecraft:door`, ...), as
        /// a sorted JSON array string (`[]` for unknown kinds).
        pub fn by_kind_json(
            kind: &DiplomatStr,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            let kind = std::str::from_utf8(kind).map_err(|_| NucleationError::InvalidArgument)?;
            let _ = write!(
                out,
                "{}",
                crate::blockpedia::facts_json::block_ids_by_kind_json(kind)
            );
            Ok(())
        }

        /// The base block followed by all its shape variants — blocks whose
        /// `base_block` is `base_id` (stairs, slabs, walls, fences of the
        /// base) — as a JSON array string. The base itself is always first;
        /// variants follow sorted by id. Errors with `NotFound` for unknown
        /// base ids.
        pub fn variants_of_json(
            base_id: &DiplomatStr,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            let base_id =
                std::str::from_utf8(base_id).map_err(|_| NucleationError::InvalidArgument)?;
            let json = crate::blockpedia::facts_json::variants_of_ids_json(base_id)
                .ok_or(NucleationError::NotFound)?;
            let _ = write!(out, "{json}");
            Ok(())
        }

        /// All known vanilla block tag names as a sorted JSON array string
        /// (`minecraft:`-prefixed, e.g. `minecraft:wool`).
        pub fn tags_json(out: &mut DiplomatWrite) {
            let _ = write!(out, "{}", crate::blockpedia::facts_json::all_tags_json());
        }

        /// Every property-value combination of the block as a JSON array of
        /// `{prop: value}` objects (a single `{}` entry for property-less
        /// blocks). Errors with `NotFound` for unknown ids and with
        /// `InvalidArgument` if the combination count exceeds 4096 (guard
        /// against pathological output; the current data tops out at 1350
        /// for `minecraft:note_block`).
        pub fn states_json(
            id: &DiplomatStr,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            let id = std::str::from_utf8(id).map_err(|_| NucleationError::InvalidArgument)?;
            let json =
                crate::blockpedia::facts_json::block_states_json(id).map_err(|e| match e {
                    crate::blockpedia::BlockpediaError::Block(
                        crate::blockpedia::errors::BlockError::NotFound(_),
                    ) => NucleationError::NotFound,
                    _ => NucleationError::InvalidArgument,
                })?;
            let _ = write!(out, "{json}");
            Ok(())
        }

        /// Total number of blocks in the table.
        pub fn count() -> usize {
            crate::blockpedia::facts_json::block_count()
        }
    }
}
