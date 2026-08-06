//! Resource-pack discovery, shared by the TUI's GPU view and `mesh`.
//!
//! Best first: `NUCLEATION_PACK`, then the newest installed Minecraft
//! client jar — a jar *is* a resource pack for meshing purposes, and both
//! the vanilla launcher layout (`versions/<v>/<v>.jar`) and the
//! Prism/MultiMC layout
//! (`libraries/com/mojang/minecraft/<v>/minecraft-<v>-client.jar`)
//! are scanned; a Prism machine keeps no jars in the vanilla tree at all.

pub(crate) fn discover_pack() -> Option<std::path::PathBuf> {
    if let Ok(pack) = std::env::var("NUCLEATION_PACK") {
        let path = std::path::PathBuf::from(pack);
        return path.exists().then_some(path);
    }
    let home = std::path::PathBuf::from(std::env::var_os("HOME")?);
    let app_support = home.join("Library/Application Support");
    let mut vanilla_roots = vec![
        app_support.join("minecraft/versions"),
        home.join(".minecraft/versions"),
    ];
    let mut mojang_lib_roots = vec![
        app_support.join("PrismLauncher/libraries/com/mojang/minecraft"),
        app_support.join("MultiMC/libraries/com/mojang/minecraft"),
        home.join(".local/share/PrismLauncher/libraries/com/mojang/minecraft"),
        home.join(".local/share/multimc/libraries/com/mojang/minecraft"),
    ];
    if let Some(appdata) = std::env::var_os("APPDATA") {
        let appdata = std::path::PathBuf::from(appdata);
        vanilla_roots.push(appdata.join(".minecraft/versions"));
        mojang_lib_roots.push(appdata.join("PrismLauncher/libraries/com/mojang/minecraft"));
    }
    let mut newest: Option<(std::time::SystemTime, std::path::PathBuf)> = None;
    let mut consider = |jar: std::path::PathBuf| {
        let Ok(meta) = jar.metadata() else { return };
        let Ok(modified) = meta.modified() else {
            return;
        };
        if newest.as_ref().is_none_or(|(t, _)| modified > *t) {
            newest = Some((modified, jar));
        }
    };
    for root in vanilla_roots {
        let Ok(entries) = std::fs::read_dir(&root) else {
            continue;
        };
        for entry in entries.flatten() {
            let dir = entry.path();
            let Some(name) = dir.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            consider(dir.join(format!("{name}.jar")));
        }
    }
    for root in mojang_lib_roots {
        let Ok(entries) = std::fs::read_dir(&root) else {
            continue;
        };
        for entry in entries.flatten() {
            let dir = entry.path();
            let Some(version) = dir.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            consider(dir.join(format!("minecraft-{version}-client.jar")));
        }
    }
    newest.map(|(_, jar)| jar)
}
