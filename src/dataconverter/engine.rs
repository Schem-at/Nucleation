//! The conversion engine: the `MCValueType` / `MCDataType` / `IDDataType` type
//! system and the breakpoint-segmented version walk.
//!
//! Ported from `datatypes/MCValueType.java`, `datatypes/MCDataType.java`,
//! `datatypes/IDDataType.java`, and `MCDataConverter.java`. Unlike Java, which
//! threads a possibly-replaced root (`ret = data = replace`), our converters
//! and walkers mutate the NBT in place through `&mut`; a converter that wants
//! to replace the whole map writes `*data = new_map`. The registry itself is
//! immutable at convert time — only the NBT data is mutated.

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::sync::Arc;

use crate::nbt::{NbtMap, NbtValue};

use super::loss::{self, Direction};
use super::registry::Registry;
use super::types::MapExt;
use super::version::{encode_versions, EncodedVersion, MAX_STEP, V99};

/// A structure converter: mutates the compound in place for one version step.
pub type Converter = Box<dyn Fn(&mut NbtMap, EncodedVersion, EncodedVersion) + Send + Sync>;
/// A value-type converter: mutates a scalar value (usually a string id).
pub type ValueConverter = Box<dyn Fn(&mut NbtValue, EncodedVersion, EncodedVersion) + Send + Sync>;
/// A walker: recurses into nested typed sub-structures by calling other types'
/// `convert`. Stored as `Arc` so `copy_walkers` (id renames) can share it.
pub type Walker = Arc<dyn Fn(&Registry, &mut NbtMap, EncodedVersion, EncodedVersion) + Send + Sync>;

type HookFn = Box<dyn Fn(&mut NbtMap, EncodedVersion, EncodedVersion) + Send + Sync>;
type ValueHookFn = Box<dyn Fn(&mut NbtValue, EncodedVersion, EncodedVersion) + Send + Sync>;

/// A pre/post hook wrapping the converter and walker phases of an [`MCDataType`].
pub struct Hook {
    pub pre: Option<HookFn>,
    pub post: Option<HookFn>,
}

/// A pre/post hook for an [`MCValueType`].
pub struct ValueHook {
    pub pre: Option<ValueHookFn>,
    pub post: Option<ValueHookFn>,
}

#[inline]
fn floor<'a, V>(map: &'a BTreeMap<EncodedVersion, V>, version: EncodedVersion) -> Option<&'a V> {
    map.range(..=version).next_back().map(|(_, v)| v)
}

// ---------------------------------------------------------------------------
// MCValueType — scalar leaf types (block name, item name, entity name, …).
// Converters only, no walkers (MCValueType.java).
// ---------------------------------------------------------------------------

pub struct MCValueType {
    pub name: &'static str,
    converters: Vec<(EncodedVersion, ValueConverter)>,
    /// Inverse value converters (new -> old), e.g. inverted rename tables. Run
    /// in descending version order by [`MCValueType::convert_reverse`].
    reverse_converters: Vec<(EncodedVersion, ValueConverter)>,
    hooks: BTreeMap<EncodedVersion, Vec<ValueHook>>,
}

impl MCValueType {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            converters: Vec::new(),
            reverse_converters: Vec::new(),
            hooks: BTreeMap::new(),
        }
    }

    pub fn add_converter(&mut self, version: i32, step: i32, converter: ValueConverter) {
        self.converters
            .push((encode_versions(version, step), converter));
    }

    /// Register the inverse of a value converter for reverse (new -> old) walks.
    pub fn add_reverse_converter(&mut self, version: i32, step: i32, converter: ValueConverter) {
        self.reverse_converters
            .push((encode_versions(version, step), converter));
    }

    pub fn add_structure_hook(&mut self, version: i32, step: i32, hook: ValueHook) {
        self.hooks
            .entry(encode_versions(version, step))
            .or_default()
            .push(hook);
    }

    /// Sort converters into ascending encoded-version order (stable, so equal
    /// versions keep registration order) — called once after registration.
    pub fn finalize(&mut self) {
        self.converters.sort_by_key(|(v, _)| *v);
        self.reverse_converters.sort_by_key(|(v, _)| *v);
    }

    /// Dispatch on the thread-local direction (see [`super::loss`]).
    pub fn convert(&self, data: &mut NbtValue, from: EncodedVersion, to: EncodedVersion) {
        match loss::direction() {
            Direction::Forward => self.convert_forward(data, from, to),
            Direction::Reverse => self.convert_reverse(data, from, to),
        }
    }

    fn convert_forward(&self, data: &mut NbtValue, from: EncodedVersion, to: EncodedVersion) {
        for (cv, converter) in &self.converters {
            let cv = *cv;
            if cv <= from {
                continue;
            }
            if cv > to {
                break;
            }
            if let Some(hooks) = floor(&self.hooks, cv) {
                for h in hooks {
                    if let Some(pre) = &h.pre {
                        pre(data, from, to);
                    }
                }
            }
            converter(data, from, to);
            if let Some(hooks) = floor(&self.hooks, to) {
                for h in hooks {
                    if let Some(post) = &h.post {
                        post(data, from, to);
                    }
                }
            }
        }
    }

    /// `from` is the newer version, `to` the older; undo converters over the
    /// window `(to, from]` in descending order.
    fn convert_reverse(&self, data: &mut NbtValue, from: EncodedVersion, to: EncodedVersion) {
        for (cv, converter) in self.reverse_converters.iter().rev() {
            let cv = *cv;
            if cv > from {
                continue;
            }
            if cv <= to {
                break;
            }
            converter(data, from, to);
        }
    }
}

