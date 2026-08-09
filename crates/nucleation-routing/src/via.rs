//! Vertical via templates.
//!
//! New vertical primitives (droppers, observers, ...) should be a template
//! registration, not a router fork. The registry is seeded with the one
//! verified template: the 1x1 torch ladder (`probe_vert.py`): 2 blocks per
//! torch, inverts per torch (so an even count is non-inverting), the cap
//! block is strongly powered and exits a fresh 15, and two ladders 2 apart
//! do not crosstalk.

/// A registered vertical via.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ViaTemplate {
    /// Template name.
    pub name: String,
    /// Torch count. Rise is `2 * torches + 1` (exit dust sits on the cap).
    pub torches: u32,
    /// Horizontal reach: the exit lands this many cells ahead of the
    /// current path cell (the entry dust takes the first of them).
    pub reach: i32,
    /// Search cost of taking this via.
    pub cost: u32,
    /// Whether the signal is inverted end to end (odd torch count).
    pub inverting: bool,
    /// Entry contract: the base needs STRAIGHT dead-end dust pointing in.
    /// The climb lays its OWN entry cell one step ahead: a fresh dead-end
    /// dust with a single neighbour behind it is always a straight line
    /// into the base. Using the current path cell does NOT work — its
    /// shape may run perpendicular, and dust only powers blocks it points
    /// into (the dead-ladder-climbs bug).
    pub needs_dead_end_entry: bool,
    /// Signal strength at the exit (the ladder cap is strongly powered:
    /// a fresh 15, which is what makes ladder climbs decay-safe).
    pub output_strength: u8,
}

impl ViaTemplate {
    /// Total rise in y from the path cell to the exit dust.
    pub fn rise(&self) -> i32 {
        2 * self.torches as i32 + 1
    }

    /// Height of the column (base block through cap) that must be free
    /// before emission: y .. y + `column_top()` inclusive.
    pub fn column_top(&self) -> i32 {
        2 * self.torches as i32
    }

    /// The verified 1x1 torch ladder: 2 torches, +5 rise, cost 9,
    /// non-inverting, fresh-15 cap.
    pub fn torch_ladder() -> Self {
        ViaTemplate {
            name: "torch_ladder".to_string(),
            torches: 2,
            reach: 2,
            cost: 9,
            inverting: false,
            needs_dead_end_entry: true,
            output_strength: 15,
        }
    }
}

/// The via registry consulted by the fabric's move generator.
#[derive(Clone, Debug)]
pub struct ViaRegistry {
    templates: Vec<ViaTemplate>,
}

impl Default for ViaRegistry {
    fn default() -> Self {
        ViaRegistry {
            templates: vec![ViaTemplate::torch_ladder()],
        }
    }
}

impl ViaRegistry {
    /// An empty registry (no vertical moves).
    pub fn empty() -> Self {
        ViaRegistry {
            templates: Vec::new(),
        }
    }

    /// Register a template, returning its id.
    pub fn register(&mut self, t: ViaTemplate) -> usize {
        self.templates.push(t);
        self.templates.len() - 1
    }

    /// All templates, id-ordered.
    pub fn templates(&self) -> &[ViaTemplate] {
        &self.templates
    }

    /// Template by id.
    pub fn get(&self, id: usize) -> &ViaTemplate {
        &self.templates[id]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn torch_ladder_matches_probe_vert() {
        // probe_vert.py: 2 torches climb +4 of column, exit dust at
        // y0 + 2*torches + 1; even torch count is non-inverting; cap exits
        // fresh 15.
        let t = ViaTemplate::torch_ladder();
        assert_eq!(t.rise(), 5);
        assert_eq!(t.column_top(), 4);
        assert!(!t.inverting);
        assert_eq!(t.output_strength, 15);
        assert!(t.needs_dead_end_entry);
        let reg = ViaRegistry::default();
        assert_eq!(reg.templates().len(), 1);
        assert_eq!(reg.get(0).name, "torch_ladder");
    }
}
