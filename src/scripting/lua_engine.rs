use crate::scripting::shared::ScriptingSchematic;
use mlua::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

/// Lua-side handle to a ScriptingSchematic, using Rc<RefCell<>> for interior mutability.
struct LuaSchematic(Rc<RefCell<ScriptingSchematic>>);

impl LuaUserData for LuaSchematic {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        // -- Metadata getters/setters --
        methods.add_method("get_name", |_, this, ()| Ok(this.0.borrow().get_name()));
        methods.add_method_mut("set_name", |_, this, name: String| {
            this.0.borrow_mut().set_name(&name);
            Ok(())
        });
        methods.add_method("get_author", |_, this, ()| Ok(this.0.borrow().get_author()));
        methods.add_method_mut("set_author", |_, this, author: String| {
            this.0.borrow_mut().set_author(&author);
            Ok(())
        });
        methods.add_method("get_description", |_, this, ()| {
            Ok(this.0.borrow().get_description())
        });
        methods.add_method_mut("set_description", |_, this, desc: String| {
            this.0.borrow_mut().set_description(&desc);
            Ok(())
        });

        // -- Blocks --
        methods.add_method_mut(
            "set_block",
            |_, this, (x, y, z, name): (i32, i32, i32, String)| {
                this.0.borrow_mut().set_block(x, y, z, &name);
                Ok(())
            },
        );
        methods.add_method("get_block", |_, this, (x, y, z): (i32, i32, i32)| {
            Ok(this.0.borrow().get_block(x, y, z))
        });

        // -- Building --
        methods.add_method_mut(
            "fill_cuboid",
            |_,
             this,
             (min_x, min_y, min_z, max_x, max_y, max_z, block): (
                i32,
                i32,
                i32,
                i32,
                i32,
                i32,
                String,
            )| {
                this.0.borrow_mut().fill_cuboid(
                    (min_x, min_y, min_z),
                    (max_x, max_y, max_z),
                    &block,
                );
                Ok(())
            },
        );
        methods.add_method_mut(
            "fill_sphere",
            |_, this, (cx, cy, cz, radius, block): (i32, i32, i32, f64, String)| {
                this.0
                    .borrow_mut()
                    .fill_sphere((cx, cy, cz), radius, &block);
                Ok(())
            },
        );

        // -- Info --
        methods.add_method("get_dimensions", |lua, this, ()| {
            let (w, h, d) = this.0.borrow().get_dimensions();
            let t = lua.create_table()?;
            t.set("width", w)?;
            t.set("height", h)?;
            t.set("depth", d)?;
            Ok(t)
        });
        methods.add_method("get_block_count", |_, this, ()| {
            Ok(this.0.borrow().get_block_count())
        });
        methods.add_method("get_volume", |_, this, ()| Ok(this.0.borrow().get_volume()));

        // -- Transforms --
        methods.add_method_mut("flip_x", |_, this, ()| {
            this.0.borrow_mut().flip_x();
            Ok(())
        });
        methods.add_method_mut("flip_y", |_, this, ()| {
            this.0.borrow_mut().flip_y();
            Ok(())
        });
        methods.add_method_mut("flip_z", |_, this, ()| {
            this.0.borrow_mut().flip_z();
            Ok(())
        });
        methods.add_method_mut("rotate_x", |_, this, degrees: i32| {
            this.0
                .borrow_mut()
                .rotate_x(degrees)
                .map_err(LuaError::external)?;
            Ok(())
        });
        methods.add_method_mut("rotate_y", |_, this, degrees: i32| {
            this.0
                .borrow_mut()
                .rotate_y(degrees)
                .map_err(LuaError::external)?;
            Ok(())
        });
        methods.add_method_mut("rotate_z", |_, this, degrees: i32| {
            this.0
                .borrow_mut()
                .rotate_z(degrees)
                .map_err(LuaError::external)?;
            Ok(())
        });

        // -- Export --
        methods.add_method("to_schematic", |lua, this, ()| {
            let bytes = this.0.borrow().to_schematic().map_err(LuaError::external)?;
            lua.create_string(&bytes)
        });
        methods.add_method("to_litematic", |lua, this, ()| {
            let bytes = this.0.borrow().to_litematic().map_err(LuaError::external)?;
            lua.create_string(&bytes)
        });
        methods.add_method("save_as", |lua, this, format: String| {
            let bytes = this
                .0
                .borrow()
                .save_as(&format)
                .map_err(LuaError::external)?;
            lua.create_string(&bytes)
        });
        methods.add_method("save_to_file", |_, this, path: String| {
            this.0
                .borrow()
                .save_to_file(&path)
                .map_err(LuaError::external)?;
            Ok(())
        });

