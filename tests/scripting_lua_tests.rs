#![cfg(feature = "scripting-lua")]

use nucleation::scripting::lua_engine::run_lua_code;

#[test]
fn test_lua_create_schematic() {
    let result = run_lua_code(
        r#"
        local s = Schematic.new("LuaTest")
        s:set_block(0, 0, 0, "minecraft:stone")
        result = s
    "#,
    )
    .expect("lua should not error");
    let s = result.expect("should return a schematic");
    assert_eq!(s.get_name(), "LuaTest");
    assert!(s.get_block(0, 0, 0).unwrap().contains("stone"));
}

#[test]
fn test_lua_fill_cuboid() {
    let result = run_lua_code(
        r#"
        local s = Schematic.new("CuboidTest")
        s:fill_cuboid(0, 0, 0, 4, 4, 4, "minecraft:stone")
        result = s
    "#,
    )
    .expect("lua should not error");
    let s = result.expect("should return a schematic");
    assert!(s.get_block_count() > 0);
    assert!(s.get_block(2, 2, 2).unwrap().contains("stone"));
}

#[test]
fn test_lua_fill_sphere() {
    let result = run_lua_code(
        r#"
        local s = Schematic.new("SphereTest")
        s:fill_sphere(10, 10, 10, 5.0, "minecraft:glass")
        result = s
    "#,
    )
    .expect("lua should not error");
    let s = result.expect("should return a schematic");
    // Center block should be filled
    assert!(s.get_block(10, 10, 10).unwrap().contains("glass"));
}

#[test]
fn test_lua_metadata() {
    let result = run_lua_code(
        r#"
        local s = Schematic.new()
        s:set_name("Named")
        s:set_author("Author")
        s:set_description("Desc")
        assert(s:get_name() == "Named", "name mismatch")
        assert(s:get_author() == "Author", "author mismatch")
        assert(s:get_description() == "Desc", "description mismatch")
        result = s
    "#,
    )
    .expect("lua should not error");
    let s = result.expect("should return a schematic");
    assert_eq!(s.get_name(), "Named");
    assert_eq!(s.get_author(), "Author");
    assert_eq!(s.get_description(), "Desc");
}

#[test]
fn test_lua_dimensions() {
    let result = run_lua_code(
        r#"
        local s = Schematic.new("DimTest")
        s:set_block(0, 0, 0, "minecraft:stone")
        s:set_block(5, 3, 7, "minecraft:dirt")
        local dims = s:get_dimensions()
        assert(dims.width >= 6, "width should be >= 6, got " .. dims.width)
        assert(dims.height >= 4, "height should be >= 4, got " .. dims.height)
        assert(dims.depth >= 8, "depth should be >= 8, got " .. dims.depth)
        result = s
    "#,
    )
    .expect("lua should not error");
    assert!(result.is_some());
}

#[test]
fn test_lua_transformations() {
    let result = run_lua_code(
        r#"
        local s = Schematic.new("FlipTest")
        s:set_block(0, 0, 0, "minecraft:stone")
        s:set_block(1, 0, 0, "minecraft:dirt")
        local count_before = s:get_block_count()
        s:flip_x()
        local count_after = s:get_block_count()
        assert(count_before == count_after, "block count changed after flip")
        result = s
    "#,
    )
    .expect("lua should not error");
    assert!(result.is_some());
}

#[test]
fn test_lua_export_schematic() {
    let result = run_lua_code(
        r#"
        local s = Schematic.new("ExportTest")
        s:set_block(0, 0, 0, "minecraft:stone")
        local bytes = s:to_schematic()
        assert(#bytes > 0, "exported bytes should be non-empty")
        result = s
    "#,
    )
    .expect("lua should not error");
    assert!(result.is_some());
}

#[test]
fn test_lua_get_all_blocks() {
    let result = run_lua_code(
        r#"
        local s = Schematic.new("BlocksTest")
        s:set_block(0, 0, 0, "minecraft:stone")
        s:set_block(1, 0, 0, "minecraft:dirt")
        local blocks = s:get_all_blocks()
        assert(#blocks >= 2, "should have at least 2 blocks, got " .. #blocks)
        result = s
    "#,
    )
    .expect("lua should not error");
    assert!(result.is_some());
}

#[test]
fn test_lua_save_to_file() {
    let tmp = std::env::temp_dir().join("lua_test_output.schem");
    let path = tmp.to_str().unwrap().replace('\\', "/");
    let code = format!(
        r#"
        local s = Schematic.new("SaveTest")
        s:set_block(0, 0, 0, "minecraft:stone")
        s:save_to_file("{}")
    "#,
        path
    );
    run_lua_code(&code).expect("lua should not error");
    assert!(tmp.exists(), "output file should exist");
    std::fs::remove_file(&tmp).ok();
}

#[test]
fn test_lua_error_handling() {
    let result = run_lua_code(r#"error("intentional error")"#);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("intentional error"));
}

#[test]
fn test_lua_no_imports_needed() {
    // Schematic.new() should work without any require()
    let result = run_lua_code(
        r#"
        local s = Schematic.new("NoImport")
        result = s
    "#,
    )
    .expect("lua should not error");
    let s = result.expect("should return a schematic");
    assert_eq!(s.get_name(), "NoImport");
}

#[test]
fn test_lua_no_result_returns_none() {
    let result = run_lua_code(
        r#"
        local s = Schematic.new("Temp")
        s:set_block(0, 0, 0, "minecraft:stone")
        -- no result = s
    "#,
    )
    .expect("lua should not error");
    assert!(result.is_none());
}