// ---------------------------------------------------------------------------
// DataType — compound types. Serves both MCDataType (walkers_by_id empty) and
// IDDataType (id-discriminated walkers/converters).
// ---------------------------------------------------------------------------

pub struct DataType {
    pub name: &'static str,
    converters: Vec<(EncodedVersion, Converter)>,
    /// Inverse converters (new -> old). Run in descending version order, after
    /// the walker descent, by [`DataType::convert_reverse`].
    reverse_converters: Vec<(EncodedVersion, Converter)>,
    walkers: BTreeMap<EncodedVersion, Vec<Walker>>,
    hooks: BTreeMap<EncodedVersion, Vec<Hook>>,
    walkers_by_id: HashMap<String, BTreeMap<EncodedVersion, Vec<Walker>>>,
}

impl DataType {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            converters: Vec::new(),
            reverse_converters: Vec::new(),
            walkers: BTreeMap::new(),
            hooks: BTreeMap::new(),
            walkers_by_id: HashMap::new(),
        }
    }

    pub fn add_structure_converter(&mut self, version: i32, step: i32, converter: Converter) {
        self.converters
            .push((encode_versions(version, step), converter));
    }

    /// Register an inverse structure converter (new -> old) for reverse walks.
    /// It should undo the forward converter registered at the same `(version,
    /// step)`; see [`super::loss`] for how to report unrecoverable data.
    pub fn add_reverse_converter(&mut self, version: i32, step: i32, converter: Converter) {
        self.reverse_converters
            .push((encode_versions(version, step), converter));
    }

    /// Register a converter that only runs for compounds whose `"id"` matches —
    /// `IDDataType.addConverterForId` (IDDataType.java:23-33).
    pub fn add_converter_for_id(
        &mut self,
        id: &'static str,
        version: i32,
        step: i32,
        converter: Converter,
    ) {
        self.add_structure_converter(
            version,
            step,
            Box::new(move |data, from, to| {
                if data.get_string("id") == Some(id) {
                    converter(data, from, to);
                }
            }),
        );
    }

    /// Reverse counterpart of [`DataType::add_converter_for_id`]. The `id` it
    /// matches is the **new** (post-forward) id, since reverse converters run on
    /// data that is still in the newer schema (the inverse rename, if any, runs
    /// later in the descending sweep).
    pub fn add_reverse_converter_for_id(
        &mut self,
        id: &'static str,
        version: i32,
        step: i32,
        converter: Converter,
    ) {
        self.add_reverse_converter(
            version,
            step,
            Box::new(move |data, from, to| {
                if data.get_string("id") == Some(id) {
                    converter(data, from, to);
                }
            }),
        );
    }

    pub fn add_structure_walker(&mut self, version: i32, step: i32, walker: Walker) {
        self.walkers
            .entry(encode_versions(version, step))
            .or_default()
            .push(walker);
    }

    pub fn add_walker(&mut self, version: i32, step: i32, id: &str, walker: Walker) {
        self.walkers_by_id
            .entry(id.to_string())
            .or_default()
            .entry(encode_versions(version, step))
            .or_default()
            .push(walker);
    }

    /// `IDDataType.copyWalkers` — duplicate the floor-version walker list from
    /// `from_id` onto `to_id` (used by id renames like `Chest`→`minecraft:chest`).
    pub fn copy_walkers(&mut self, version: i32, step: i32, from_id: &str, to_id: &str) {
        let key = encode_versions(version, step);
        let to_copy: Vec<Walker> = match self.walkers_by_id.get(from_id) {
            Some(per_version) => match floor(per_version, key) {
                Some(list) => list.clone(),
                None => return,
            },
            None => return,
        };
        for w in to_copy {
            self.add_walker(version, step, to_id, w);
        }
    }

    pub fn add_structure_hook(&mut self, version: i32, step: i32, hook: Hook) {
        self.hooks
            .entry(encode_versions(version, step))
            .or_default()
            .push(hook);
    }

    pub fn finalize(&mut self) {
        self.converters.sort_by_key(|(v, _)| *v);
        self.reverse_converters.sort_by_key(|(v, _)| *v);
    }

    fn run_pre_hooks(
        &self,
        at: EncodedVersion,
        data: &mut NbtMap,
        from: EncodedVersion,
        to: EncodedVersion,
    ) {
        if let Some(hooks) = floor(&self.hooks, at) {
            for h in hooks {
                if let Some(pre) = &h.pre {
                    pre(data, from, to);
                }
            }
        }
    }

    fn run_post_hooks(
        &self,
        at: EncodedVersion,
        data: &mut NbtMap,
        from: EncodedVersion,
        to: EncodedVersion,
    ) {
        if let Some(hooks) = floor(&self.hooks, at) {
            // Post-hooks run in reverse registration order (MCDataType.java:92).
            for h in hooks.iter().rev() {
                if let Some(post) = &h.post {
                    post(data, from, to);
                }
            }
        }
    }

    /// Dispatch on the thread-local direction (see [`super::loss`]): forward
    /// callers (the default) get the faithful Java port; a reverse session gets
    /// the inverse walk. Every call site — the top-level drivers, the walker
    /// descent helpers, and direct `reg.<type>.convert(...)` calls inside
    /// converter/walker bodies — routes through here, so direction propagates
    /// uniformly without touching any version file.
    pub fn convert(
        &self,
        reg: &Registry,
        data: &mut NbtMap,
        from: EncodedVersion,
        to: EncodedVersion,
    ) {
        match loss::direction() {
            Direction::Forward => self.convert_forward(reg, data, from, to),
            Direction::Reverse => self.convert_reverse(reg, data, from, to),
        }
    }

    /// Port of `IDDataType.convert` (the superset of `MCDataType.convert`; the
    /// id-walker block is a no-op when this type has no id walkers).
    fn convert_forward(
        &self,
        reg: &Registry,
        data: &mut NbtMap,
        from: EncodedVersion,
        to: EncodedVersion,
    ) {
        // 1. Structure converters (incl. id-guarded), each wrapped in floor hooks.
        for (cv, converter) in &self.converters {
            let cv = *cv;
            if cv <= from {
                continue;
            }
            if cv > to {
                break;
            }
            self.run_pre_hooks(cv, data, from, to);
            converter(data, from, to);
            self.run_post_hooks(to, data, from, to);
        }

        // 2. Pre-hooks around the walker phase.
        self.run_pre_hooks(to, data, from, to);

        // 3. Structure walkers (latest schema ≤ toVersion).
        if let Some(walkers) = floor(&self.walkers, to) {
            for w in walkers {
                w(reg, data, from, to);
            }
        }

        // 4. Per-id walkers (IDDataType only).
        if !self.walkers_by_id.is_empty() {
            if let Some(id) = data.get_string("id").map(|s| s.to_string()) {
                if let Some(per_version) = self.walkers_by_id.get(&id) {
                    if let Some(walkers) = floor(per_version, to) {
                        for w in walkers {
                            w(reg, data, from, to);
                        }
                    }
                }
            }
        }

        // 5. Post-hooks.
        self.run_post_hooks(to, data, from, to);
    }

    /// The inverse of [`DataType::convert_forward`]: `from` is the newer version
    /// (the larger encoded value), `to` the older. The forward call did
    /// "converters (ascending), then walker @ floor(to)"; the exact inverse is
    /// "walker @ floor(from), then inverse converters (descending)" — descend
    /// **first** so nested children are reverse-converted while they are still
    /// reachable by the newer schema's paths, then undo this node's converters.
    ///
    /// The walker is selected at `floor(from)` — the same walker list the
    /// forward pass used at `floor(to)` for this segment, since both endpoints
    /// of a breakpoint segment share the segment's top version. Walkers are
    /// direction-independent (they only locate sub-structures); the recursive
    /// `convert` calls inside them re-enter this reverse path via the
    /// thread-local direction.
    fn convert_reverse(
        &self,
        reg: &Registry,
        data: &mut NbtMap,
        from: EncodedVersion,
        to: EncodedVersion,
    ) {
        // Namespace-enforce hooks on entry. They only *add* a `minecraft:`
        // prefix to a parseable unnamespaced id and no-op on legacy CamelCase
        // ids, so they are safe (and usually inert) on the newer-schema data.
        self.run_pre_hooks(from, data, from, to);

        // 1. Descend first (children reverse-convert in the newer schema).
        if let Some(walkers) = floor(&self.walkers, from) {
            for w in walkers {
                w(reg, data, from, to);
            }
        }
        if !self.walkers_by_id.is_empty() {
            if let Some(id) = data.get_string("id").map(|s| s.to_string()) {
                if let Some(per_version) = self.walkers_by_id.get(&id) {
                    if let Some(walkers) = floor(per_version, from) {
                        for w in walkers {
                            w(reg, data, from, to);
                        }
                    }
                }
            }
        }

        // 2. Undo this node's converters over (to, from] in descending order.
        for (cv, converter) in self.reverse_converters.iter().rev() {
            let cv = *cv;
            if cv > from {
                continue;
            }
            if cv <= to {
                break;
            }
            converter(data, from, to);
        }

        // 3. Namespace-enforce hooks on exit (inert on the now-older data).
        self.run_post_hooks(from, data, from, to);
    }
}

