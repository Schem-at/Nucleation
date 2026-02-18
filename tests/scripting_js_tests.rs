#![cfg(feature = "scripting-js")]

use nucleation::scripting::js_engine::run_js_code;

#[test]
fn test_js_create_schematic() {
    let result = run_js_code(
        r#"
        let s = new Schematic("JsTest");
        s.set_block(0, 0, 0, "minecraft:stone");
        result = s;
    "#,
    )
    .expect("js should not error");
    let s = result.expect("should return a schematic");
    assert_eq!(s.get_name(), "JsTest");
    assert!(s.get_block(0, 0, 0).unwrap().contains("stone"));
}

#[test]
fn test_js_fill_cuboid() {
    let result = run_js_code(
        r#"
        let s = new Schematic("CuboidTest");
        s.fill_cuboid(0, 0, 0, 4, 4, 4, "minecraft:stone");
        if (s.get_block_count() <= 0) throw new Error("block count should be > 0");
        let b = s.get_block(2, 2, 2);
        if (!b || !b.includes("stone")) throw new Error("interior block should be stone");
        result = s;
    "#,
    )
    .expect("js should not error");
    assert!(result.is_some());
}

#[test]
fn test_js_fill_sphere() {
    let result = run_js_code(
        r#"
        let s = new Schematic("SphereTest");
        s.fill_sphere(10, 10, 10, 5.0, "minecraft:glass");
        let b = s.get_block(10, 10, 10);
        if (!b || !b.includes("glass")) throw new Error("center should be glass");
        result = s;
    "#,
    )
    .expect("js should not error");
    assert!(result.is_some());
}

#[test]
fn test_js_metadata() {
    let result = run_js_code(
        r#"
        let s = new Schematic();
        s.name = "Named";
        s.author = "Author";
        s.description = "Desc";
        if (s.name !== "Named") throw new Error("name mismatch: " + s.name);
        if (s.author !== "Author") throw new Error("author mismatch");
        if (s.description !== "Desc") throw new Error("description mismatch");
        result = s;
    "#,
    )
    .expect("js should not error");
    let s = result.expect("should return a schematic");
    assert_eq!(s.get_name(), "Named");
    assert_eq!(s.get_author(), "Author");
    assert_eq!(s.get_description(), "Desc");
}

#[test]
fn test_js_dimensions() {
    run_js_code(
        r#"
        let s = new Schematic("DimTest");
        s.set_block(0, 0, 0, "minecraft:stone");
        s.set_block(5, 3, 7, "minecraft:dirt");
        let dims = s.get_dimensions();
        if (dims.width < 6) throw new Error("width should be >= 6, got " + dims.width);
        if (dims.height < 4) throw new Error("height should be >= 4, got " + dims.height);
        if (dims.depth < 8) throw new Error("depth should be >= 8, got " + dims.depth);
    "#,
    )
    .expect("js should not error");
}

#[test]
fn test_js_transformations() {
    run_js_code(
        r#"
        let s = new Schematic("FlipTest");
        s.set_block(0, 0, 0, "minecraft:stone");
        s.set_block(1, 0, 0, "minecraft:dirt");
        let before = s.get_block_count();
        s.flip_x();
        let after = s.get_block_count();
        if (before !== after) throw new Error("block count changed after flip");
    "#,
    )
    .expect("js should not error");
}

#[test]
fn test_js_export() {
    run_js_code(
        r#"
        let s = new Schematic("ExportTest");
        s.set_block(0, 0, 0, "minecraft:stone");
        let bytes = s.to_schematic();
        if (bytes.length === 0) throw new Error("exported bytes should be non-empty");
    "#,
    )
    .expect("js should not error");
}

#[test]
fn test_js_save_to_file() {
    let tmp = std::env::temp_dir().join("js_test_output.schem");
    let path = tmp.to_str().unwrap().replace('\\', "/");
    let code = format!(
        r#"
        let s = new Schematic("SaveTest");
        s.set_block(0, 0, 0, "minecraft:stone");
        s.save_to_file("{}");
    "#,
        path
    );
    run_js_code(&code).expect("js should not error");
    assert!(tmp.exists(), "output file should exist");
    std::fs::remove_file(&tmp).ok();
}

#[test]
fn test_js_error_handling() {
    let result = run_js_code(r#"throw new Error("intentional error")"#);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("intentional error"));
}

#[test]
fn test_js_no_imports_needed() {
    // new Schematic() should work globally without imports
    let result = run_js_code(
        r#"
        let s = new Schematic("NoImport");
        result = s;
    "#,
    )
    .expect("js should not error");
    let s = result.expect("should return a schematic");
    assert_eq!(s.get_name(), "NoImport");
}

#[test]
fn test_js_no_result_returns_none() {
    let result = run_js_code(
        r#"
        let s = new Schematic("Temp");
        s.set_block(0, 0, 0, "minecraft:stone");
        // no result = s;
    "#,
    )
    .expect("js should not error");
    assert!(result.is_none());
}
