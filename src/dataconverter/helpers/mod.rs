//! Reusable converter helpers — the rename infrastructure that the bulk of the
//! version files (bucket A in the findings) are built on, plus the value-rename
//! primitive. Ported from `converters/helpers/`, `converters/blockname/`,
//! `converters/itemname/`, `converters/entity/`.

use std::collections::HashMap;
use std::sync::Arc;

use crate::nbt::{NbtMap, NbtValue};

use super::engine::{Hook, MCValueType, ValueHook};
use super::registry::RegistryBuilder;
use super::types::MapExt;

/// A rename function: returns `Some(new_id)` to rename, `None` to leave as-is.
/// (Java models this as `Function<String,String>` returning the new name or
/// null.)
pub type Renamer = Arc<dyn Fn(&str) -> Option<String> + Send + Sync>;

/// A rename registration that knows *both* directions. The `register_*_rename`
/// helpers consume this and register a forward converter **and** its inverse, so
/// every table-driven rename (bucket A in the findings) is reversed for free —
/// without touching any of the ~90 `map_renamer(TABLE)` call sites, which now
/// produce a [`RenameSpec`] instead of a bare [`Renamer`].
pub enum RenameSpec {
    /// A static `(old, new)` table — the inverse is `(new, old)` (first-wins on
    /// the rare many-to-one collision).
    Pairs(&'static [(&'static str, &'static str)]),
    /// A bespoke forward closure with an explicit inverse. Constructed via
    /// [`RenameSpec::custom`]; a plain [`Renamer`] converts to this with a
    /// **no-op** reverse (the safe default when an inverse can't be derived).
    Custom { forward: Renamer, reverse: Renamer },
}

impl RenameSpec {
    /// A bespoke rename with an explicit inverse renamer.
    pub fn custom(forward: Renamer, reverse: Renamer) -> Self {
        RenameSpec::Custom { forward, reverse }
    }

    /// The forward (old -> new) renamer.
    pub fn forward(&self) -> Renamer {
        match self {
            RenameSpec::Pairs(pairs) => map_table_renamer(pairs.iter().map(|(o, n)| (*o, *n))),
            RenameSpec::Custom { forward, .. } => forward.clone(),
        }
    }

