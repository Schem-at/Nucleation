use crate::scripting::shared::ScriptingSchematic;
use rquickjs::{
    class::Trace, CatchResultExt, Class, Context, Ctx, Function, JsLifetime, Result as JsResult,
    Runtime,
};

#[rquickjs::class]
pub struct JsSchematic {
    inner: ScriptingSchematic,
}

// ScriptingSchematic contains no JS values, so tracing is a no-op.
impl<'js> Trace<'js> for JsSchematic {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

// JsSchematic has no JS lifetime — it's a purely Rust-owned type.
unsafe impl<'js> JsLifetime<'js> for JsSchematic {
    type Changed<'to> = JsSchematic;
}

fn make_js_err(msg: &str) -> rquickjs::Error {
    rquickjs::Error::IntoJs {
        from: "Rust",
        to: "JS",
        message: Some(msg.to_string()),
    }
}

#[rquickjs::methods]
impl JsSchematic {
    // -- Constructor --
    #[qjs(constructor)]
    pub fn new(name: rquickjs::function::Opt<String>) -> Self {
        Self {
            inner: ScriptingSchematic::new(name.0),
        }
    }

    // -- Metadata properties --
    #[qjs(get, rename = "name")]
    pub fn get_name(&self) -> String {
        self.inner.get_name()
    }

    #[qjs(set, rename = "name")]
    pub fn set_name(&mut self, name: String) {
        self.inner.set_name(&name);
    }

    #[qjs(get, rename = "author")]
    pub fn get_author(&self) -> String {
        self.inner.get_author()
    }

    #[qjs(set, rename = "author")]
    pub fn set_author(&mut self, author: String) {
        self.inner.set_author(&author);
    }

    #[qjs(get, rename = "description")]
    pub fn get_description(&self) -> String {
        self.inner.get_description()
    }

    #[qjs(set, rename = "description")]
    pub fn set_description(&mut self, desc: String) {
        self.inner.set_description(&desc);
    }

    // -- Blocks --
    pub fn set_block(&mut self, x: i32, y: i32, z: i32, name: String) {
        self.inner.set_block(x, y, z, &name);
    }

    pub fn get_block(&self, x: i32, y: i32, z: i32) -> Option<String> {
        self.inner.get_block(x, y, z)
    }

    // -- Building --
    pub fn fill_cuboid(
        &mut self,
        min_x: i32,
        min_y: i32,
        min_z: i32,
        max_x: i32,
        max_y: i32,
        max_z: i32,
        block: String,
    ) {
        self.inner
            .fill_cuboid((min_x, min_y, min_z), (max_x, max_y, max_z), &block);
    }

    pub fn fill_sphere(&mut self, cx: i32, cy: i32, cz: i32, radius: f64, block: String) {
        self.inner.fill_sphere((cx, cy, cz), radius, &block);
    }