// ---------------------------------------------------------------------------
// Breakpoint-segmented walk — MCDataConverter.convertWithSubVersion.
// ---------------------------------------------------------------------------

/// Drive a conversion from `from` to `to`, splitting at the registry's
/// breakpoints (MCDataConverter.java:51-83). `apply(segment_from, segment_to)`
/// performs one type's `convert` over a breakpoint-bounded segment.
pub fn walk_with_breakpoints(
    breakpoints: &[EncodedVersion],
    from: EncodedVersion,
    to: EncodedVersion,
    mut apply: impl FnMut(EncodedVersion, EncodedVersion),
) {
    let mut current = from;
    for &bp in breakpoints {
        if current >= bp {
            continue;
        }
        let seg_to = to.min(bp - 1);
        apply(current, seg_to);
        current = seg_to;
        if current == to {
            break;
        }
    }
    if current != to {
        apply(current, to);
    }
}

/// Reverse of [`walk_with_breakpoints`]: drive a conversion from the newer
/// version `from` down to the older `to` (`from > to`), visiting the *same*
/// breakpoint-bounded segments but in descending order, with each segment's
/// endpoints swapped. `apply(segment_hi, segment_lo)` performs one type's
/// reverse `convert` over a segment.
///
/// Each hard boundary (the Flattening, the 1.20.5 split, V4290) is thus undone
/// atomically — the whole segment is one `convert_reverse` call — mirroring how
/// the forward pass applies it atomically. Segment endpoints are computed by
/// running the forward partition over `[to, from]` and reversing it, so the
/// `(lo, hi]` half-open windows line up exactly and no converter version runs
/// twice or is skipped.
pub fn walk_with_breakpoints_reverse(
    breakpoints: &[EncodedVersion],
    from: EncodedVersion,
    to: EncodedVersion,
    mut apply: impl FnMut(EncodedVersion, EncodedVersion),
) {
    let mut segments: Vec<(EncodedVersion, EncodedVersion)> = Vec::new();
    walk_with_breakpoints(breakpoints, to, from, |seg_lo, seg_hi| {
        segments.push((seg_lo, seg_hi));
    });
    for (seg_lo, seg_hi) in segments.into_iter().rev() {
        apply(seg_hi, seg_lo);
    }
}

/// Encode the endpoints of a top-level request the way `MCDataConverter.convert`
/// does: clamp the source up to ≥ V99 and use step `Integer.MAX_VALUE` on both
/// ends (MCDataConverter.java:43-49).
#[inline]
pub fn encode_endpoints(
    from_data_version: i32,
    to_data_version: i32,
) -> (EncodedVersion, EncodedVersion) {
    (
        encode_versions(from_data_version.max(V99), MAX_STEP),
        encode_versions(to_data_version, MAX_STEP),
    )
}
