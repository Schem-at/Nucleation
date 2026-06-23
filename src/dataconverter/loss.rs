//! Conversion context (direction + data-loss reporting) for the reverse engine.
//!
//! Forward conversion is faithful and lossless by construction (it mirrors the
//! Java engine). **Reverse** conversion (new -> old), used to save a schematic
//! for an older Minecraft version, must sometimes approximate: the 1.13
//! Flattening, the 1.20.5 component squash, item `Damage` flattening and a few
//! entity-id merges are many-to-one, so their inverses are relations, not
//! functions. Whenever a reverse converter cannot perfectly restore the older
//! shape it records a [`LossEntry`]; the accumulated [`LossReport`] is returned
//! to the caller (the developer) and surfaced across the WASM boundary (the
//! tool's user). Conversion is **never silent** about loss.
//!
//! Direction and the active loss collector live in a thread-local so that
//! neither the converter/walker signatures nor the 176 version files need to
//! change: the shared `&'static Registry` stays immutable and `Send + Sync`,
//! while each thread carries its own direction + report. `DataType::convert`
//! reads the direction and dispatches forward/reverse accordingly.

use std::cell::RefCell;

/// Which way the engine is walking the version chain.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Direction {
    /// old -> new (faithful port of the Java engine).
    Forward,
    /// new -> old (inverse converters, best-effort for lossy buckets).
    Reverse,
}

/// How severe a reverse-conversion approximation is.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Severity {
    /// A field was dropped or approximated and the original value is *not*
    /// recoverable — genuine data loss the user should see.
    Loss,
    /// A best-effort substitution was applied that is usually correct (e.g. a
    /// default-filled flattened block reverts to its variant-0 form). Worth
    /// surfacing but not strictly "lost".
    Approximated,
}

/// A machine-readable category for a loss, so the UI can group/aggregate.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LossKind {
    /// 1.13 block flattening: a flattened state had no unique pre-flattening
    /// `(id, meta)` preimage (default-filled), so the oldest/variant-0 meta was
    /// chosen. (C1)
    FlatteningAmbiguous,
    /// 1.13 block flattening: the flattened block name is unknown to the inverse
    /// table; the block could not be downgraded and was left as-is. (C1)
    FlatteningUnknownBlock,
    /// 1.13 item flattening: `Damage`-based durability could not be restored for
    /// a non-whitelisted item, or the flat item id has no numeric preimage. (C3)
    ItemFlatteningDamage,
    /// 1.20.5 components: a component had no legacy `tag` representation and was
    /// dropped (it survives only via `minecraft:custom_data` when present). (C2)
    ComponentDropped,
    /// An entity/item id merge (e.g. skeleton variants) could not be
    /// disambiguated when reversing. (C4)
    EntityMergeAmbiguous,
    /// A many-to-one rename was reversed by picking a canonical preimage. (A')
    RenameAmbiguous,
    /// A derived fingerprint (feature -> string id, generator settings) cannot be
    /// expanded back to its full source. (C5)
    FingerprintCollapse,
    /// The target version predates a structure entirely; it was dropped because
    /// the older game could not represent it.
    UnsupportedInTarget,
    /// A catch-all for a documented best-effort inverse not covered above.
    Other,
}

impl LossKind {
    /// A stable machine-readable tag for serialization / UI grouping.
    pub fn as_str(self) -> &'static str {
        match self {
            LossKind::FlatteningAmbiguous => "flattening_ambiguous",
            LossKind::FlatteningUnknownBlock => "flattening_unknown_block",
            LossKind::ItemFlatteningDamage => "item_flattening_damage",
            LossKind::ComponentDropped => "component_dropped",
            LossKind::EntityMergeAmbiguous => "entity_merge_ambiguous",
            LossKind::RenameAmbiguous => "rename_ambiguous",
            LossKind::FingerprintCollapse => "fingerprint_collapse",
            LossKind::UnsupportedInTarget => "unsupported_in_target",
            LossKind::Other => "other",
        }
    }
}

/// One recorded data-loss / approximation event during a reverse conversion.
#[derive(Clone, Debug)]
pub struct LossEntry {
    /// The data version boundary at which the loss occurred (the version of the
    /// converter being inverted).
    pub version: i32,
    pub kind: LossKind,
    pub severity: Severity,
    /// A human path to the affected datum, e.g.
    /// `block_entity minecraft:chest @ (1,2,3) > Items[0]`.
    pub path: String,
    /// A human-readable explanation of what was lost or approximated.
    pub detail: String,
}

/// The accumulated set of [`LossEntry`]s from one reverse conversion.
#[derive(Clone, Debug, Default)]
pub struct LossReport {
    pub entries: Vec<LossEntry>,
}

impl LossReport {
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    /// Number of entries that represent genuine (unrecoverable) data loss.
    pub fn loss_count(&self) -> usize {
        self.entries
            .iter()
            .filter(|e| e.severity == Severity::Loss)
            .count()
    }
    /// Serialize to a JSON array of `{version, kind, severity, path, detail}`
    /// objects, for surfacing across the WASM boundary to the tool's user.
    pub fn to_json(&self) -> String {
        let entries: Vec<serde_json::Value> = self
            .entries
            .iter()
            .map(|e| {
                serde_json::json!({
                    "version": e.version,
                    "kind": e.kind.as_str(),
                    "severity": match e.severity {
                        Severity::Loss => "loss",
                        Severity::Approximated => "approximated",
                    },
                    "path": e.path,
                    "detail": e.detail,
                })
            })
            .collect();
        serde_json::to_string(&entries).unwrap_or_else(|_| "[]".to_string())
    }

