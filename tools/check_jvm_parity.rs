//! Standalone parity check: nucleation-jvm Rust JNI exports vs nucleation-py.
//!
//! Lives alongside `tools/check_api_parity.rs` and runs as a separate program
//! so the existing WASM/Python/FFI parity tool is left untouched.
//!
//! Compile: `rustc tools/check_jvm_parity.rs -o target/check_jvm_parity`
//! Run:     `./target/check_jvm_parity` (from repo root)
//!
//! Exit codes:
//!   0 — JVM surface is a superset of Python (every Python method has a
//!       matching JVM export, modulo exclusions).
//!   1 — Gaps detected; report printed to stdout.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::process::ExitCode;

/// Canonical class name (Python class minus "Py" prefix).
type Class = String;
/// Canonical lowercase snake_case method name.
type Method = String;

fn main() -> ExitCode {
    let base = std::env::var("NUCLEATION_DIR")
        .unwrap_or_else(|_| ".".to_string());

    let python = parse_python(&base);
    let jvm = parse_jvm(&base);
    let exclusions = parse_exclusions(&base);

    let mut missing: Vec<(Class, Method)> = Vec::new();
    let mut matched = 0usize;
    let mut excluded = 0usize;

    for (class, methods) in &python {
        for m in methods {
            let canonical_m = canonicalise(m);
            let present = jvm
                .get(class)
                .map(|s| s.contains(&canonical_m))
                .unwrap_or(false);
            if present {
                matched += 1;
            } else if exclusions.covers(class, &canonical_m) {
                excluded += 1;
            } else {
                missing.push((class.clone(), canonical_m));
            }
        }
    }

    println!("=== nucleation-jvm ↔ nucleation-py parity ===");
    println!("Python methods checked: {}", python.values().map(|v| v.len()).sum::<usize>());
    println!("  matched on JVM      : {matched}");
    println!("  excluded            : {excluded}");
    println!("  missing on JVM      : {}", missing.len());

    if missing.is_empty() {
        println!("\n✅ All Python methods have JVM counterparts.");
        return ExitCode::SUCCESS;
    }

    println!("\n❌ JVM is missing the following methods:");
    let mut by_class: BTreeMap<&String, Vec<&String>> = BTreeMap::new();
    for (c, m) in &missing {
        by_class.entry(c).or_default().push(m);
    }
    for (c, ms) in by_class {
        println!("  {c}:");
        for m in ms {
            println!("    - {m}");
        }
    }
    println!("\nTo silence a known JVM-side gap, add it to tools/jvm_parity_exclusions.txt:");
    println!("  <Class>.<method>    # reason");
    ExitCode::from(1)
}

// ============================================================================
// Parsers
// ============================================================================

fn parse_python(base: &str) -> BTreeMap<Class, BTreeSet<Method>> {
    let mut out: BTreeMap<Class, BTreeSet<Method>> = BTreeMap::new();
    let dir = Path::new(base).join("src/python");
    if !dir.exists() {
        return out;
    }
    for entry in fs::read_dir(&dir).unwrap().flatten() {
        let p = entry.path();
        if p.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        if p.file_stem().and_then(|s| s.to_str()) == Some("mod") {
            continue;
        }
        let content = match fs::read_to_string(&p) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;
        while i < lines.len() {
            let line = lines[i].trim();
            if line == "#[pymethods]" {
                // Find the next `impl Foo {`
                let mut j = i + 1;
                while j < lines.len() && !lines[j].trim_start().starts_with("impl ") {
                    j += 1;
                }
                if j >= lines.len() {
                    break;
                }
                let impl_name = match extract_impl_name(lines[j].trim()) {
                    Some(n) => n,
                    None => {
                        i = j + 1;
                        continue;
                    }
                };
                let canonical = impl_name
                    .strip_prefix("Py")
                    .unwrap_or(&impl_name)
                    .to_string();
                // Walk the impl block while tracking brace depth.
                let mut depth = 0i32;
                let mut started = false;
                let mut k = j;
                while k < lines.len() {
                    let l = lines[k];
                    for ch in l.chars() {
                        if ch == '{' {
                            depth += 1;
                            started = true;
                        } else if ch == '}' {
                            depth -= 1;
                        }
                    }
                    // Capture method declarations at depth 1 only (top-level
                    // of the impl block, not nested function bodies).
                    let trimmed = l.trim();
                    if started && depth >= 1 {
                        if let Some(name) = extract_fn_name(trimmed) {
                            out.entry(canonical.clone())
                                .or_default()
                                .insert(canonicalise(&name));
                        }
                    }
                    if started && depth <= 0 {
                        i = k + 1;
                        break;
                    }
                    k += 1;
                }
                if k >= lines.len() {
                    break;
                }
                continue;
            }
            i += 1;
        }
    }
    out
}