    /// The reverse (new -> old) renamer.
    pub fn reverse(&self) -> Renamer {
        match self {
            RenameSpec::Pairs(pairs) => map_table_renamer(pairs.iter().map(|(o, n)| (*n, *o))),
            RenameSpec::Custom { reverse, .. } => reverse.clone(),
        }
    }
}

impl From<Renamer> for RenameSpec {
    fn from(forward: Renamer) -> Self {
        // No safe automatic inverse for an opaque closure; default to a no-op
        // reverse (leave the value unchanged on downgrade). The two closure-based
        // rename sites that need a real inverse build a `RenameSpec::custom`.
        RenameSpec::Custom {
            forward,
            reverse: Arc::new(|_| None),
        }
    }
}

/// Build a renamer from `(from, to)` pairs, first-wins on duplicate `from` keys.
fn map_table_renamer(pairs: impl Iterator<Item = (&'static str, &'static str)>) -> Renamer {
    let mut map: HashMap<&'static str, &'static str> = HashMap::new();
    for (from, to) in pairs {
        map.entry(from).or_insert(to);
    }
    Arc::new(move |id: &str| map.get(id).map(|s| s.to_string()))
}

/// Build a rename spec from a static `(old, new)` table. The reverse engine
/// inverts the table automatically (see [`RenameSpec`]).
pub fn map_renamer(pairs: &'static [(&'static str, &'static str)]) -> RenameSpec {
    RenameSpec::Pairs(pairs)
}

/// Invert a rename table for reverse conversion. Panics in debug if the forward
/// table is not injective (two old ids map to one new id), which would make the
/// inverse ambiguous — those must be handled as lossy (bucket C).
pub fn invert_pairs(
    pairs: &'static [(&'static str, &'static str)],
) -> Vec<(&'static str, &'static str)> {
    let mut seen: HashMap<&'static str, &'static str> = HashMap::new();
    let mut out = Vec::with_capacity(pairs.len());
    for (old, new) in pairs {
        debug_assert!(
            seen.insert(*new, *old).is_none(),
            "non-injective rename table: {new} has multiple preimages"
        );
        out.push((*new, *old));
    }
    out
}

/// Build a value-rename converter closure from a renamer.
fn value_rename_converter(renamer: Renamer) -> super::engine::ValueConverter {
    Box::new(move |val: &mut NbtValue, _from, _to| {
        if let NbtValue::String(s) = val {
            if let Some(new) = renamer(s) {
                *s = new;
            }
        }
    })
}

/// Build a `{Name}`-field rename converter closure (for BLOCK_STATE).
fn name_field_rename_converter(renamer: Renamer) -> super::engine::Converter {
    Box::new(move |data: &mut NbtMap, _from, _to| {
        if let Some(name) = data.get_string("Name").map(|s| s.to_string()) {
            if let Some(new) = renamer(&name) {
                data.set_string("Name", new);
            }
        }
    })
}

/// Build an `{id}`-field rename converter closure (for ENTITY).
fn id_field_rename_converter(renamer: Renamer) -> super::engine::Converter {
    Box::new(move |data: &mut NbtMap, _from, _to| {
        if let Some(id) = data.get_string("id").map(|s| s.to_string()) {
            if let Some(new) = renamer(&id) {
                data.set_string("id", new);
            }
        }
    })
}

/// Build a flattened-blockstate-string rename converter closure (renames only
/// the block-name prefix before the first `[` / `{`).
fn flat_state_rename_converter(renamer: Renamer) -> super::engine::ValueConverter {
    Box::new(move |val: &mut NbtValue, _from, _to| {
        if let NbtValue::String(s) = val {
            if s.is_empty() {
                return;
            }
            let end = flat_state_name_end(s);
            if let Some(new) = renamer(&s[..end]) {
                let rest = s[end..].to_string();
                *s = format!("{new}{rest}");
            }
        }
    })
}

/// `ConverterAbstractStringValueTypeRename.register` — register a value-type
/// rename converter (e.g. on ITEM_NAME / ENTITY_NAME / BLOCK_NAME / BIOME) and
/// its inverse for the reverse engine.
pub fn register_value_rename(
    ty: &mut MCValueType,
    version: i32,
    step: i32,
    spec: impl Into<RenameSpec>,
) {
    let spec = spec.into();
    ty.add_converter(version, step, value_rename_converter(spec.forward()));
    ty.add_reverse_converter(version, step, value_rename_converter(spec.reverse()));
}

/// `ConverterAbstractItemRename.register` — rename on ITEM_NAME (both directions).
pub fn register_item_rename(reg: &mut RegistryBuilder, version: i32, spec: impl Into<RenameSpec>) {
    register_value_rename(&mut reg.item_name, version, 0, spec);
}

/// `ConverterAbstractEntityRename.register` — rename the entity `id` field and
/// the ENTITY_NAME value type, plus the inverses.
pub fn register_entity_rename(
    reg: &mut RegistryBuilder,
    version: i32,
    spec: impl Into<RenameSpec>,
) {
    let spec = spec.into();
    reg.entity
        .add_structure_converter(version, 0, id_field_rename_converter(spec.forward()));
    reg.entity
        .add_reverse_converter(version, 0, id_field_rename_converter(spec.reverse()));
    register_value_rename(&mut reg.entity_name, version, 0, spec);
}

/// `ConverterAbstractBlockRename.register` — rename BLOCK_NAME, the BLOCK_STATE
/// `Name` field, and the FLAT_BLOCK_STATE string prefix, plus the inverses.
pub fn register_block_rename(reg: &mut RegistryBuilder, version: i32, spec: impl Into<RenameSpec>) {
    let spec = spec.into();

    register_value_rename(
        &mut reg.block_name,
        version,
        0,
        RenameSpec::custom(spec.forward(), spec.reverse()),
    );

    reg.block_state.add_structure_converter(
        version,
        0,
        name_field_rename_converter(spec.forward()),
    );
    reg.block_state
        .add_reverse_converter(version, 0, name_field_rename_converter(spec.reverse()));

    reg.flat_block_state
        .add_converter(version, 0, flat_state_rename_converter(spec.forward()));
    reg.flat_block_state.add_reverse_converter(
        version,
        0,
        flat_state_rename_converter(spec.reverse()),
    );
}

// --- namespace enforcement (hooks/DataHook*EnforceNamespaced) ---------------

/// `NamespaceUtil.correctNamespaceOrNull` — mirrors Java, which delegates to
/// `Identifier.tryParse(value)`: an unnamespaced *valid* path gains the
/// `minecraft:` namespace, an already-namespaced value is unchanged, and a value
/// that is **not a parseable resource location** (e.g. a CamelCase legacy id like
/// `Chest`) is left untouched (`tryParse` returns null → no change). Returns the
/// corrected string only when it actually differs from the input.
pub fn correct_namespace_or_null(value: &str) -> Option<String> {
    if value.is_empty() {
        return None;
    }
    let corrected = correct_namespace(value);
    if corrected == value {
        None
    } else {
        Some(corrected)
    }
}

/// `NamespaceUtil.correctNamespace` — `Identifier.tryParse(value)?.toString()`,
/// falling back to the original value when it does not parse as a resource
/// location. Mojang's `ResourceLocation` accepts `[a-z0-9._-]` in the namespace
/// and `[a-z0-9._/-]` in the path; an absent namespace defaults to `minecraft`.
pub fn correct_namespace(value: &str) -> String {
    match try_parse_identifier(value) {
        Some(canonical) => canonical,
        None => value.to_string(),
    }
}

/// Port of `ResourceLocation.tryParse`: returns the canonical `namespace:path`
/// form, or `None` when the value contains characters illegal for a resource
/// location.
fn try_parse_identifier(value: &str) -> Option<String> {
    let (namespace, path) = match value.find(':') {
        Some(i) => (&value[..i], &value[i + 1..]),
        None => ("minecraft", value),
    };
    if !is_valid_namespace(namespace) || !is_valid_path(path) {
        return None;
    }
    Some(format!("{namespace}:{path}"))
}

fn is_valid_namespace(namespace: &str) -> bool {
    namespace
        .bytes()
        .all(|b| matches!(b, b'a'..=b'z' | b'0'..=b'9' | b'.' | b'_' | b'-'))
}

fn is_valid_path(path: &str) -> bool {
    path.bytes()
        .all(|b| matches!(b, b'a'..=b'z' | b'0'..=b'9' | b'.' | b'_' | b'-' | b'/'))
}

/// `DataHookEnforceNamespacedID` — pre-hook that namespaces the `id` (or other
/// path) field of a compound.
pub fn enforce_namespaced_id_hook(path: &'static str) -> Hook {
    Hook {
        pre: Some(Box::new(move |data, _from, _to| {
            if let Some(id) = data.get_string(path).map(|s| s.to_string()) {
                if let Some(new) = correct_namespace_or_null(&id) {
                    data.set_string(path, new);
                }
            }
        })),
        post: None,
    }
}

/// `DataHookValueTypeEnforceNamespaced` — pre-hook that namespaces a value-type
/// string (block/item id).
pub fn enforce_namespaced_value_hook() -> ValueHook {
    ValueHook {
        pre: Some(Box::new(|val, _from, _to| {
            if let NbtValue::String(s) = val {
                if let Some(new) = correct_namespace_or_null(s) {
                    *s = new;
                }
            }
        })),
        post: None,
    }
}

/// The end index of the block name in a flattened blockstate string, i.e. the
/// first `[` or `{` (ConverterAbstractBlockRename.java:43-53).
fn flat_state_name_end(s: &str) -> usize {
    let b = s.find('[').filter(|&i| i > 0);
    let c = s.find('{').filter(|&i| i > 0);
    match (b, c) {
        (Some(b), Some(c)) => b.min(c),
        (Some(b), None) => b,
        (None, Some(c)) => c,
        (None, None) => s.len(),
    }
}

// --- text-component helpers (ComponentUtils) --------------------------------

/// `ComponentUtils.createPlainTextComponent` — `{"text": text}` as stable JSON.
pub fn create_plain_text_component(text: &str) -> String {
    serde_json::to_string(&serde_json::json!({ "text": text }))
        .expect("serializing a single-key object never fails")
}

/// `ComponentUtils.createTranslatableComponent` — `{"translate": key}` as JSON.
pub fn create_translatable_component(key: &str) -> String {
    serde_json::to_string(&serde_json::json!({ "translate": key }))
        .expect("serializing a single-key object never fails")
}

/// `ComponentUtils.isValidJson` — whether the string parses as JSON.
pub fn is_valid_json(input: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(input).is_ok()
}

/// `ComponentUtils.convertFromLenient` — coerce a legacy lenient string into a
/// proper JSON text component: empty/"null" → `{"text":""}`; a value that looks
/// like JSON (matching quote/brace/bracket delimiters) is parsed — a JSON
/// primitive becomes `{"text": <its string>}`, anything else is re-emitted as
/// stable JSON; otherwise the raw string is wrapped as plain text.
pub fn convert_from_lenient(input: &str) -> String {
    if input.is_empty() || input == "null" {
        return create_plain_text_component("");
    }
    let bytes = input.as_bytes();
    let first = bytes[0];
    let last = bytes[bytes.len() - 1];
    let looks_json = (first == b'"' && last == b'"')
        || (first == b'{' && last == b'}')
        || (first == b'[' && last == b']');
    if looks_json {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(input) {
            if let serde_json::Value::String(s) = &json {
                return create_plain_text_component(s);
            }
            return serde_json::to_string(&json).expect("re-serialize parsed JSON");
        }
    }
    create_plain_text_component(input)
}
