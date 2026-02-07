// tools/check_api_parity.rs
// API Parity Checker for Nucleation
//
// Detects public APIs across WASM, Python, and FFI bindings and reports gaps.
// Compile: rustc tools/check_api_parity.rs -o target/check_api_parity
// Usage:   target/check_api_parity [--generate-stubs] [--verbose] [--json]

#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::Path;

// ============================================================================
// Data Structures
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Target {
    Wasm,
    Python,
    Ffi,
}

impl Target {
    fn label(&self) -> &'static str {
        match self {
            Target::Wasm => "WASM",
            Target::Python => "Python",
            Target::Ffi => "FFI",
        }
    }
}

#[derive(Debug, Clone)]
struct ApiMethod {
    name: String,
    is_constructor: bool,
    is_static: bool,
    is_getter: bool,
    is_setter: bool,
    feature_gate: Option<String>,
    source_file: String,
    source_line: usize,
}

#[derive(Debug, Clone)]
struct ApiClass {
    canonical_name: String,
    methods: Vec<ApiMethod>,
}

#[derive(Debug)]
struct ApiSurface {
    target: Target,
    classes: Vec<ApiClass>,
    free_functions: Vec<ApiMethod>,
}

// ============================================================================
// Exclusion Parsing
// ============================================================================

#[derive(Debug)]
struct Exclusions {
    wasm_only: Vec<ExclusionEntry>,
    python_only: Vec<ExclusionEntry>,
    ffi_only: Vec<ExclusionEntry>,
    no_ffi: Vec<ExclusionEntry>,
    no_wasm: Vec<ExclusionEntry>,
    no_python: Vec<ExclusionEntry>,
}

#[derive(Debug)]
struct ExclusionEntry {
    class_pattern: String,  // "*" or exact
    method_pattern: String, // "*" or exact or "prefix*"
    reason: String,
}

impl Exclusions {
    fn is_excluded(&self, class: &str, method: &str, missing_from: Target) -> Option<&str> {
        // [wasm_only] = method is expected to only exist in WASM -> exclude if missing from Python/FFI
        // [python_only] = method is expected to only exist in Python -> exclude if missing from WASM/FFI
        // [ffi_only] = method is expected to only exist in FFI -> exclude if missing from WASM/Python
        // Check all three *_only lists
        for (entries, only_target) in [
            (&self.wasm_only, Target::Wasm),
            (&self.python_only, Target::Python),
            (&self.ffi_only, Target::Ffi),
        ] {
            if missing_from == only_target {
                continue; // If missing from the "only" target, don't exclude
            }
            for entry in entries {
                if entry.matches(class, method) {
                    return Some(&entry.reason);
                }
            }
        }
        // [no_*] entries exclude when missing from the specific target
        let no_entries: &[(&Vec<ExclusionEntry>, Target)] = &[
            (&self.no_ffi, Target::Ffi),
            (&self.no_wasm, Target::Wasm),
            (&self.no_python, Target::Python),
        ];
        for (entries, target) in no_entries {
            if missing_from == *target {
                for entry in *entries {
                    if entry.matches(class, method) {
                        return Some(&entry.reason);
                    }
                }
            }
        }
        None
    }

    fn is_excluded_free_fn(&self, method: &str, missing_from: Target) -> Option<&str> {
        self.is_excluded("", method, missing_from)
    }
}

impl ExclusionEntry {
    fn matches(&self, class: &str, method: &str) -> bool {
        let class_match = self.class_pattern == "*"
            || self.class_pattern.is_empty()
            || self.class_pattern == class;

        if !class_match {
            return false;
        }

        if self.method_pattern == "*" {
            return true;
        }
        if self.method_pattern.ends_with('*') {
            let prefix = &self.method_pattern[..self.method_pattern.len() - 1];
            return method.starts_with(prefix);
        }
        self.method_pattern == method
    }
}

fn parse_exclusions(path: &str) -> Exclusions {
    let mut exclusions = Exclusions {
        wasm_only: Vec::new(),
        python_only: Vec::new(),
        ffi_only: Vec::new(),
        no_ffi: Vec::new(),
        no_wasm: Vec::new(),
        no_python: Vec::new(),
    };

    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return exclusions,
    };

    #[derive(Clone, Copy)]
    enum Section {
        TargetOnly(Target),
        No(Target),
    }

    let mut current_section: Option<Section> = None;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if line.starts_with('[') && line.ends_with(']') {
            let section = &line[1..line.len() - 1];
            current_section = match section {
                "wasm_only" => Some(Section::TargetOnly(Target::Wasm)),
                "python_only" => Some(Section::TargetOnly(Target::Python)),
                "ffi_only" => Some(Section::TargetOnly(Target::Ffi)),
                "no_ffi" => Some(Section::No(Target::Ffi)),
                "no_wasm" => Some(Section::No(Target::Wasm)),
                "no_python" => Some(Section::No(Target::Python)),
                _ => None,
            };
            continue;
        }

        if let Some(section) = current_section {
            // Parse: Class.method = reason  OR  free_function = reason
            if let Some(eq_pos) = line.find('=') {
                let key = line[..eq_pos].trim();
                let reason = line[eq_pos + 1..].trim().to_string();

                let (class_pattern, method_pattern) = if let Some(dot_pos) = key.find('.') {
                    (
                        key[..dot_pos].to_string(),
                        key[dot_pos + 1..].to_string(),
                    )
                } else {
                    // Free function
                    (String::new(), key.to_string())
                };

                let entry = ExclusionEntry {
                    class_pattern,
                    method_pattern,
                    reason,
                };

                match section {
                    Section::TargetOnly(Target::Wasm) => exclusions.wasm_only.push(entry),
                    Section::TargetOnly(Target::Python) => exclusions.python_only.push(entry),
                    Section::TargetOnly(Target::Ffi) => exclusions.ffi_only.push(entry),
                    Section::No(Target::Ffi) => exclusions.no_ffi.push(entry),
                    Section::No(Target::Wasm) => exclusions.no_wasm.push(entry),
                    Section::No(Target::Python) => exclusions.no_python.push(entry),
                };
            }
        }
    }

    exclusions
}

