use serde_json::Value;
use std::io::{Cursor, Read, Write};
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

/// Normalize resource-pack JSON added by newer Minecraft versions into the
/// equivalent schema understood by schematic-mesher 0.2.
///
/// Returns `Ok(None)` for non-ZIP input or when no entry needs normalization.
pub(super) fn normalize_zip(data: &[u8]) -> Result<Option<Vec<u8>>, String> {
    let mut archive = match ZipArchive::new(Cursor::new(data)) {
        Ok(archive) => archive,
        Err(_) => return Ok(None),
    };

    let mut entries = Vec::with_capacity(archive.len());
    let mut any_changed = false;

    for index in 0..archive.len() {
        let mut file = archive.by_index(index).map_err(|e| e.to_string())?;
        let name = file.name().to_string();
        let is_dir = file.is_dir();
        let compression = file.compression();
        let permissions = file.unix_mode();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).map_err(|e| e.to_string())?;

        if name.ends_with(".json") {
            if let Ok(mut json) = serde_json::from_slice::<Value>(&bytes) {
                let changed = if name.contains("/models/") {
                    normalize_model(&mut json)
                } else if name.contains("/blockstates/") {
                    normalize_blockstate(&mut json)
                } else {
                    false
                };
                if changed {
                    bytes = serde_json::to_vec(&json).map_err(|e| e.to_string())?;
                    any_changed = true;
                }
            }
        }

        entries.push((name, is_dir, compression, permissions, bytes));
    }

    if !any_changed {
        return Ok(None);
    }

    let mut output = Cursor::new(Vec::with_capacity(data.len()));
    {
        let mut writer = ZipWriter::new(&mut output);
        for (name, is_dir, compression, permissions, bytes) in entries {
            let mut options = SimpleFileOptions::default().compression_method(compression);
            if let Some(mode) = permissions {
                options = options.unix_permissions(mode);
            }
            if is_dir {
                writer
                    .add_directory(name, options)
                    .map_err(|e| e.to_string())?;
            } else {
                writer
                    .start_file(name, options)
                    .map_err(|e| e.to_string())?;
                writer.write_all(&bytes).map_err(|e| e.to_string())?;
            }
        }
        writer.finish().map_err(|e| e.to_string())?;
    }
    Ok(Some(output.into_inner()))
}

fn normalize_model(root: &mut Value) -> bool {
    let mut changed = false;

    if let Some(textures) = root.get_mut("textures").and_then(Value::as_object_mut) {
        for texture in textures.values_mut() {
            if let Some(sprite) = texture
                .as_object()
                .and_then(|object| object.get("sprite"))
                .and_then(Value::as_str)
                .map(str::to_owned)
            {
                *texture = Value::String(sprite);
                changed = true;
            }
        }
    }

    changed | normalize_direction_names(root)
}

fn normalize_direction_names(value: &mut Value) -> bool {
    let mut changed = false;
    match value {
        Value::Object(object) => {
            for (key, child) in object {
                if key == "cullface" {
                    if let Some(direction) = child.as_str() {
                        let canonical = match direction {
                            "bottom" => Some("down"),
                            "top" => Some("up"),
                            _ => None,
                        };
                        if let Some(canonical) = canonical {
                            *child = Value::String(canonical.to_string());
                            changed = true;
                        }
                    }
                } else {
                    changed |= normalize_direction_names(child);
                }
            }
        }
        Value::Array(values) => {
            for child in values {
                changed |= normalize_direction_names(child);
            }
        }
        _ => {}
    }
    changed
}

fn normalize_blockstate(root: &mut Value) -> bool {
    let Some(multipart) = root.get_mut("multipart").and_then(Value::as_array_mut) else {
        return false;
    };
    multipart.iter_mut().fold(false, |changed, part| {
        changed
            | part
                .get_mut("when")
                .map(normalize_condition_scalars)
                .unwrap_or(false)
    })
}

fn normalize_condition_scalars(value: &mut Value) -> bool {
    let mut changed = false;
    match value {
        Value::Object(object) => {
            for child in object.values_mut() {
                changed |= normalize_condition_scalars(child);
            }
        }
        Value::Array(values) => {
            for child in values {
                changed |= normalize_condition_scalars(child);
            }
        }
        Value::Number(number) => {
            *value = Value::String(number.to_string());
            changed = true;
        }
        Value::Bool(boolean) => {
            *value = Value::String(boolean.to_string());
            changed = true;
        }
        _ => {}
    }
    changed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modern_model_textures_and_direction_aliases_are_normalized() {
        let mut model = serde_json::json!({
            "textures": {
                "all": {"sprite": "minecraft:block/glass", "force_translucent": true}
            },
            "elements": [{"faces": {"down": {"cullface": "bottom"}}}]
        });

        assert!(normalize_model(&mut model));
        assert_eq!(model["textures"]["all"], "minecraft:block/glass");
        assert_eq!(model["elements"][0]["faces"]["down"]["cullface"], "down");
    }

    #[test]
    fn numeric_multipart_conditions_become_property_strings() {
        let mut blockstate = serde_json::json!({
            "multipart": [{"when": {"power": 0}, "apply": {"model": "block/x"}}]
        });

        assert!(normalize_blockstate(&mut blockstate));
        assert_eq!(blockstate["multipart"][0]["when"]["power"], "0");
    }
}