        // -- Iteration --
        methods.add_method("get_all_blocks", |lua, this, ()| {
            let blocks = this.0.borrow().get_all_blocks();
            let table = lua.create_table()?;
            for (i, (x, y, z, name)) in blocks.iter().enumerate() {
                let entry = lua.create_table()?;
                entry.set("x", *x)?;
                entry.set("y", *y)?;
                entry.set("z", *z)?;
                entry.set("name", name.as_str())?;
                table.set(i + 1, entry)?;
            }
            Ok(table)
        });
        methods.add_method("get_region_names", |lua, this, ()| {
            let names = this.0.borrow().get_region_names();
            let table = lua.create_table()?;
            for (i, name) in names.iter().enumerate() {
                table.set(i + 1, name.as_str())?;
            }
            Ok(table)
        });
    }
}

/// Set up the Lua VM with the `Schematic` global table providing `new()` and `load()`.
fn setup_lua(lua: &Lua) -> LuaResult<()> {
    let schematic_table = lua.create_table()?;

    schematic_table.set(
        "new",
        lua.create_function(|_, name: Option<String>| {
            let ss = ScriptingSchematic::new(name);
            Ok(LuaSchematic(Rc::new(RefCell::new(ss))))
        })?,
    )?;

    schematic_table.set(
        "load",
        lua.create_function(|_, path: String| {
            let ss = ScriptingSchematic::from_file(&path).map_err(LuaError::external)?;
            Ok(LuaSchematic(Rc::new(RefCell::new(ss))))
        })?,
    )?;

    lua.globals().set("Schematic", schematic_table)?;

    // -- Palette helpers (globals, mirroring the bridge's Palette surface) --
    lua.globals().set(
        "palette_gradient_ids",
        lua.create_function(
            |lua,
             (name, r1, g1, b1, r2, g2, b2, steps): (
                String,
                u8,
                u8,
                u8,
                u8,
                u8,
                u8,
                usize,
            )| {
                let ids =
                    crate::scripting::shared::palette_gradient_ids(&name, (r1, g1, b1), (r2, g2, b2), steps)
                        .map_err(LuaError::external)?;
                let table = lua.create_table()?;
                for (i, id) in ids.iter().enumerate() {
                    table.set(i + 1, id.as_str())?;
                }
                Ok(table)
            },
        )?,
    )?;

    lua.globals().set(
        "palette_block_ids",
        lua.create_function(|lua, name: String| {
            let ids =
                crate::scripting::shared::palette_block_ids(&name).map_err(LuaError::external)?;
            let table = lua.create_table()?;
            for (i, id) in ids.iter().enumerate() {
                table.set(i + 1, id.as_str())?;
            }
            Ok(table)
        })?,
    )?;

    lua.globals().set(
        "palette_closest_block",
        lua.create_function(|_, (name, r, g, b): (String, u8, u8, u8)| {
            crate::scripting::shared::palette_closest_block(&name, r, g, b)
                .map_err(LuaError::external)
        })?,
    )?;

    Ok(())
}

/// Run a Lua script file. If the script assigns to the global `result`, its
/// inner ScriptingSchematic is returned.
pub fn run_lua_script(path: &str) -> Result<Option<ScriptingSchematic>, String> {
    let code =
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read Lua script: {}", e))?;
    run_lua_code(&code)
}

/// Run Lua source code. If the code assigns to the global `result`, its
/// inner ScriptingSchematic is extracted and returned.
pub fn run_lua_code(code: &str) -> Result<Option<ScriptingSchematic>, String> {
    let lua = Lua::new();
    setup_lua(&lua).map_err(|e| format!("Lua setup error: {}", e))?;

    lua.load(code)
        .exec()
        .map_err(|e| format!("Lua execution error: {}", e))?;

    // Check if the script set a global `result`
    let result: Option<mlua::AnyUserData> = lua.globals().get("result").ok();

    match result {
        Some(ud) => {
            let ls = ud
                .borrow::<LuaSchematic>()
                .map_err(|e| format!("Failed to extract result: {}", e))?;
            let cloned = ls.0.borrow().inner.clone();
            drop(ls);
            Ok(Some(ScriptingSchematic { inner: cloned }))
        }
        None => Ok(None),
    }
}