// ============================================================================
// Feature Gate Detection
// ============================================================================

fn detect_feature_gates(mod_path: &str) -> HashMap<String, String> {
    // Maps filename (without .rs) -> feature name
    let mut gates: HashMap<String, String> = HashMap::new();

    let content = match fs::read_to_string(mod_path) {
        Ok(c) => c,
        Err(_) => return gates,
    };

    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim();
        // Look for #[cfg(feature = "xxx")]
        if line.starts_with("#[cfg(feature") {
            if let Some(start) = line.find('"') {
                if let Some(end) = line[start + 1..].find('"') {
                    let feature = line[start + 1..start + 1 + end].to_string();
                    // Next line should be `mod foo;`
                    if i + 1 < lines.len() {
                        let next_line = lines[i + 1].trim();
                        if next_line.starts_with("mod ") {
                            if let Some(semi) = next_line.find(';') {
                                let mod_name = next_line[4..semi].trim().to_string();
                                gates.insert(mod_name, feature);
                            }
                        }
                    }
                }
            }
        }
        i += 1;
    }
    gates
}

// ============================================================================
// WASM Parser
// ============================================================================

fn parse_wasm_surface(base_dir: &str) -> ApiSurface {
    let wasm_dir = format!("{}/src/wasm", base_dir);
    let feature_gates = detect_feature_gates(&format!("{}/mod.rs", wasm_dir));

    let mut classes: Vec<ApiClass> = Vec::new();
    let mut free_functions: Vec<ApiMethod> = Vec::new();

    // Collect all .rs files in the wasm directory
    let entries = match fs::read_dir(&wasm_dir) {
        Ok(e) => e,
        Err(_) => {
            return ApiSurface {
                target: Target::Wasm,
                classes,
                free_functions,
            }
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().map(|e| e == "rs").unwrap_or(false) {
            let filename = path.file_stem().unwrap().to_str().unwrap().to_string();
            if filename == "mod" {
                continue;
            }

            let feature = feature_gates.get(&filename).cloned();
            let content = match fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => continue,
            };
            let file_str = format!("src/wasm/{}.rs", filename);

            parse_wasm_file(
                &content,
                &file_str,
                feature.as_deref(),
                &mut classes,
                &mut free_functions,
            );
        }
    }

    // Merge classes with the same canonical name
    let merged = merge_classes(classes);

    ApiSurface {
        target: Target::Wasm,
        classes: merged,
        free_functions,
    }
}

fn normalize_wasm_class_name(name: &str) -> String {
    let name = if name.ends_with("Wrapper") {
        &name[..name.len() - 7]
    } else {
        name
    };
    let name = if name.starts_with("Wasm") {
        &name[4..]
    } else {
        name
    };
    name.to_string()
}

fn parse_wasm_file(
    content: &str,
    file_path: &str,
    default_feature: Option<&str>,
    classes: &mut Vec<ApiClass>,
    free_functions: &mut Vec<ApiMethod>,
) {
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        // Check for feature-gated sections within a file
        let mut local_feature = default_feature.map(|s| s.to_string());
        if line.starts_with("#[cfg(feature") {
            if let Some(f) = extract_feature_from_cfg(line) {
                local_feature = Some(f);
            }
        }

        // Look for #[wasm_bindgen] on free functions
        if line == "#[wasm_bindgen]" || line.starts_with("#[wasm_bindgen(") {
            // Peek ahead: is this before an `impl` block or a standalone `pub fn`?
            let mut j = i + 1;
            while j < lines.len() && lines[j].trim().starts_with('#') || lines[j].trim().is_empty()
            {
                j += 1;
            }
            if j < lines.len() {
                let next_line = lines[j].trim();
                if next_line.starts_with("pub fn ") {
                    // Free function
                    if let Some(name) = extract_fn_name(next_line) {
                        free_functions.push(ApiMethod {
                            name,
                            is_constructor: false,
                            is_static: false,
                            is_getter: false,
                            is_setter: false,
                            feature_gate: local_feature.clone(),
                            source_file: file_path.to_string(),
                            source_line: j + 1,
                        });
                    }
                    i = j + 1;
                    continue;
                } else if next_line.starts_with("impl ") {
                    // impl block
                    let struct_name = extract_impl_struct_name(next_line);
                    if let Some(struct_name) = struct_name {
                        let canonical = normalize_wasm_class_name(&struct_name);
                        let (methods, end_line) = parse_wasm_impl_block(
                            &lines,
                            j,
                            file_path,
                            local_feature.as_deref(),
                        );
                        classes.push(ApiClass {
                            canonical_name: canonical,
                            methods,
                        });
                        i = end_line;
                        continue;
                    }
                }
            }
        }

        // Also catch #[wasm_bindgen(start)] on a function
        if line.starts_with("#[wasm_bindgen(start)]") {
            let mut j = i + 1;
            while j < lines.len() && (lines[j].trim().is_empty() || lines[j].trim().starts_with('#')) {
                j += 1;
            }
            if j < lines.len() {
                let next_line = lines[j].trim();
                if next_line.starts_with("pub fn ") {
                    if let Some(name) = extract_fn_name(next_line) {
                        free_functions.push(ApiMethod {
                            name,
                            is_constructor: false,
                            is_static: false,
                            is_getter: false,
                            is_setter: false,
                            feature_gate: local_feature.clone(),
                            source_file: file_path.to_string(),
                            source_line: j + 1,
                        });
                    }
                    i = j + 1;
                    continue;
                }
            }
        }

        i += 1;
    }
}