    // -- Info --
    pub fn get_dimensions<'js>(&self, ctx: Ctx<'js>) -> JsResult<rquickjs::Object<'js>> {
        let (w, h, d) = self.inner.get_dimensions();
        let obj = rquickjs::Object::new(ctx)?;
        obj.set("width", w)?;
        obj.set("height", h)?;
        obj.set("depth", d)?;
        Ok(obj)
    }

    pub fn get_block_count(&self) -> i32 {
        self.inner.get_block_count()
    }

    pub fn get_volume(&self) -> i32 {
        self.inner.get_volume()
    }

    // -- Transforms --
    pub fn flip_x(&mut self) {
        self.inner.flip_x();
    }

    pub fn flip_y(&mut self) {
        self.inner.flip_y();
    }

    pub fn flip_z(&mut self) {
        self.inner.flip_z();
    }

    pub fn rotate_x(&mut self, degrees: i32) {
        self.inner.rotate_x(degrees);
    }

    pub fn rotate_y(&mut self, degrees: i32) {
        self.inner.rotate_y(degrees);
    }

    pub fn rotate_z(&mut self, degrees: i32) {
        self.inner.rotate_z(degrees);
    }

    // -- Export --
    pub fn to_schematic(&self) -> rquickjs::Result<Vec<u8>> {
        self.inner.to_schematic().map_err(|e| make_js_err(&e))
    }

    pub fn to_litematic(&self) -> rquickjs::Result<Vec<u8>> {
        self.inner.to_litematic().map_err(|e| make_js_err(&e))
    }

    pub fn save_as(&self, format: String) -> rquickjs::Result<Vec<u8>> {
        self.inner.save_as(&format).map_err(|e| make_js_err(&e))
    }

    pub fn save_to_file(&self, path: String) -> rquickjs::Result<()> {
        self.inner.save_to_file(&path).map_err(|e| make_js_err(&e))
    }

    // -- Iteration --
    pub fn get_all_blocks<'js>(&self, ctx: Ctx<'js>) -> JsResult<rquickjs::Array<'js>> {
        let blocks = self.inner.get_all_blocks();
        let arr = rquickjs::Array::new(ctx.clone())?;
        for (i, (x, y, z, name)) in blocks.iter().enumerate() {
            let obj = rquickjs::Object::new(ctx.clone())?;
            obj.set("x", *x)?;
            obj.set("y", *y)?;
            obj.set("z", *z)?;
            obj.set("name", name.as_str())?;
            arr.set(i, obj)?;
        }
        Ok(arr)
    }

    pub fn get_region_names<'js>(&self, ctx: Ctx<'js>) -> JsResult<rquickjs::Array<'js>> {
        let names = self.inner.get_region_names();
        let arr = rquickjs::Array::new(ctx)?;
        for (i, name) in names.iter().enumerate() {
            arr.set(i, name.as_str())?;
        }
        Ok(arr)
    }
}

/// Register `Schematic` class and a `Schematic.load(path)` static method on the global.
fn setup_js(ctx: &Ctx<'_>) -> JsResult<()> {
    Class::<JsSchematic>::define(&ctx.globals())?;

    // Add a static `load` function on the Schematic constructor
    let schematic_ctor: Function = ctx.globals().get("JsSchematic")?;
    schematic_ctor.set(
        "load",
        Function::new(
            ctx.clone(),
            |path: String| -> rquickjs::Result<JsSchematic> {
                let ss = ScriptingSchematic::from_file(&path).map_err(|e| make_js_err(&e))?;
                Ok(JsSchematic { inner: ss })
            },
        )?,
    )?;

    // Alias: expose as `Schematic` too (the class registers as `JsSchematic`)
    let ctor: rquickjs::Value = ctx.globals().get("JsSchematic")?;
    ctx.globals().set("Schematic", ctor)?;

    Ok(())
}

/// Run a JS script file.
pub fn run_js_script(path: &str) -> Result<Option<ScriptingSchematic>, String> {
    let code =
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read JS script: {}", e))?;
    run_js_code(&code)
}

/// Run JS source code.
pub fn run_js_code(code: &str) -> Result<Option<ScriptingSchematic>, String> {
    let rt = Runtime::new().map_err(|e| format!("JS runtime error: {}", e))?;
    let ctx = Context::full(&rt).map_err(|e| format!("JS context error: {}", e))?;

    ctx.with(|ctx| {
        setup_js(&ctx).map_err(|e| format!("JS setup error: {}", e))?;

        // Pre-declare `result` so scripts can assign to it without `var`/`let`
        ctx.globals()
            .set("result", rquickjs::Value::new_undefined(ctx.clone()))
            .map_err(|e| format!("JS setup error: {}", e))?;

        ctx.eval::<(), _>(code)
            .catch(&ctx)
            .map_err(|e| format!("JS execution error: {}", e))?;

        // Try to extract a `result` global if set
        let result: Option<Class<JsSchematic>> = ctx.globals().get("result").ok();
        match result {
            Some(cls) => {
                let borrow = cls.borrow();
                let cloned = borrow.inner.inner.clone();
                drop(borrow);
                Ok(Some(ScriptingSchematic { inner: cloned }))
            }
            None => Ok(None),
        }
    })
}