fn parse_jvm(base: &str) -> BTreeMap<Class, BTreeSet<Method>> {
    let mut out: BTreeMap<Class, BTreeSet<Method>> = BTreeMap::new();
    let dir = Path::new(base).join("nucleation-jvm/src/exports");
    if !dir.exists() {
        return out;
    }
    // Map JNI-name prefix → canonical Python class name.
    // Longer prefixes must come first so "BlockState" matches before "Block".
    let prefix_map: &[(&str, &str)] = &[
        ("SchematicMeshByRegion", "Schematic"),
        ("SchematicMesh", "Schematic"),
        ("Schematic", "Schematic"),
        ("BlockState", "BlockState"),
        ("Building", "BuildingTool"),
        ("Builder", "SchematicBuilder"),
        ("Brush", "Brush"),
        ("Shape", "Shape"),
        ("Mchprs", "MchprsWorld"),
        ("ResourcePack", "ResourcePack"),
        ("MeshConfig", "MeshConfig"),
        ("MeshResult", "MeshResult"),
        ("MultiMesh", "MultiMeshResult"),
        ("Diff", "Diff"),
    ];
    for entry in fs::read_dir(&dir).unwrap().flatten() {
        let p = entry.path();
        if p.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let content = match fs::read_to_string(&p) {
            Ok(c) => c,
            Err(_) => continue,
        };
        for line in content.lines() {
            let Some(name) = extract_nm_name(line) else { continue };
            let body = name.strip_prefix('n').unwrap_or(&name);
            let (class, method_pascal) = match prefix_map.iter().find_map(|(pref, cls)| {
                body.strip_prefix(pref).map(|rest| (*cls, rest))
            }) {
                Some(v) => v,
                None => continue,
            };
            if method_pascal.is_empty() {
                continue;
            }
            let method_snake = pascal_to_snake(method_pascal);
            out.entry(class.to_string())
                .or_default()
                .insert(canonicalise(&method_snake));
        }
    }
    out
}

fn pascal_to_snake(name: &str) -> String {
    let mut out = String::new();
    for (i, c) in name.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            out.push('_');
        }
        out.push(c.to_ascii_lowercase());
    }
    out
}

struct Exclusions {
    /// Exact (class, method) excluded pairs.
    exact: BTreeSet<(Class, Method)>,
    /// Wildcard "*.method_or_pattern" — excludes that method on any class.
    /// Patterns ending in `*` are treated as a prefix match.
    wildcard_method: BTreeSet<String>,
    /// "ClassName.*" — every method on that class excluded.
    wildcard_class: BTreeSet<Class>,
}

impl Exclusions {
    fn covers(&self, class: &str, method: &str) -> bool {
        if self.exact.contains(&(class.to_string(), method.to_string())) {
            return true;
        }
        if self.wildcard_class.contains(class)
            || self
                .wildcard_class
                .contains(&format!("Py{class}"))
        {
            return true;
        }
        for w in &self.wildcard_method {
            if let Some(prefix) = w.strip_suffix('*') {
                if method.starts_with(prefix) {
                    return true;
                }
            } else if w == method {
                return true;
            }
        }
        false
    }
}

fn parse_exclusions(base: &str) -> Exclusions {
    let mut exact = BTreeSet::new();
    let mut wildcard_method = BTreeSet::new();
    let mut wildcard_class = BTreeSet::new();
    let path = Path::new(base).join("tools/jvm_parity_exclusions.txt");
    if let Ok(content) = fs::read_to_string(&path) {
        for line in content.lines() {
            let line = line.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            let pair = if let Some(eq) = line.find('=') {
                line[..eq].trim().to_string()
            } else {
                line.to_string()
            };
            let Some(dot) = pair.find('.') else { continue };
            let class = pair[..dot].trim().to_string();
            let method = canonicalise(pair[dot + 1..].trim());
            if class == "*" {
                wildcard_method.insert(method);
            } else if method == "*" {
                wildcard_class.insert(class);
            } else {
                exact.insert((class, method));
            }
        }
    }
    Exclusions {
        exact,
        wildcard_method,
        wildcard_class,
    }
}

// ============================================================================
// Helpers
// ============================================================================

fn extract_impl_name(line: &str) -> Option<String> {
    let rest = line.strip_prefix("impl ")?.trim();
    let stop = rest
        .find(|c: char| c == '<' || c == '{' || c == ' ')
        .unwrap_or(rest.len());
    Some(rest[..stop].to_string())
}

fn extract_fn_name(line: &str) -> Option<String> {
    let l = line.trim_start_matches("pub ").trim_start();
    let l = l.strip_prefix("fn ")?;
    let stop = l.find(|c: char| c == '(' || c == '<')?;
    Some(l[..stop].trim().to_string())
}

/// Extract first string literal in `nm("name", ...)`.
fn extract_nm_name(line: &str) -> Option<String> {
    let pos = line.find("nm(\"")?;
    let after = &line[pos + 4..];
    let end = after.find('"')?;
    Some(after[..end].to_string())
}

/// Normalise getters/setters and common naming differences to a comparable form.
fn canonicalise(name: &str) -> String {
    let lower = name.to_string();
    // Strip "get_" prefix to align Python `#[getter] fn name` with JVM `getName`.
    if let Some(rest) = lower.strip_prefix("get_") {
        return rest.to_string();
    }
    if let Some(rest) = lower.strip_prefix("set_") {
        return format!("set_{}", rest);
    }
    lower
}