fn count_braces(line: &str) -> (usize, usize) {
    let mut opens = 0;
    let mut closes = 0;
    let mut in_string = false;
    let in_char = false;
    let mut escape_next = false;
    let chars: Vec<char> = line.chars().collect();
    let len = chars.len();
    let mut i = 0;
    while i < len {
        let ch = chars[i];
        if escape_next {
            escape_next = false;
            i += 1;
            continue;
        }
        if ch == '\\' && (in_string || in_char) {
            escape_next = true;
            i += 1;
            continue;
        }
        if ch == '"' && !in_char {
            in_string = !in_string;
            i += 1;
            continue;
        }
        if in_string {
            i += 1;
            continue;
        }
        // Handle char literals: 'x', '\n', '\x7D', etc.
        if ch == '\'' && !in_char && !in_string {
            // Look ahead for closing quote to confirm it's a char literal
            // Patterns: 'x', '\x', '\xx', '\xxx' (up to 4 chars inside)
            if i + 2 < len && chars[i + 1] != '\\' && chars[i + 2] == '\'' {
                // Simple char literal like '{' or '}'
                i += 3;
                continue;
            }
            if i + 3 < len && chars[i + 1] == '\\' && chars[i + 3] == '\'' {
                // Escaped char literal like '\n' or '\\'
                i += 4;
                continue;
            }
            // Longer escapes like '\x7D' — scan ahead for closing quote
            if i + 1 < len && chars[i + 1] == '\\' {
                let mut j = i + 2;
                while j < len && j < i + 8 && chars[j] != '\'' {
                    j += 1;
                }
                if j < len && chars[j] == '\'' {
                    i = j + 1;
                    continue;
                }
            }
        }
        if ch == '{' {
            opens += 1;
        }
        if ch == '}' {
            closes += 1;
        }
        i += 1;
    }
    (opens, closes)
}

fn parse_wasm_impl_block(
    lines: &[&str],
    impl_line: usize,
    file_path: &str,
    default_feature: Option<&str>,
) -> (Vec<ApiMethod>, usize) {
    let mut methods = Vec::new();
    let mut brace_depth: i32 = 0;
    let mut i = impl_line;
    let mut local_feature: Option<String> = default_feature.map(|s| s.to_string());

    // Find the opening brace of the impl block
    while i < lines.len() {
        let (opens, closes) = count_braces(lines[i]);
        brace_depth += opens as i32 - closes as i32;
        if brace_depth > 0 {
            break;
        }
        i += 1;
    }

    // Now brace_depth should be 1 (inside the impl block)
    i += 1;
    let mut pending_attrs: Vec<String> = Vec::new();

    while i < lines.len() {
        let line = lines[i].trim();

        // Compute depth BEFORE this line's braces
        let depth_before = brace_depth;

        let (opens, closes) = count_braces(lines[i]);
        brace_depth += opens as i32 - closes as i32;

        if brace_depth <= 0 {
            return (methods, i + 1);
        }

        // Only look at lines where depth before is 1 (direct children of impl)
        if depth_before == 1 {
            // Collect attributes
            if line.starts_with('#') {
                pending_attrs.push(line.to_string());
                if line.starts_with("#[cfg(feature") {
                    if let Some(f) = extract_feature_from_cfg(line) {
                        local_feature = Some(f);
                    }
                }
                i += 1;
                continue;
            }

            // Look for pub fn or fn
            if line.starts_with("pub fn ") || line.starts_with("fn ") {
                if let Some(name) = extract_fn_name(line) {
                    // Skip private helper functions
                    if !line.starts_with("pub fn ")
                        && !pending_attrs.iter().any(|a| a.contains("wasm_bindgen"))
                    {
                        pending_attrs.clear();
                        i += 1;
                        continue;
                    }

                    let is_constructor = pending_attrs
                        .iter()
                        .any(|a| a.contains("constructor"));
                    let is_getter = pending_attrs
                        .iter()
                        .any(|a| a.contains("getter"));
                    let is_setter = pending_attrs
                        .iter()
                        .any(|a| a.contains("setter"));
                    let is_static = !line.contains("&self") && !line.contains("&mut self");

                    let method_feature = if pending_attrs.iter().any(|a| a.contains("cfg(feature")) {
                        pending_attrs
                            .iter()
                            .find(|a| a.contains("cfg(feature"))
                            .and_then(|a| extract_feature_from_cfg(a))
                    } else {
                        local_feature.clone()
                    };

                    methods.push(ApiMethod {
                        name,
                        is_constructor,
                        is_static,
                        is_getter,
                        is_setter,
                        feature_gate: method_feature,
                        source_file: file_path.to_string(),
                        source_line: i + 1,
                    });
                }
                pending_attrs.clear();
                local_feature = default_feature.map(|s| s.to_string());
            } else if !line.is_empty()
                && !line.starts_with("//")
                && !line.starts_with("///")
            {
                pending_attrs.clear();
            }
        }

        i += 1;
    }

    (methods, i)
}

