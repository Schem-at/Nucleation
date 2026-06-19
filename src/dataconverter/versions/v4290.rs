//! V4290 (1.21.4 + 101) — schematic-relevant subset of `V4290.java`.
//!
//! VERSION = MCVersions.V1_21_4 (4189) + 101 = 4290.
//!
//! Ported: the step-1 TEXT_COMPONENT structure walker (V4290.java:532-575),
//! which descends a compound text component's `extra` list and `separator`, and
//! routes its `hoverEvent` sub-reads through the right schematic types:
//!   * `show_text`   -> `contents` is a nested TEXT_COMPONENT
//!   * `show_item`   -> `contents` is an ITEM_NAME (when a bare string) or an
//!                      ITEM_STACK (otherwise)
//!   * `show_entity` -> `type` is an ENTITY_NAME; `name` is a nested TEXT_COMPONENT
//!
//! Skipped / not applicable:
//!   * The TEXT_COMPONENT structure converter (V4290.java:475-529): it only does
//!     real work when the component is a *string* of unparsed JSON (it parses the
//!     SNBT/JSON, migrates legacy `hoverEvent.value` -> `contents`, and runs the
//!     ITEM/ENTITY sub-reads). In this engine a TEXT_COMPONENT is a `DataType`
//!     whose converters/walkers only ever see the already-parsed *compound* form;
//!     for a compound input the Java converter returns null (no mutation). String
//!     components are leaf values that never reach the converter, so the
//!     JSON-parse + legacy-hover migration path is unreachable and is omitted.
//!     (Map-form components in modern schematics already use `contents`, so no
//!     migration is needed.)
//!   * The `clickEvent` `run_command` / `suggest_command` descent into
//!     DATACONVERTER_CUSTOM_TYPE_COMMAND: that custom command type is not a
//!     schematic type and is absent from the restricted registry; it only
//!     rewrites the command string and carries no nested schematic data
//!     (consistent with the v1488 skip).

use std::sync::Arc;

use crate::nbt::NbtValue;

use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;
use super::super::walker::{convert, convert_list, convert_value};

const VERSION: i32 = 4290;

pub fn register(reg: &mut RegistryBuilder) {
    // step 1 — see registry breakpoint `after(4290, 0)`.
    reg.text_component.add_structure_walker(
        VERSION,
        1,
        Arc::new(|reg, root, from, to| {
            convert_list(reg, &reg.text_component, root, "extra", from, to);
            convert(reg, &reg.text_component, root, "separator", from, to);

            // clickEvent run_command/suggest_command -> custom command type
            // (non-schematic, skipped).

            if let Some(hover_event) = root.get_map_mut("hoverEvent") {
                match hover_event.get_string("action").unwrap_or("") {
                    "show_text" => {
                        convert(reg, &reg.text_component, hover_event, "contents", from, to);
                    }
                    "show_item" => {
                        // contents is an ITEM_NAME if a bare string, else ITEM_STACK.
                        let is_string =
                            matches!(hover_event.get("contents"), Some(NbtValue::String(_)));
                        if is_string {
                            convert_value(&reg.item_name, hover_event, "contents", from, to);
                        } else {
                            convert(reg, &reg.item_stack, hover_event, "contents", from, to);
                        }
                    }
                    "show_entity" => {
                        convert_value(&reg.entity_name, hover_event, "type", from, to);
                        convert(reg, &reg.text_component, hover_event, "name", from, to);
                    }
                    _ => {}
                }
            }
        }),
    );
}
