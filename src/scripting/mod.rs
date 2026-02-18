pub mod shared;

#[cfg(feature = "scripting-lua")]
pub mod lua_engine;

#[cfg(feature = "scripting-js")]
pub mod js_engine;

pub use shared::ScriptingSchematic;

/// Auto-detect script engine by file extension and run the script.
/// Returns the resulting schematic if the script produces one.
pub fn run_script(path: &str) -> Result<Option<ScriptingSchematic>, String> {
    let ext = std::path::Path::new(path)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        #[cfg(feature = "scripting-lua")]
        "lua" => lua_engine::run_lua_script(path),

        #[cfg(feature = "scripting-js")]
        "js" => js_engine::run_js_script(path),

        _ => Err(format!("Unsupported script extension: .{}", ext)),
    }
}

/// Run code with a specific engine name ("lua" or "js").
pub fn run_code(engine: &str, code: &str) -> Result<Option<ScriptingSchematic>, String> {
    match engine {
        #[cfg(feature = "scripting-lua")]
        "lua" => lua_engine::run_lua_code(code),

        #[cfg(feature = "scripting-js")]
        "js" => js_engine::run_js_code(code),

        _ => Err(format!("Unsupported scripting engine: {}", engine)),
    }
}