// ============================================================================
// Python Parser
// ============================================================================

fn parse_python_surface(base_dir: &str) -> ApiSurface {
    let python_dir = format!("{}/src/python", base_dir);
    let feature_gates = detect_feature_gates(&format!("{}/mod.rs", python_dir));

    let mut classes: Vec<ApiClass> = Vec::new();
    let mut free_functions: Vec<ApiMethod> = Vec::new();

    let entries = match fs::read_dir(&python_dir) {
        Ok(e) => e,
        Err(_) => {
            return ApiSurface {
                target: Target::Python,
                classes,
                free_functions,
            }
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().map(|e| e == "rs").unwrap_or(false) {
            let filename = path.file_stem().unwrap().to_str().unwrap().to_string();
            if filename == "mod" {
                continue;
            }

            let feature = feature_gates.get(&filename).cloned();
            let content = match fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => continue,
            };
            let file_str = format!("src/python/{}.rs", filename);

            parse_python_file(
                &content,
                &file_str,
                feature.as_deref(),
                &mut classes,
                &mut free_functions,
            );
        }
    }

    let merged = merge_classes(classes);

    ApiSurface {
        target: Target::Python,
        classes: merged,
        free_functions,
    }
}

fn normalize_python_class_name(name: &str) -> String {
    let name = if name.starts_with("Py") && name.len() > 2 && name.as_bytes()[2].is_ascii_uppercase()
    {
        &name[2..]
    } else {
        name
    };
    name.to_string()
}

fn parse_python_file(
    content: &str,
    file_path: &str,
    default_feature: Option<&str>,
    classes: &mut Vec<ApiClass>,
    free_functions: &mut Vec<ApiMethod>,
) {
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        let mut local_feature = default_feature.map(|s| s.to_string());
        if line.starts_with("#[cfg(feature") {
            if let Some(f) = extract_feature_from_cfg(line) {
                local_feature = Some(f);
            }
        }

        // Look for #[pyfunction]
        if line == "#[pyfunction]" || line.starts_with("#[pyfunction(") {
            let mut j = i + 1;
            while j < lines.len()
                && (lines[j].trim().is_empty() || lines[j].trim().starts_with('#'))
            {
                j += 1;
            }
            if j < lines.len() {
                let next_line = lines[j].trim();
                if next_line.starts_with("pub fn ") || next_line.starts_with("fn ") {
                    if let Some(name) = extract_fn_name(next_line) {
                        free_functions.push(ApiMethod {
                            name,
                            is_constructor: false,
                            is_static: false,
                            is_getter: false,
                            is_setter: false,
                            feature_gate: local_feature.clone(),
                            source_file: file_path.to_string(),
                            source_line: j + 1,
                        });
                    }
                }
                i = j + 1;
                continue;
            }
        }

        // Look for #[pymethods]
        if line == "#[pymethods]" {
            let mut j = i + 1;
            while j < lines.len()
                && (lines[j].trim().is_empty() || lines[j].trim().starts_with('#'))
            {
                j += 1;
            }
            if j < lines.len() {
                let next_line = lines[j].trim();
                if next_line.starts_with("impl ") {
                    let struct_name = extract_impl_struct_name(next_line);
                    if let Some(struct_name) = struct_name {
                        let canonical = normalize_python_class_name(&struct_name);
                        let (methods, end_line) = parse_python_impl_block(
                            &lines,
                            j,
                            file_path,
                            local_feature.as_deref(),
                        );
                        classes.push(ApiClass {
                            canonical_name: canonical,
                            methods,
                        });
                        i = end_line;
                        continue;
                    }
                }
            }
        }

        i += 1;
    }
}

fn parse_python_impl_block(
    lines: &[&str],
    impl_line: usize,
    file_path: &str,
    default_feature: Option<&str>,
) -> (Vec<ApiMethod>, usize) {
    let mut methods = Vec::new();
    let mut brace_depth: i32 = 0;
    let mut i = impl_line;

    // Find the opening brace of the impl block
    while i < lines.len() {
        let (opens, closes) = count_braces(lines[i]);
        brace_depth += opens as i32 - closes as i32;
        if brace_depth > 0 {
            break;
        }
        i += 1;
    }

    i += 1;
    let mut pending_attrs: Vec<String> = Vec::new();

    while i < lines.len() {
        let line = lines[i].trim();

        let depth_before = brace_depth;

        let (opens, closes) = count_braces(lines[i]);
        brace_depth += opens as i32 - closes as i32;

        if brace_depth <= 0 {
            return (methods, i + 1);
        }

        if depth_before == 1 {
            if line.starts_with('#') {
                pending_attrs.push(line.to_string());
                i += 1;
                continue;
            }

            if line.starts_with("pub fn ") || line.starts_with("fn ") {
                if let Some(name) = extract_fn_name(line) {
                    let is_constructor = pending_attrs.iter().any(|a| a.contains("#[new]"));
                    let is_getter = pending_attrs.iter().any(|a| a.contains("#[getter]"));
                    let is_setter = pending_attrs.iter().any(|a| a.contains("#[setter]"));
                    let is_static = pending_attrs.iter().any(|a| a.contains("#[staticmethod]"))
                        || (!line.contains("&self") && !line.contains("&mut self")
                            && !line.contains("self,") && !line.contains("self)"));

                    let method_feature = pending_attrs
                        .iter()
                        .find(|a| a.contains("cfg(feature"))
                        .and_then(|a| extract_feature_from_cfg(a))
                        .or_else(|| default_feature.map(|s| s.to_string()));

                    methods.push(ApiMethod {
                        name,
                        is_constructor,
                        is_static,
                        is_getter,
                        is_setter,
                        feature_gate: method_feature,
                        source_file: file_path.to_string(),
                        source_line: i + 1,
                    });
                }
                pending_attrs.clear();
            } else if !line.is_empty() && !line.starts_with("//") && !line.starts_with("///") {
                pending_attrs.clear();
            }
        }

        i += 1;
    }

    (methods, i)
}