    /// A compact multi-line summary suitable for a developer log or a UI warning
    /// panel. Empty string when nothing was lost.
    pub fn summary(&self) -> String {
        if self.entries.is_empty() {
            return String::new();
        }
        let mut out = format!(
            "{} conversion issue(s) ({} data loss):\n",
            self.entries.len(),
            self.loss_count()
        );
        for e in &self.entries {
            let tag = match e.severity {
                Severity::Loss => "LOSS",
                Severity::Approximated => "APPROX",
            };
            out.push_str(&format!(
                "  [{tag}] v{} {} — {}\n",
                e.version, e.path, e.detail
            ));
        }
        out
    }
}

struct Ctx {
    direction: Direction,
    /// Whether a reverse session is collecting losses (outermost reverse call).
    collecting: bool,
    /// The path stack from the conversion root to the current node.
    path: Vec<String>,
    losses: Vec<LossEntry>,
}

impl Default for Ctx {
    fn default() -> Self {
        Self {
            direction: Direction::Forward,
            collecting: false,
            path: Vec::new(),
            losses: Vec::new(),
        }
    }
}

thread_local! {
    static CTX: RefCell<Ctx> = RefCell::new(Ctx::default());
}

/// The direction the current thread is converting in.
#[inline]
pub fn direction() -> Direction {
    CTX.with(|c| c.borrow().direction)
}

/// Convenience: are we converting new -> old?
#[inline]
pub fn is_reverse() -> bool {
    direction() == Direction::Reverse
}

/// RAII guard that pushes a path segment for the duration of a descent, so loss
/// entries carry an accurate location. Popping is automatic on drop, even if a
/// converter unwinds. Cheap (a single `Vec` push/pop) and active in both
/// directions; only [`report_loss`] reads the path.
pub struct PathGuard {
    _priv: (),
}

#[inline]
pub fn path_scope(segment: impl Into<String>) -> PathGuard {
    CTX.with(|c| c.borrow_mut().path.push(segment.into()));
    PathGuard { _priv: () }
}

impl Drop for PathGuard {
    fn drop(&mut self) {
        CTX.with(|c| {
            c.borrow_mut().path.pop();
        });
    }
}

/// The current path, joined for display (`"a > b[0] > c"`).
pub fn current_path() -> String {
    CTX.with(|c| c.borrow().path.join(" > "))
}

/// Record a data-loss / approximation event. A no-op unless a reverse session is
/// collecting (so forward conversion and stray calls cost nothing).
pub fn report_loss(version: i32, kind: LossKind, severity: Severity, detail: impl Into<String>) {
    CTX.with(|c| {
        let mut c = c.borrow_mut();
        if !c.collecting {
            return;
        }
        let path = c.path.join(" > ");
        c.losses.push(LossEntry {
            version,
            kind,
            severity,
            path,
            detail: detail.into(),
        });
    });
}

/// Guard that restores the previous direction/collecting state on drop, so a
/// panicking converter can't leave the thread stuck in reverse mode.
struct SessionGuard {
    prev_direction: Direction,
    prev_collecting: bool,
    /// True when this guard started the outermost session (and so owns draining).
    outermost: bool,
}

impl Drop for SessionGuard {
    fn drop(&mut self) {
        CTX.with(|c| {
            let mut c = c.borrow_mut();
            c.direction = self.prev_direction;
            c.collecting = self.prev_collecting;
            if self.outermost {
                c.path.clear();
            }
        });
    }
}

/// Run `f` with the thread switched to reverse mode, returning its result plus
/// the [`LossReport`] accumulated during the (outermost) reverse session.
///
/// Nesting is safe: an inner `run_reverse` (e.g. a per-type reverse entry point
/// invoked from within a whole-schematic reverse) accumulates into the outer
/// report and returns an empty report of its own. Only the outermost call
/// resets and drains the collector.
pub fn run_reverse<R>(f: impl FnOnce() -> R) -> (R, LossReport) {
    let outermost = CTX.with(|c| {
        let mut c = c.borrow_mut();
        let was_collecting = c.collecting;
        if !was_collecting {
            c.path.clear();
            c.losses.clear();
        }
        c.direction = Direction::Reverse;
        !was_collecting
    });
    let _guard = {
        // Capture previous state for restoration. (Previous direction is whatever
        // it was before this call; previous collecting mirrors `!outermost`.)
        SessionGuard {
            prev_direction: if outermost {
                Direction::Forward
            } else {
                Direction::Reverse
            },
            prev_collecting: !outermost,
            outermost,
        }
    };
    CTX.with(|c| c.borrow_mut().collecting = true);

    let result = f();

    let report = if outermost {
        CTX.with(|c| LossReport {
            entries: std::mem::take(&mut c.borrow_mut().losses),
        })
    } else {
        LossReport::default()
    };
    (result, report)
}