// ============================================================================
// FFI Parser
// ============================================================================

fn parse_ffi_surface(base_dir: &str) -> ApiSurface {
    let ffi_path = format!("{}/src/ffi.rs", base_dir);
    let content = match fs::read_to_string(&ffi_path) {
        Ok(c) => c,
        Err(_) => {
            return ApiSurface {
                target: Target::Ffi,
                classes: Vec::new(),
                free_functions: Vec::new(),
            }
        }
    };

    let mut class_methods: BTreeMap<String, Vec<ApiMethod>> = BTreeMap::new();
    let mut free_functions: Vec<ApiMethod> = Vec::new();

    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    // Known class prefixes for FFI functions: (prefix, canonical class name)
    // Order matters: longer prefixes first so "definitionregion_" matches before shorter ones
    let known_prefixes: &[(&str, &str)] = &[
        ("definitionregion_", "DefinitionRegion"),
        ("simulationoptions_", "SimulationOptions"),
        ("meshresult_", "MeshResult"),
        ("multimeshresult_", "MultiMeshResult"),
        ("chunkmeshresult_", "ChunkMeshResult"),
        ("meshconfig_", "MeshConfig"),
        ("resourcepack_", "ResourcePack"),
        ("rawmeshexport_", "RawMeshExport"),
        ("mchprsworld_", "MchprsWorld"),
        ("buildingtool_", "BuildingTool"),
        ("blockstate_", "BlockState"),
        ("schematicbuilder_", "SchematicBuilder"),
        ("schematic_", "Schematic"),
        ("shape_", "Shape"),
        ("brush_", "Brush"),
    ];

    let mut current_feature: Option<String> = None;

    while i < lines.len() {
        let line = lines[i].trim();

        // Track feature gates
        if line.starts_with("#[cfg(feature") {
            if let Some(f) = extract_feature_from_cfg(line) {
                current_feature = Some(f);
            }
        }

        // Look for #[no_mangle]
        if line == "#[no_mangle]" {
            let mut j = i + 1;
            let mut fn_feature = current_feature.clone();
            while j < lines.len()
                && (lines[j].trim().is_empty() || lines[j].trim().starts_with('#')
                    || lines[j].trim().starts_with("///"))
            {
                let attr_line = lines[j].trim();
                if attr_line.starts_with("#[cfg(feature") {
                    if let Some(f) = extract_feature_from_cfg(attr_line) {
                        fn_feature = Some(f);
                    }
                }
                j += 1;
            }
            if j < lines.len() {
                let fn_line = lines[j].trim();
                if fn_line.starts_with("pub extern \"C\" fn ") {
                    let after = &fn_line[18..]; // Skip "pub extern \"C\" fn "
                    if let Some(paren) = after.find('(') {
                        let fn_name = after[..paren].trim().to_string();

                        // Classify: which class does this belong to?
                        let mut found_class = false;
                        for &(prefix, class_canonical_str) in known_prefixes {
                            if fn_name.starts_with(prefix) {
                                let class_canonical = class_canonical_str.to_string();
                                let method_name = fn_name[prefix.len()..].to_string();

                                class_methods
                                    .entry(class_canonical.clone())
                                    .or_default()
                                    .push(ApiMethod {
                                        name: method_name,
                                        is_constructor: fn_name.ends_with("_new"),
                                        is_static: true,
                                        is_getter: false,
                                        is_setter: false,
                                        feature_gate: fn_feature.clone(),
                                        source_file: "src/ffi.rs".to_string(),
                                        source_line: j + 1,
                                    });
                                found_class = true;
                                break;
                            }
                        }

                        if !found_class {
                            free_functions.push(ApiMethod {
                                name: fn_name,
                                is_constructor: false,
                                is_static: true,
                                is_getter: false,
                                is_setter: false,
                                feature_gate: fn_feature.clone(),
                                source_file: "src/ffi.rs".to_string(),
                                source_line: j + 1,
                            });
                        }
                    }
                }
            }
            // Reset feature gate after use (it applies to the next item only)
            current_feature = None;
            i = j + 1;
            continue;
        }

        // Reset feature gate if we hit a non-attribute, non-empty line that isn't #[no_mangle]
        if !line.is_empty() && !line.starts_with('#') && !line.starts_with("//") {
            current_feature = None;
        }

        i += 1;
    }

    let classes: Vec<ApiClass> = class_methods
        .into_iter()
        .map(|(name, methods)| ApiClass {
            canonical_name: name,
            methods,
        })
        .collect();

    ApiSurface {
        target: Target::Ffi,
        classes,
        free_functions,
    }
}

// ============================================================================
// Helpers
// ============================================================================

fn extract_fn_name(line: &str) -> Option<String> {
    // Handles: "pub fn foo(..." or "fn foo(..." or "pub fn foo<...>(..."
    let start = if line.starts_with("pub fn ") {
        7
    } else if line.starts_with("fn ") {
        3
    } else {
        return None;
    };

    let rest = &line[start..];
    let end = rest.find(|c: char| c == '(' || c == '<').unwrap_or(rest.len());
    let name = rest[..end].trim().to_string();
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

fn extract_impl_struct_name(line: &str) -> Option<String> {
    // Handles: "impl FooBar {" or "impl FooBar{"
    let line = line.trim();
    if !line.starts_with("impl ") {
        return None;
    }
    let rest = &line[5..];
    let end = rest.find(|c: char| c == '{' || c == ' ' || c == '<').unwrap_or(rest.len());
    let name = rest[..end].trim().to_string();
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

fn extract_feature_from_cfg(line: &str) -> Option<String> {
    // Extract feature name from #[cfg(feature = "foo")]
    if let Some(start) = line.find('"') {
        if let Some(end) = line[start + 1..].find('"') {
            return Some(line[start + 1..start + 1 + end].to_string());
        }
    }
    None
}

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

fn merge_classes(classes: Vec<ApiClass>) -> Vec<ApiClass> {
    let mut map: BTreeMap<String, Vec<ApiMethod>> = BTreeMap::new();
    for class in classes {
        map.entry(class.canonical_name).or_default().extend(class.methods);
    }
    map.into_iter()
        .map(|(name, methods)| ApiClass {
            canonical_name: name,
            methods,
        })
        .collect()
}

// ============================================================================
// Comparison
// ============================================================================

#[derive(Debug)]
struct ParityResult {
    class: String,
    method: String,
    present_in: BTreeSet<Target>,
    missing_from: BTreeSet<Target>,
    excluded_reason: Option<String>,
}

/// Normalize a method name to a canonical form for cross-platform comparison.
/// - Python `#[getter] fn dimensions()` -> canonical "dimensions" (also matches "get_dimensions")
/// - WASM `get_dimensions()` -> canonical "dimensions" (strips get_ prefix for getters)
/// - Python `#[setter] fn name()` -> canonical "name" (setter)
/// This means if Python has `#[getter] dimensions` and WASM has `get_dimensions`, they match.
fn canonical_method_name(name: &str, is_getter: bool, is_setter: bool) -> String {
    if is_getter || is_setter {
        // Python getters/setters: the name is already bare (e.g., "dimensions")
        return name.to_string();
    }
    // Strip get_ prefix for WASM/FFI methods that are effectively getters
    if name.starts_with("get_") && name.len() > 4 {
        return name[4..].to_string();
    }
    name.to_string()
}

fn compare_surfaces(
    wasm: &ApiSurface,
    python: &ApiSurface,
    ffi: &ApiSurface,
    exclusions: &Exclusions,
) -> Vec<ParityResult> {
    let all_targets = [Target::Wasm, Target::Python, Target::Ffi];
    let mut results: Vec<ParityResult> = Vec::new();

    // Build lookup maps: (class, canonical_method) -> set of targets
    // We normalize method names so get_X and #[getter] X both map to "X"
    let mut method_map: BTreeMap<(String, String), BTreeSet<Target>> = BTreeMap::new();

    // Also keep a set of original names per (class, target) to detect
    // when a class has BOTH "get_X" and "X" as separate methods (not just aliases)
    let mut original_names: BTreeMap<(String, Target), BTreeSet<String>> = BTreeMap::new();

    for surface in [wasm, python, ffi] {
        for class in &surface.classes {
            for method in &class.methods {
                let canonical = canonical_method_name(&method.name, method.is_getter, method.is_setter);
                let key = (class.canonical_name.clone(), canonical);
                method_map
                    .entry(key)
                    .or_default()
                    .insert(surface.target);

                original_names
                    .entry((class.canonical_name.clone(), surface.target))
                    .or_default()
                    .insert(method.name.clone());
            }
        }

        // Free functions
        for method in &surface.free_functions {
            let key = (String::new(), method.name.clone());
            method_map
                .entry(key)
                .or_default()
                .insert(surface.target);
        }
    }

    for ((class, method), present) in &method_map {
        let mut missing: BTreeSet<Target> = BTreeSet::new();
        for &t in &all_targets {
            if !present.contains(&t) {
                missing.insert(t);
            }
        }

        if missing.is_empty() {
            results.push(ParityResult {
                class: class.clone(),
                method: method.clone(),
                present_in: present.clone(),
                missing_from: BTreeSet::new(),
                excluded_reason: None,
            });
            continue;
        }

        // Check exclusions for each missing target
        // We check both the canonical name and the get_ prefixed variant
        let mut all_excluded = true;
        let mut any_reason: Option<String> = None;

        for &t in &missing {
            let reason = if class.is_empty() {
                exclusions.is_excluded_free_fn(method, t)
            } else {
                exclusions.is_excluded(class, method, t)
                    .or_else(|| exclusions.is_excluded(class, &format!("get_{}", method), t))
            };
            if let Some(r) = reason {
                if any_reason.is_none() {
                    any_reason = Some(r.to_string());
                }
            } else {
                all_excluded = false;
            }
        }

        results.push(ParityResult {
            class: class.clone(),
            method: method.clone(),
            present_in: present.clone(),
            missing_from: missing,
            excluded_reason: if all_excluded { any_reason } else { None },
        });
    }

    results
}

// ============================================================================
// Stub Generation
// ============================================================================

fn generate_stub(result: &ParityResult, target: Target, surfaces: &[&ApiSurface]) -> String {
    // Find the source method in one of the other surfaces
    let source_info = surfaces.iter().find_map(|s| {
        if result.present_in.contains(&s.target) {
            let class_methods = if result.class.is_empty() {
                s.free_functions.iter().find(|m| m.name == result.method)
            } else {
                s.classes
                    .iter()
                    .find(|c| c.canonical_name == result.class)
                    .and_then(|c| c.methods.iter().find(|m| m.name == result.method))
            };
            class_methods.map(|m| (s.target, m))
        } else {
            None
        }
    });

    let source_comment = if let Some((src_target, method)) = source_info {
        format!(
            "// Source: {}:{}:{}",
            src_target.label(),
            method.source_file,
            method.source_line
        )
    } else {
        "// Source: unknown".to_string()
    };

    match target {
        Target::Ffi => {
            let prefix = if result.class.is_empty() {
                String::new()
            } else {
                format!("{}_", result.class.to_lowercase())
            };
            format!(
                "{}\n#[no_mangle]\npub extern \"C\" fn {}{}() -> c_int {{ todo!() }}",
                source_comment, prefix, result.method
            )
        }
        Target::Python => {
            let class_comment = if result.class.is_empty() {
                "#[pyfunction]".to_string()
            } else {
                format!("// In #[pymethods] impl Py{}", result.class)
            };
            format!(
                "{}\n{}\npub fn {}() -> PyResult<()> {{ todo!() }}",
                source_comment, class_comment, result.method
            )
        }
        Target::Wasm => {
            let class_comment = if result.class.is_empty() {
                "#[wasm_bindgen]".to_string()
            } else {
                format!(
                    "// In #[wasm_bindgen] impl {}Wrapper",
                    result.class
                )
            };
            format!(
                "{}\n{}\npub fn {}() {{ todo!() }}",
                source_comment, class_comment, result.method
            )
        }
    }
}

// ============================================================================
// Output Formatting
// ============================================================================

fn print_report(
    results: &[ParityResult],
    wasm: &ApiSurface,
    python: &ApiSurface,
    ffi: &ApiSurface,
    verbose: bool,
    generate_stubs: bool,
) -> bool {
    println!("=== API Parity Report ===");
    println!();

    // Group by class
    let mut by_class: BTreeMap<String, Vec<&ParityResult>> = BTreeMap::new();
    for r in results {
        by_class.entry(r.class.clone()).or_default().push(r);
    }

    let mut total_methods = 0;
    let mut total_matched = 0;
    let mut total_excluded = 0;
    let mut total_missing = 0;
    let mut has_gaps = false;

    let mut all_stubs: BTreeMap<Target, Vec<String>> = BTreeMap::new();

    for (class, methods) in &by_class {
        let class_label = if class.is_empty() {
            "Free Functions"
        } else {
            class
        };

        let canonical_count = methods.len();
        total_methods += canonical_count;

        // Count per target
        let wasm_count = methods.iter().filter(|m| m.present_in.contains(&Target::Wasm)).count();
        let python_count = methods.iter().filter(|m| m.present_in.contains(&Target::Python)).count();
        let ffi_count = methods.iter().filter(|m| m.present_in.contains(&Target::Ffi)).count();

        let matched: Vec<&&ParityResult> = methods.iter().filter(|m| m.missing_from.is_empty()).collect();
        let excluded: Vec<&&ParityResult> = methods
            .iter()
            .filter(|m| !m.missing_from.is_empty() && m.excluded_reason.is_some())
            .collect();
        let missing: Vec<&&ParityResult> = methods
            .iter()
            .filter(|m| !m.missing_from.is_empty() && m.excluded_reason.is_none())
            .collect();

        total_matched += matched.len();
        total_excluded += excluded.len();
        total_missing += missing.len();

        if !missing.is_empty() {
            has_gaps = true;
        }

        // Only print classes that have gaps or in verbose mode
        if missing.is_empty() && !verbose {
            continue;
        }

        println!(
            "{} ({} canonical methods)",
            class_label, canonical_count
        );
        if !class.is_empty() {
            println!(
                "  WASM: {} | Python: {} | FFI: {}",
                wasm_count, python_count, ffi_count
            );
        }

        // Print missing per target
        for &target in &[Target::Wasm, Target::Python, Target::Ffi] {
            let target_missing: Vec<&str> = missing
                .iter()
                .filter(|m| m.missing_from.contains(&target))
                .map(|m| m.method.as_str())
                .collect();

            if !target_missing.is_empty() {
                println!();
                println!("  Missing from {} ({}):", target.label(), target_missing.len());
                // Print in a comma-separated format, wrapping at ~70 chars
                let mut line = String::from("    ");
                for (idx, name) in target_missing.iter().enumerate() {
                    if idx > 0 {
                        line.push_str(", ");
                    }
                    if line.len() + name.len() > 76 {
                        println!("{}", line);
                        line = String::from("    ");
                    }
                    line.push_str(name);
                }
                if !line.trim().is_empty() {
                    println!("{}", line);
                }

                // Generate stubs
                if generate_stubs {
                    for m in &missing {
                        if m.missing_from.contains(&target) {
                            let stub = generate_stub(m, target, &[wasm, python, ffi]);
                            all_stubs.entry(target).or_default().push(stub);
                        }
                    }
                }
            }
        }

        if verbose {
            if !excluded.is_empty() {
                println!();
                println!("  Excluded ({}):", excluded.len());
                for m in &excluded {
                    let reason = m.excluded_reason.as_deref().unwrap_or("no reason");
                    let targets: Vec<&str> = m.missing_from.iter().map(|t| t.label()).collect();
                    println!(
                        "    {} (not in {}) - {}",
                        m.method,
                        targets.join(", "),
                        reason
                    );
                }
            }

            if !matched.is_empty() {
                println!();
                println!("  Matched ({}):", matched.len());
                let names: Vec<&str> = matched.iter().map(|m| m.method.as_str()).collect();
                let mut line = String::from("    ");
                for (idx, name) in names.iter().enumerate() {
                    if idx > 0 {
                        line.push_str(", ");
                    }
                    if line.len() + name.len() > 76 {
                        println!("{}", line);
                        line = String::from("    ");
                    }
                    line.push_str(name);
                }
                if !line.trim().is_empty() {
                    println!("{}", line);
                }
            }
        }

        println!();
    }

    // Print stubs if requested
    if generate_stubs && !all_stubs.is_empty() {
        println!("=== Generated Stubs ===");
        println!();
        for (target, stubs) in &all_stubs {
            println!("--- {} stubs ---", target.label());
            println!();
            for stub in stubs {
                println!("{}", stub);
                println!();
            }
        }
    }

    println!(
        "Summary: {} methods | {} matched | {} excluded | {} missing",
        total_methods, total_matched, total_excluded, total_missing
    );

    if has_gaps {
        println!("EXIT CODE: 1");
    } else {
        println!("EXIT CODE: 0");
    }

    has_gaps
}

fn print_json_report(results: &[ParityResult]) -> bool {
    let mut has_gaps = false;

    println!("{{");
    println!("  \"methods\": [");

    let mut first = true;
    for r in results {
        if !first {
            println!(",");
        }
        first = false;

        let present: Vec<&str> = r.present_in.iter().map(|t| t.label()).collect();
        let missing: Vec<&str> = r.missing_from.iter().map(|t| t.label()).collect();

        if !r.missing_from.is_empty() && r.excluded_reason.is_none() {
            has_gaps = true;
        }

        let excluded = r.excluded_reason.as_deref().unwrap_or("");

        print!(
            "    {{\"class\": \"{}\", \"method\": \"{}\", \"present\": {:?}, \"missing\": {:?}, \"excluded\": \"{}\"}}",
            r.class, r.method, present, missing, excluded
        );
    }
    println!();
    println!("  ],");

    let total = results.len();
    let matched = results.iter().filter(|r| r.missing_from.is_empty()).count();
    let excluded = results
        .iter()
        .filter(|r| !r.missing_from.is_empty() && r.excluded_reason.is_some())
        .count();
    let missing_count = results
        .iter()
        .filter(|r| !r.missing_from.is_empty() && r.excluded_reason.is_none())
        .count();

    println!(
        "  \"summary\": {{\"total\": {}, \"matched\": {}, \"excluded\": {}, \"missing\": {}}}",
        total, matched, excluded, missing_count
    );
    println!("}}");

    has_gaps
}

// ============================================================================
// Main
// ============================================================================

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let verbose = args.iter().any(|a| a == "--verbose");
    let generate_stubs = args.iter().any(|a| a == "--generate-stubs");
    let json_output = args.iter().any(|a| a == "--json");

    // Determine base directory: try current dir, then script-relative
    let base_dir = find_project_root().unwrap_or_else(|| {
        eprintln!("Error: Could not find project root (looking for src/wasm/ directory)");
        eprintln!("Run this tool from the project root or ensure the project structure exists.");
        std::process::exit(2);
    });

    // Parse exclusions
    let exclusions_path = format!("{}/tools/api_parity_exclusions.txt", base_dir);
    let exclusions = parse_exclusions(&exclusions_path);

    // Parse all three surfaces
    let wasm = parse_wasm_surface(&base_dir);
    let python = parse_python_surface(&base_dir);
    let ffi = parse_ffi_surface(&base_dir);

    // Compare
    let results = compare_surfaces(&wasm, &python, &ffi, &exclusions);

    // Output
    let has_gaps = if json_output {
        print_json_report(&results)
    } else {
        print_report(&results, &wasm, &python, &ffi, verbose, generate_stubs)
    };

    std::process::exit(if has_gaps { 1 } else { 0 });
}

fn find_project_root() -> Option<String> {
    // Try current directory
    if Path::new("src/wasm").is_dir() {
        return Some(".".to_string());
    }

    // Try parent of the binary location
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            // target/check_api_parity -> project root is parent of target
            if let Some(grandparent) = parent.parent() {
                let candidate = grandparent.to_str().unwrap_or(".");
                if Path::new(&format!("{}/src/wasm", candidate)).is_dir() {
                    return Some(candidate.to_string());
                }
            }
            // tools/check_api_parity -> project root is parent
            let candidate = parent.to_str().unwrap_or(".");
            if Path::new(&format!("{}/src/wasm", candidate)).is_dir() {
                return Some(candidate.to_string());
            }
        }
    }

    None
}
