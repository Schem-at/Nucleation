use super::*;

// =============================================================================
// Simulation FFI Bindings (feature-gated)
// =============================================================================

#[cfg(feature = "simulation")]
pub mod simulation_ffi {
    use super::super::definition_region::DefinitionRegionWrapper;
    use super::*;
    use crate::simulation::circuit_builder::CircuitBuilder;
    use crate::simulation::typed_executor::{
        ExecutionMode, ExecutionResult, IoLayout, IoLayoutBuilder, IoType, LayoutFunction,
        LayoutInfo, OutputCondition, SortStrategy, StateMode, TypedCircuitExecutor, Value,
    };
    use crate::simulation::CustomIoChange;
    use crate::simulation::{MchprsWorld, SimulationOptions};
    use mchprs_blocks::BlockPos;

    // --- Wrapper Structs ---

    pub struct ValueWrapper(Value);
    pub struct IoTypeWrapper(IoType);
    pub struct LayoutFunctionWrapper(LayoutFunction);
    pub struct OutputConditionWrapper(OutputCondition);
    pub struct ExecutionModeWrapper(ExecutionMode);
    pub struct SortStrategyWrapper(SortStrategy);
    pub struct IoLayoutBuilderWrapper(Option<IoLayoutBuilder>);
    pub struct IoLayoutWrapper(IoLayout);
    pub struct CircuitBuilderWrapper(Option<CircuitBuilder>);
    pub struct TypedCircuitExecutorWrapper(*mut TypedCircuitExecutor);

    /// Creates a new MchprsWorld from a schematic with default options.
    /// Returns null on error. Caller must free with `mchprs_world_free`.
    #[no_mangle]
    pub extern "C" fn mchprs_world_new(
        schematic: *const SchematicWrapper,
    ) -> *mut MchprsWorldWrapper {
        if schematic.is_null() {
            return ptr::null_mut();
        }
        let s = unsafe { &*(*schematic).0 };
        match MchprsWorld::new(s.clone()) {
            Ok(world) => {
                let w = Box::into_raw(Box::new(world));
                Box::into_raw(Box::new(MchprsWorldWrapper(w)))
            }
            Err(_) => ptr::null_mut(),
        }
    }

    /// Creates a new MchprsWorld from a schematic with options.
    /// `optimize`: 0=false, non-zero=true
    /// `io_only`: 0=false, non-zero=true
    /// Returns null on error. Caller must free with `mchprs_world_free`.
    #[no_mangle]
    pub extern "C" fn mchprs_world_new_with_options(
        schematic: *const SchematicWrapper,
        optimize: c_int,
        io_only: c_int,
    ) -> *mut MchprsWorldWrapper {
        if schematic.is_null() {
            return ptr::null_mut();
        }
        let s = unsafe { &*(*schematic).0 };
        let options = SimulationOptions {
            optimize: optimize != 0,
            io_only: io_only != 0,
            custom_io: Vec::new(),
        };
        match MchprsWorld::with_options(s.clone(), options) {
            Ok(world) => {
                let w = Box::into_raw(Box::new(world));
                Box::into_raw(Box::new(MchprsWorldWrapper(w)))
            }
            Err(_) => ptr::null_mut(),
        }
    }

    /// Creates a new MchprsWorld from a Schematic with custom IO positions.
    ///
    /// `custom_io_positions`: pointer to array of [x, y, z, x, y, z, ...] i32 values
    /// `custom_io_count`: number of positions (i.e., array length / 3)
    /// Returns null on error. Caller must free with `mchprs_world_free`.
    #[no_mangle]
    pub extern "C" fn mchprs_world_new_with_custom_io(
        schematic: *const SchematicWrapper,
        optimize: c_int,
        io_only: c_int,
        custom_io_positions: *const c_int,
        custom_io_count: c_int,
    ) -> *mut MchprsWorldWrapper {
        if schematic.is_null() {
            return ptr::null_mut();
        }
        let s = unsafe { &*(*schematic).0 };
        let mut custom_io = Vec::new();
        if !custom_io_positions.is_null() && custom_io_count > 0 {
            let coords = unsafe {
                std::slice::from_raw_parts(custom_io_positions, custom_io_count as usize * 3)
            };
            for chunk in coords.chunks_exact(3) {
                custom_io.push(BlockPos::new(chunk[0], chunk[1], chunk[2]));
            }
        }
        let options = SimulationOptions {
            optimize: optimize != 0,
            io_only: io_only != 0,
            custom_io,
        };
        match MchprsWorld::with_options(s.clone(), options) {
            Ok(world) => {
                let w = Box::into_raw(Box::new(world));
                Box::into_raw(Box::new(MchprsWorldWrapper(w)))
            }
            Err(_) => ptr::null_mut(),
        }
    }

    /// Frees a MchprsWorld.
    #[no_mangle]
    pub extern "C" fn mchprs_world_free(world: *mut MchprsWorldWrapper) {
        if !world.is_null() {
            unsafe {
                let wrapper = Box::from_raw(world);
                let _ = Box::from_raw(wrapper.0);
            }
        }
    }

    /// Advances the simulation by the specified number of ticks.
    /// Returns 0 on success, -1 on null pointer.
    #[no_mangle]
    pub extern "C" fn mchprs_world_tick(world: *mut MchprsWorldWrapper, ticks: u32) -> c_int {
        if world.is_null() {
            return -1;
        }
        let w = unsafe { &mut *(*world).0 };
        w.tick(ticks);
        0
    }

    /// Flushes pending changes from the compiler to the world.
    /// Returns 0 on success, -1 on null pointer.
    #[no_mangle]
    pub extern "C" fn mchprs_world_flush(world: *mut MchprsWorldWrapper) -> c_int {
        if world.is_null() {
            return -1;
        }
        let w = unsafe { &mut *(*world).0 };
        w.flush();
        0
    }

    /// Sets the power state of a lever.
    /// `powered`: 0=off, non-zero=on. Uses c_int instead of bool for ABI safety.
    /// Returns 0 on success, -1 on null pointer.
    #[no_mangle]
    pub extern "C" fn mchprs_world_set_lever_power(
        world: *mut MchprsWorldWrapper,
        x: c_int,
        y: c_int,
        z: c_int,
        powered: c_int,
    ) -> c_int {
        if world.is_null() {
            return -1;
        }
        let w = unsafe { &mut *(*world).0 };
        w.set_lever_power(BlockPos::new(x, y, z), powered != 0);
        0
    }

    /// Gets the power state of a lever.
    /// Returns 1 if powered, 0 if not, -1 on null pointer.
    #[no_mangle]
    pub extern "C" fn mchprs_world_get_lever_power(
        world: *const MchprsWorldWrapper,
        x: c_int,
        y: c_int,
        z: c_int,
    ) -> c_int {
        if world.is_null() {
            return -1;
        }
        let w = unsafe { &*(*world).0 };
        if w.get_lever_power(BlockPos::new(x, y, z)) {
            1
        } else {
            0
        }
    }

    /// Checks if a redstone lamp is lit at the given position.
    /// Returns 1 if lit, 0 if not, -1 on null pointer.
    #[no_mangle]
    pub extern "C" fn mchprs_world_is_lit(
        world: *const MchprsWorldWrapper,
        x: c_int,
        y: c_int,
        z: c_int,
    ) -> c_int {
        if world.is_null() {
            return -1;
        }
        let w = unsafe { &*(*world).0 };
        if w.is_lit(BlockPos::new(x, y, z)) {
            1
        } else {
            0
        }
    }

    /// Sets the signal strength at a position (for custom IO).
    /// `strength` is 0-15.
    /// Returns 0 on success, -1 on null pointer.
    #[no_mangle]
    pub extern "C" fn mchprs_world_set_signal_strength(
        world: *mut MchprsWorldWrapper,
        x: c_int,
        y: c_int,
        z: c_int,
        strength: u8,
    ) -> c_int {
        if world.is_null() {
            return -1;
        }
        let w = unsafe { &mut *(*world).0 };
        w.set_signal_strength(BlockPos::new(x, y, z), strength);
        0
    }

    /// Gets the signal strength at a position.
    /// Returns 0-15 signal strength, or 0 on null pointer.
    #[no_mangle]
    pub extern "C" fn mchprs_world_get_signal_strength(
        world: *const MchprsWorldWrapper,
        x: c_int,
        y: c_int,
        z: c_int,
    ) -> u8 {
        if world.is_null() {
            return 0;
        }
        let w = unsafe { &*(*world).0 };
        w.get_signal_strength(BlockPos::new(x, y, z))
    }

    /// Simulates a right-click on a block (typically a lever).
    /// Returns 0 on success, -1 on null pointer.
    #[no_mangle]
    pub extern "C" fn mchprs_world_on_use_block(
        world: *mut MchprsWorldWrapper,
        x: c_int,
        y: c_int,
        z: c_int,
    ) -> c_int {
        if world.is_null() {
            return -1;
        }
        let w = unsafe { &mut *(*world).0 };
        w.on_use_block(BlockPos::new(x, y, z));
        0
    }

    /// Syncs the simulation state back to the schematic.
    /// Returns 0 on success, -1 on null pointer.
    #[no_mangle]
    pub extern "C" fn mchprs_world_sync_to_schematic(world: *mut MchprsWorldWrapper) -> c_int {
        if world.is_null() {
            return -1;
        }
        let w = unsafe { &mut *(*world).0 };
        w.sync_to_schematic();
        0
    }

    /// Gets a clone of the schematic from the world.
    /// Caller must free the returned SchematicWrapper with `schematic_free`.
    /// Returns null on null pointer.
    #[no_mangle]
    pub extern "C" fn mchprs_world_get_schematic(
        world: *const MchprsWorldWrapper,
    ) -> *mut SchematicWrapper {
        if world.is_null() {
            return ptr::null_mut();
        }
        let w = unsafe { &*(*world).0 };
        let cloned = w.get_schematic().clone();
        let boxed = Box::into_raw(Box::new(cloned));
        Box::into_raw(Box::new(SchematicWrapper(boxed)))
    }

    /// One-shot redstone simulation convenience.
    ///
    /// Creates a simulation world from `schematic`, fires `n_events` `on_use_block`
    /// events read from `events_xyz` (a flat `[x,y,z, x,y,z, ...]` array of
    /// length `n_events * 3`), runs `ticks` ticks, syncs the result back, and
    /// writes the simulated schematic state into the original `*mut SchematicWrapper`.
    ///
    /// Pass a null `events_xyz` (with `n_events == 0`) to just tick the world.
    ///
    /// Returns 0 on success, negative on error.
    #[no_mangle]
    pub extern "C" fn schematic_simulate_use_block(
        schematic: *mut SchematicWrapper,
        ticks: u32,
        events_xyz: *const c_int,
        n_events: usize,
    ) -> c_int {
        if schematic.is_null() {
            set_last_error("schematic pointer is null".to_string());
            return -1;
        }
        let s = unsafe { &mut *(*schematic).0 };
        let mut world = match MchprsWorld::new(s.clone()) {
            Ok(w) => w,
            Err(e) => {
                set_last_error(format!("MchprsWorld::new failed: {}", e));
                return -2;
            }
        };
        if !events_xyz.is_null() && n_events > 0 {
            let slice = unsafe { std::slice::from_raw_parts(events_xyz, n_events * 3) };
            for ev in slice.chunks_exact(3) {
                world.on_use_block(BlockPos::new(ev[0], ev[1], ev[2]));
            }
        }
        world.tick(ticks);
        world.sync_to_schematic();
        *s = world.into_schematic();
        0
    }

    // =========================================================================
    // Value FFI Bindings
    // =========================================================================

    #[no_mangle]
    pub extern "C" fn value_from_u32(v: u32) -> *mut ValueWrapper {
        Box::into_raw(Box::new(ValueWrapper(Value::U32(v))))
    }

    #[no_mangle]
    pub extern "C" fn value_from_i32(v: i32) -> *mut ValueWrapper {
        Box::into_raw(Box::new(ValueWrapper(Value::I32(v))))
    }

    #[no_mangle]
    pub extern "C" fn value_from_f32(v: f32) -> *mut ValueWrapper {
        Box::into_raw(Box::new(ValueWrapper(Value::F32(v))))
    }

    #[no_mangle]
    pub extern "C" fn value_from_bool(v: c_int) -> *mut ValueWrapper {
        Box::into_raw(Box::new(ValueWrapper(Value::Bool(v != 0))))
    }

    #[no_mangle]
    pub extern "C" fn value_from_string(s: *const c_char) -> *mut ValueWrapper {
        if s.is_null() {
            return ptr::null_mut();
        }
        let s = unsafe { CStr::from_ptr(s) }.to_string_lossy().into_owned();
        Box::into_raw(Box::new(ValueWrapper(Value::String(s))))
    }

    #[no_mangle]
    pub extern "C" fn value_as_u32(v: *const ValueWrapper) -> u32 {
        if v.is_null() {
            return 0;
        }
        unsafe { &*v }.0.as_u32().unwrap_or(0)
    }

    #[no_mangle]
    pub extern "C" fn value_as_i32(v: *const ValueWrapper) -> i32 {
        if v.is_null() {
            return 0;
        }
        unsafe { &*v }.0.as_i32().unwrap_or(0)
    }

    #[no_mangle]
    pub extern "C" fn value_as_f32(v: *const ValueWrapper) -> f32 {
        if v.is_null() {
            return 0.0;
        }
        unsafe { &*v }.0.as_f32().unwrap_or(0.0)
    }

    #[no_mangle]
    pub extern "C" fn value_as_bool(v: *const ValueWrapper) -> c_int {
        if v.is_null() {
            return 0;
        }
        if unsafe { &*v }.0.as_bool().unwrap_or(false) {
            1
        } else {
            0
        }
    }

    /// Returns the string value. Caller must free with `schematic_free_string`.
    /// Returns null if not a string value.
    #[no_mangle]
    pub extern "C" fn value_as_string(v: *const ValueWrapper) -> *mut c_char {
        if v.is_null() {
            return ptr::null_mut();
        }
        match unsafe { &*v }.0.as_str() {
            Ok(s) => CString::new(s).unwrap_or_default().into_raw(),
            Err(_) => ptr::null_mut(),
        }
    }

    /// Returns the type name (e.g. "u32", "bool", "string").
    /// Caller must free with `schematic_free_string`.
    #[no_mangle]
    pub extern "C" fn value_type_name(v: *const ValueWrapper) -> *mut c_char {
        if v.is_null() {
            return ptr::null_mut();
        }
        let name = match &unsafe { &*v }.0 {
            Value::U32(_) => "u32",
            Value::U64(_) => "u64",
            Value::I32(_) => "i32",
            Value::I64(_) => "i64",
            Value::F32(_) => "f32",
            Value::Bool(_) => "bool",
            Value::String(_) => "string",
            Value::BitArray(_) => "bit_array",
            Value::Bytes(_) => "bytes",
            Value::Array(_) => "array",
            Value::Struct(_) => "struct",
        };
        CString::new(name).unwrap_or_default().into_raw()
    }

    #[no_mangle]
    pub extern "C" fn value_free(ptr: *mut ValueWrapper) {
        if !ptr.is_null() {
            unsafe {
                let _ = Box::from_raw(ptr);
            }
        }
    }

    // =========================================================================
    // IoType FFI Bindings
    // =========================================================================

    #[no_mangle]
    pub extern "C" fn io_type_unsigned_int(bits: usize) -> *mut IoTypeWrapper {
        Box::into_raw(Box::new(IoTypeWrapper(IoType::UnsignedInt { bits })))
    }

    #[no_mangle]
    pub extern "C" fn io_type_signed_int(bits: usize) -> *mut IoTypeWrapper {
        Box::into_raw(Box::new(IoTypeWrapper(IoType::SignedInt { bits })))
    }

    #[no_mangle]
    pub extern "C" fn io_type_float32() -> *mut IoTypeWrapper {
        Box::into_raw(Box::new(IoTypeWrapper(IoType::Float32)))
    }

    #[no_mangle]
    pub extern "C" fn io_type_boolean() -> *mut IoTypeWrapper {
        Box::into_raw(Box::new(IoTypeWrapper(IoType::Boolean)))
    }

    #[no_mangle]
    pub extern "C" fn io_type_ascii(chars: usize) -> *mut IoTypeWrapper {
        Box::into_raw(Box::new(IoTypeWrapper(IoType::Ascii { chars })))
    }

    #[no_mangle]
    pub extern "C" fn io_type_free(ptr: *mut IoTypeWrapper) {
        if !ptr.is_null() {
            unsafe {
                let _ = Box::from_raw(ptr);
            }
        }
    }

    // =========================================================================
    // LayoutFunction FFI Bindings
    // =========================================================================

    #[no_mangle]
    pub extern "C" fn layout_function_one_to_one() -> *mut LayoutFunctionWrapper {
        Box::into_raw(Box::new(LayoutFunctionWrapper(LayoutFunction::OneToOne)))
    }

    #[no_mangle]
    pub extern "C" fn layout_function_packed4() -> *mut LayoutFunctionWrapper {
        Box::into_raw(Box::new(LayoutFunctionWrapper(LayoutFunction::Packed4)))
    }

    /// `mapping` is a pointer to an array of usize values of length `len`.
    #[no_mangle]
    pub extern "C" fn layout_function_custom(
        mapping: *const usize,
        len: usize,
    ) -> *mut LayoutFunctionWrapper {
        if mapping.is_null() || len == 0 {
            return ptr::null_mut();
        }
        let mapping = unsafe { std::slice::from_raw_parts(mapping, len) }.to_vec();
        Box::into_raw(Box::new(LayoutFunctionWrapper(LayoutFunction::Custom(
            mapping,
        ))))
    }

    #[no_mangle]
    pub extern "C" fn layout_function_row_major(
        rows: usize,
        cols: usize,
        bits_per_element: usize,
    ) -> *mut LayoutFunctionWrapper {
        Box::into_raw(Box::new(LayoutFunctionWrapper(LayoutFunction::RowMajor {
            rows,
            cols,
            bits_per_element,
        })))
    }

    #[no_mangle]
    pub extern "C" fn layout_function_column_major(
        rows: usize,
        cols: usize,
        bits_per_element: usize,
    ) -> *mut LayoutFunctionWrapper {
        Box::into_raw(Box::new(LayoutFunctionWrapper(
            LayoutFunction::ColumnMajor {
                rows,
                cols,
                bits_per_element,
            },
        )))
    }

    #[no_mangle]
    pub extern "C" fn layout_function_scanline(
        width: usize,
        height: usize,
        bits_per_pixel: usize,
    ) -> *mut LayoutFunctionWrapper {
        Box::into_raw(Box::new(LayoutFunctionWrapper(LayoutFunction::Scanline {
            width,
            height,
            bits_per_pixel,
        })))
    }

    #[no_mangle]
    pub extern "C" fn layout_function_free(ptr: *mut LayoutFunctionWrapper) {
        if !ptr.is_null() {
            unsafe {
                let _ = Box::from_raw(ptr);
            }
        }
    }

    // =========================================================================
    // OutputCondition FFI Bindings
    // =========================================================================

    #[no_mangle]
    pub extern "C" fn output_condition_equals(
        value: *const ValueWrapper,
    ) -> *mut OutputConditionWrapper {
        if value.is_null() {
            return ptr::null_mut();
        }
        let v = unsafe { &*value }.0.clone();
        Box::into_raw(Box::new(OutputConditionWrapper(OutputCondition::Equals(v))))
    }

    #[no_mangle]
    pub extern "C" fn output_condition_not_equals(
        value: *const ValueWrapper,
    ) -> *mut OutputConditionWrapper {
        if value.is_null() {
            return ptr::null_mut();
        }
        let v = unsafe { &*value }.0.clone();
        Box::into_raw(Box::new(OutputConditionWrapper(
            OutputCondition::NotEquals(v),
        )))
    }

    #[no_mangle]
    pub extern "C" fn output_condition_greater_than(
        value: *const ValueWrapper,
    ) -> *mut OutputConditionWrapper {
        if value.is_null() {
            return ptr::null_mut();
        }
        let v = unsafe { &*value }.0.clone();
        Box::into_raw(Box::new(OutputConditionWrapper(
            OutputCondition::GreaterThan(v),
        )))
    }

    #[no_mangle]
    pub extern "C" fn output_condition_less_than(
        value: *const ValueWrapper,
    ) -> *mut OutputConditionWrapper {
        if value.is_null() {
            return ptr::null_mut();
        }
        let v = unsafe { &*value }.0.clone();
        Box::into_raw(Box::new(OutputConditionWrapper(OutputCondition::LessThan(
            v,
        ))))
    }

    #[no_mangle]
    pub extern "C" fn output_condition_bitwise_and(mask: u32) -> *mut OutputConditionWrapper {
        Box::into_raw(Box::new(OutputConditionWrapper(
            OutputCondition::BitwiseAnd(mask as u64),
        )))
    }

    #[no_mangle]
    pub extern "C" fn output_condition_free(ptr: *mut OutputConditionWrapper) {
        if !ptr.is_null() {
            unsafe {
                let _ = Box::from_raw(ptr);
            }
        }
    }

    // =========================================================================
    // ExecutionMode FFI Bindings
    // =========================================================================

    #[no_mangle]
    pub extern "C" fn execution_mode_fixed_ticks(ticks: u32) -> *mut ExecutionModeWrapper {
        Box::into_raw(Box::new(ExecutionModeWrapper(ExecutionMode::FixedTicks {
            ticks,
        })))
    }

    #[no_mangle]
    pub extern "C" fn execution_mode_until_condition(
        output_name: *const c_char,
        condition: *const OutputConditionWrapper,
        max_ticks: u32,
        check_interval: u32,
    ) -> *mut ExecutionModeWrapper {
        if output_name.is_null() || condition.is_null() {
            return ptr::null_mut();
        }
        let name = unsafe { CStr::from_ptr(output_name) }
            .to_string_lossy()
            .into_owned();
        let cond = unsafe { &*condition }.0.clone();
        Box::into_raw(Box::new(ExecutionModeWrapper(
            ExecutionMode::UntilCondition {
                output_name: name,
                condition: cond,
                max_ticks,
                check_interval,
            },
        )))
    }

    #[no_mangle]
    pub extern "C" fn execution_mode_until_change(
        max_ticks: u32,
        check_interval: u32,
    ) -> *mut ExecutionModeWrapper {
        Box::into_raw(Box::new(ExecutionModeWrapper(ExecutionMode::UntilChange {
            max_ticks,
            check_interval,
        })))
    }

    #[no_mangle]
    pub extern "C" fn execution_mode_until_stable(
        stable_ticks: u32,
        max_ticks: u32,
    ) -> *mut ExecutionModeWrapper {
        Box::into_raw(Box::new(ExecutionModeWrapper(ExecutionMode::UntilStable {
            stable_ticks,
            max_ticks,
        })))
    }

    #[no_mangle]
    pub extern "C" fn execution_mode_free(ptr: *mut ExecutionModeWrapper) {
        if !ptr.is_null() {
            unsafe {
                let _ = Box::from_raw(ptr);
            }
        }
    }

    // =========================================================================
    // SortStrategy FFI Bindings
    // =========================================================================

    #[no_mangle]
    pub extern "C" fn sort_strategy_yxz() -> *mut SortStrategyWrapper {
        Box::into_raw(Box::new(SortStrategyWrapper(SortStrategy::YXZ)))
    }

    #[no_mangle]
    pub extern "C" fn sort_strategy_xyz() -> *mut SortStrategyWrapper {
        Box::into_raw(Box::new(SortStrategyWrapper(SortStrategy::XYZ)))
    }

    #[no_mangle]
    pub extern "C" fn sort_strategy_zyx() -> *mut SortStrategyWrapper {
        Box::into_raw(Box::new(SortStrategyWrapper(SortStrategy::ZYX)))
    }

    #[no_mangle]
    pub extern "C" fn sort_strategy_y_desc_xz() -> *mut SortStrategyWrapper {
        Box::into_raw(Box::new(SortStrategyWrapper(SortStrategy::YDescXZ)))
    }

    #[no_mangle]
    pub extern "C" fn sort_strategy_x_desc_yz() -> *mut SortStrategyWrapper {
        Box::into_raw(Box::new(SortStrategyWrapper(SortStrategy::XDescYZ)))
    }

    #[no_mangle]
    pub extern "C" fn sort_strategy_z_desc_yx() -> *mut SortStrategyWrapper {
        Box::into_raw(Box::new(SortStrategyWrapper(SortStrategy::ZDescYX)))
    }

    #[no_mangle]
    pub extern "C" fn sort_strategy_descending() -> *mut SortStrategyWrapper {
        Box::into_raw(Box::new(SortStrategyWrapper(SortStrategy::YXZDesc)))
    }

    #[no_mangle]
    pub extern "C" fn sort_strategy_distance_from(
        x: i32,
        y: i32,
        z: i32,
    ) -> *mut SortStrategyWrapper {
        Box::into_raw(Box::new(SortStrategyWrapper(SortStrategy::DistanceFrom {
            reference: (x, y, z),
        })))
    }

    #[no_mangle]
    pub extern "C" fn sort_strategy_distance_from_desc(
        x: i32,
        y: i32,
        z: i32,
    ) -> *mut SortStrategyWrapper {
        Box::into_raw(Box::new(SortStrategyWrapper(
            SortStrategy::DistanceFromDesc {
                reference: (x, y, z),
            },
        )))
    }

    #[no_mangle]
    pub extern "C" fn sort_strategy_preserve() -> *mut SortStrategyWrapper {
        Box::into_raw(Box::new(SortStrategyWrapper(SortStrategy::Preserve)))
    }

    #[no_mangle]
    pub extern "C" fn sort_strategy_reverse() -> *mut SortStrategyWrapper {
        Box::into_raw(Box::new(SortStrategyWrapper(SortStrategy::Reverse)))
    }

    /// Parse a sort strategy from a string (e.g. "yxz", "descending", "preserve").
    /// Returns null if the string is not recognized.
    #[no_mangle]
    pub extern "C" fn sort_strategy_from_string(s: *const c_char) -> *mut SortStrategyWrapper {
        if s.is_null() {
            return ptr::null_mut();
        }
        let s = unsafe { CStr::from_ptr(s) }.to_string_lossy();
        match SortStrategy::from_str(&s) {
            Some(strategy) => Box::into_raw(Box::new(SortStrategyWrapper(strategy))),
            None => ptr::null_mut(),
        }
    }

    /// Returns the strategy name. Caller must free with `schematic_free_string`.
    #[no_mangle]
    pub extern "C" fn sort_strategy_name(ptr: *const SortStrategyWrapper) -> *mut c_char {
        if ptr.is_null() {
            return ptr::null_mut();
        }
        let name = unsafe { &*ptr }.0.name();
        CString::new(name).unwrap_or_default().into_raw()
    }

    #[no_mangle]
    pub extern "C" fn sort_strategy_free(ptr: *mut SortStrategyWrapper) {
        if !ptr.is_null() {
            unsafe {
                let _ = Box::from_raw(ptr);
            }
        }
    }

    // =========================================================================
    // IoLayoutBuilder FFI Bindings
    // =========================================================================

    #[no_mangle]
    pub extern "C" fn io_layout_builder_new() -> *mut IoLayoutBuilderWrapper {
        Box::into_raw(Box::new(IoLayoutBuilderWrapper(Some(
            IoLayoutBuilder::new(),
        ))))
    }

    /// Adds an input to the builder.
    /// `positions` is a flat array of [x,y,z,x,y,z,...] with `count` positions (array length = count*3).
    /// Returns 0 on success, -1 on error.
    #[no_mangle]
    pub extern "C" fn io_layout_builder_add_input(
        builder: *mut IoLayoutBuilderWrapper,
        name: *const c_char,
        io_type: *const IoTypeWrapper,
        layout: *const LayoutFunctionWrapper,
        positions: *const c_int,
        count: usize,
    ) -> c_int {
        if builder.is_null() || name.is_null() || io_type.is_null() || layout.is_null() {
            return -1;
        }
        let b = unsafe { &mut *builder };
        let inner = match b.0.take() {
            Some(inner) => inner,
            None => {
                set_last_error("Builder already consumed".into());
                return -1;
            }
        };
        let name_str = unsafe { CStr::from_ptr(name) }
            .to_string_lossy()
            .into_owned();
        let io_t = unsafe { &*io_type }.0.clone();
        let lay = unsafe { &*layout }.0.clone();
        let pos = parse_positions(positions, count);

        match inner.add_input(name_str, io_t, lay, pos) {
            Ok(new_builder) => {
                b.0 = Some(new_builder);
                0
            }
            Err(e) => {
                set_last_error(e);
                -1
            }
        }
    }

    /// Adds an output to the builder.
    /// `positions` is a flat array of [x,y,z,...] with `count` positions.
    /// Returns 0 on success, -1 on error.
    #[no_mangle]
    pub extern "C" fn io_layout_builder_add_output(
        builder: *mut IoLayoutBuilderWrapper,
        name: *const c_char,
        io_type: *const IoTypeWrapper,
        layout: *const LayoutFunctionWrapper,
        positions: *const c_int,
        count: usize,
    ) -> c_int {
        if builder.is_null() || name.is_null() || io_type.is_null() || layout.is_null() {
            return -1;
        }
        let b = unsafe { &mut *builder };
        let inner = match b.0.take() {
            Some(inner) => inner,
            None => {
                set_last_error("Builder already consumed".into());
                return -1;
            }
        };
        let name_str = unsafe { CStr::from_ptr(name) }
            .to_string_lossy()
            .into_owned();
        let io_t = unsafe { &*io_type }.0.clone();
        let lay = unsafe { &*layout }.0.clone();
        let pos = parse_positions(positions, count);

        match inner.add_output(name_str, io_t, lay, pos) {
            Ok(new_builder) => {
                b.0 = Some(new_builder);
                0
            }
            Err(e) => {
                set_last_error(e);
                -1
            }
        }
    }

    /// Adds an input with automatic layout inference.
    #[no_mangle]
    pub extern "C" fn io_layout_builder_add_input_auto(
        builder: *mut IoLayoutBuilderWrapper,
        name: *const c_char,
        io_type: *const IoTypeWrapper,
        positions: *const c_int,
        count: usize,
    ) -> c_int {
        if builder.is_null() || name.is_null() || io_type.is_null() {
            return -1;
        }
        let b = unsafe { &mut *builder };
        let inner = match b.0.take() {
            Some(inner) => inner,
            None => {
                set_last_error("Builder already consumed".into());
                return -1;
            }
        };
        let name_str = unsafe { CStr::from_ptr(name) }
            .to_string_lossy()
            .into_owned();
        let io_t = unsafe { &*io_type }.0.clone();
        let pos = parse_positions(positions, count);

        match inner.add_input_auto(name_str, io_t, pos) {
            Ok(new_builder) => {
                b.0 = Some(new_builder);
                0
            }
            Err(e) => {
                set_last_error(e);
                -1
            }
        }
    }

    /// Adds an output with automatic layout inference.
    #[no_mangle]
    pub extern "C" fn io_layout_builder_add_output_auto(
        builder: *mut IoLayoutBuilderWrapper,
        name: *const c_char,
        io_type: *const IoTypeWrapper,
        positions: *const c_int,
        count: usize,
    ) -> c_int {
        if builder.is_null() || name.is_null() || io_type.is_null() {
            return -1;
        }
        let b = unsafe { &mut *builder };
        let inner = match b.0.take() {
            Some(inner) => inner,
            None => {
                set_last_error("Builder already consumed".into());
                return -1;
            }
        };
        let name_str = unsafe { CStr::from_ptr(name) }
            .to_string_lossy()
            .into_owned();
        let io_t = unsafe { &*io_type }.0.clone();
        let pos = parse_positions(positions, count);

        match inner.add_output_auto(name_str, io_t, pos) {
            Ok(new_builder) => {
                b.0 = Some(new_builder);
                0
            }
            Err(e) => {
                set_last_error(e);
                -1
            }
        }
    }

    /// Adds an input from a DefinitionRegion.
    #[no_mangle]
    pub extern "C" fn io_layout_builder_add_input_from_region(
        builder: *mut IoLayoutBuilderWrapper,
        name: *const c_char,
        io_type: *const IoTypeWrapper,
        layout: *const LayoutFunctionWrapper,
        region: *const DefinitionRegionWrapper,
    ) -> c_int {
        if builder.is_null()
            || name.is_null()
            || io_type.is_null()
            || layout.is_null()
            || region.is_null()
        {
            return -1;
        }
        let b = unsafe { &mut *builder };
        let inner = match b.0.take() {
            Some(inner) => inner,
            None => {
                set_last_error("Builder already consumed".into());
                return -1;
            }
        };
        let name_str = unsafe { CStr::from_ptr(name) }
            .to_string_lossy()
            .into_owned();
        let io_t = unsafe { &*io_type }.0.clone();
        let lay = unsafe { &*layout }.0.clone();
        let reg = unsafe { &*region }.0.clone();

        match inner.add_input_from_region(name_str, io_t, lay, reg) {
            Ok(new_builder) => {
                b.0 = Some(new_builder);
                0
            }
            Err(e) => {
                set_last_error(e);
                -1
            }
        }
    }

    /// Adds an input from a DefinitionRegion with automatic layout inference.
    #[no_mangle]
    pub extern "C" fn io_layout_builder_add_input_from_region_auto(
        builder: *mut IoLayoutBuilderWrapper,
        name: *const c_char,
        io_type: *const IoTypeWrapper,
        region: *const DefinitionRegionWrapper,
    ) -> c_int {
        if builder.is_null() || name.is_null() || io_type.is_null() || region.is_null() {
            return -1;
        }
        let b = unsafe { &mut *builder };
        let inner = match b.0.take() {
            Some(inner) => inner,
            None => {
                set_last_error("Builder already consumed".into());
                return -1;
            }
        };
        let name_str = unsafe { CStr::from_ptr(name) }
            .to_string_lossy()
            .into_owned();
        let io_t = unsafe { &*io_type }.0.clone();
        let reg = unsafe { &*region }.0.clone();

        match inner.add_input_from_region_auto(name_str, io_t, reg) {
            Ok(new_builder) => {
                b.0 = Some(new_builder);
                0
            }
            Err(e) => {
                set_last_error(e);
                -1
            }
        }
    }

    /// Adds an output from a DefinitionRegion.
    #[no_mangle]
    pub extern "C" fn io_layout_builder_add_output_from_region(
        builder: *mut IoLayoutBuilderWrapper,
        name: *const c_char,
        io_type: *const IoTypeWrapper,
        layout: *const LayoutFunctionWrapper,
        region: *const DefinitionRegionWrapper,
    ) -> c_int {
        if builder.is_null()
            || name.is_null()
            || io_type.is_null()
            || layout.is_null()
            || region.is_null()
        {
            return -1;
        }
        let b = unsafe { &mut *builder };
        let inner = match b.0.take() {
            Some(inner) => inner,
            None => {
                set_last_error("Builder already consumed".into());
                return -1;
            }
        };
        let name_str = unsafe { CStr::from_ptr(name) }
            .to_string_lossy()
            .into_owned();
        let io_t = unsafe { &*io_type }.0.clone();
        let lay = unsafe { &*layout }.0.clone();
        let reg = unsafe { &*region }.0.clone();

        match inner.add_output_from_region(name_str, io_t, lay, reg) {
            Ok(new_builder) => {
                b.0 = Some(new_builder);
                0
            }
            Err(e) => {
                set_last_error(e);
                -1
            }
        }
    }

    /// Adds an output from a DefinitionRegion with automatic layout inference.
    #[no_mangle]
    pub extern "C" fn io_layout_builder_add_output_from_region_auto(
        builder: *mut IoLayoutBuilderWrapper,
        name: *const c_char,
        io_type: *const IoTypeWrapper,
        region: *const DefinitionRegionWrapper,
    ) -> c_int {
        if builder.is_null() || name.is_null() || io_type.is_null() || region.is_null() {
            return -1;
        }
        let b = unsafe { &mut *builder };
        let inner = match b.0.take() {
            Some(inner) => inner,
            None => {
                set_last_error("Builder already consumed".into());
                return -1;
            }
        };
        let name_str = unsafe { CStr::from_ptr(name) }
            .to_string_lossy()
            .into_owned();
        let io_t = unsafe { &*io_type }.0.clone();
        let reg = unsafe { &*region }.0.clone();

        match inner.add_output_from_region_auto(name_str, io_t, reg) {
            Ok(new_builder) => {
                b.0 = Some(new_builder);
                0
            }
            Err(e) => {
                set_last_error(e);
                -1
            }
        }
    }

    /// Builds the IoLayout from the builder. Consumes the builder.
    /// Returns null on error. Caller must free with `io_layout_free`.
    #[no_mangle]
    pub extern "C" fn io_layout_builder_build(
        builder: *mut IoLayoutBuilderWrapper,
    ) -> *mut IoLayoutWrapper {
        if builder.is_null() {
            return ptr::null_mut();
        }
        let b = unsafe { &mut *builder };
        match b.0.take() {
            Some(inner) => {
                let layout = inner.build();
                Box::into_raw(Box::new(IoLayoutWrapper(layout)))
            }
            None => {
                set_last_error("Builder already consumed".into());
                ptr::null_mut()
            }
        }
    }

    #[no_mangle]
    pub extern "C" fn io_layout_builder_free(ptr: *mut IoLayoutBuilderWrapper) {
        if !ptr.is_null() {
            unsafe {
                let _ = Box::from_raw(ptr);
            }
        }
    }

    // =========================================================================
    // IoLayout FFI Bindings
    // =========================================================================

    /// Returns input names as a JSON array string. Caller must free with `schematic_free_string`.
    #[no_mangle]
    pub extern "C" fn io_layout_input_names(layout: *const IoLayoutWrapper) -> *mut c_char {
        if layout.is_null() {
            return ptr::null_mut();
        }
        let names: Vec<&str> = unsafe { &*layout }.0.input_names();
        let json = serde_json::to_string(&names).unwrap_or_else(|_| "[]".to_string());
        CString::new(json).unwrap_or_default().into_raw()
    }

    /// Returns output names as a JSON array string. Caller must free with `schematic_free_string`.
    #[no_mangle]
    pub extern "C" fn io_layout_output_names(layout: *const IoLayoutWrapper) -> *mut c_char {
        if layout.is_null() {
            return ptr::null_mut();
        }
        let names: Vec<&str> = unsafe { &*layout }.0.output_names();
        let json = serde_json::to_string(&names).unwrap_or_else(|_| "[]".to_string());
        CString::new(json).unwrap_or_default().into_raw()
    }

    #[no_mangle]
    pub extern "C" fn io_layout_free(ptr: *mut IoLayoutWrapper) {
        if !ptr.is_null() {
            unsafe {
                let _ = Box::from_raw(ptr);
            }
        }
    }

    // =========================================================================
    // CircuitBuilder FFI Bindings
    // =========================================================================

    /// Creates a new CircuitBuilder from a schematic.
    #[no_mangle]
    pub extern "C" fn circuit_builder_new(
        schematic: *const SchematicWrapper,
    ) -> *mut CircuitBuilderWrapper {
        if schematic.is_null() {
            return ptr::null_mut();
        }
        let s = unsafe { &*(*schematic).0 };
        Box::into_raw(Box::new(CircuitBuilderWrapper(Some(CircuitBuilder::new(
            s.clone(),
        )))))
    }

    /// Creates a CircuitBuilder pre-populated from Insign annotations.
    /// Returns null on error.
    #[no_mangle]
    pub extern "C" fn circuit_builder_from_insign(
        schematic: *const SchematicWrapper,
    ) -> *mut CircuitBuilderWrapper {
        if schematic.is_null() {
            return ptr::null_mut();
        }
        let s = unsafe { &*(*schematic).0 };
        match CircuitBuilder::from_insign(s.clone()) {
            Ok(builder) => Box::into_raw(Box::new(CircuitBuilderWrapper(Some(builder)))),
            Err(e) => {
                set_last_error(e);
                ptr::null_mut()
            }
        }
    }

    /// Adds an input with full control. Returns 0 on success, -1 on error.
    #[no_mangle]
    pub extern "C" fn circuit_builder_with_input(
        builder: *mut CircuitBuilderWrapper,
        name: *const c_char,
        io_type: *const IoTypeWrapper,
        layout: *const LayoutFunctionWrapper,
        region: *const DefinitionRegionWrapper,
    ) -> c_int {
        if builder.is_null()
            || name.is_null()
            || io_type.is_null()
            || layout.is_null()
            || region.is_null()
        {
            return -1;
        }
        let b = unsafe { &mut *builder };
        let inner = match b.0.take() {
            Some(inner) => inner,
            None => {
                set_last_error("Builder already consumed".into());
                return -1;
            }
        };
        let name_str = unsafe { CStr::from_ptr(name) }
            .to_string_lossy()
            .into_owned();
        let io_t = unsafe { &*io_type }.0.clone();
        let lay = unsafe { &*layout }.0.clone();
        let reg = unsafe { &*region }.0.clone();

        match inner.with_input(name_str, io_t, lay, reg) {
            Ok(new_builder) => {
                b.0 = Some(new_builder);
                0
            }
            Err(e) => {
                set_last_error(e);
                -1
            }
        }
    }

    /// Adds an input with full control and custom sort strategy.
    #[no_mangle]
    pub extern "C" fn circuit_builder_with_input_sorted(
        builder: *mut CircuitBuilderWrapper,
        name: *const c_char,
        io_type: *const IoTypeWrapper,
        layout: *const LayoutFunctionWrapper,
        region: *const DefinitionRegionWrapper,
        sort: *const SortStrategyWrapper,
    ) -> c_int {
        if builder.is_null()
            || name.is_null()
            || io_type.is_null()
            || layout.is_null()
            || region.is_null()
            || sort.is_null()
        {
            return -1;
        }
        let b = unsafe { &mut *builder };
        let inner = match b.0.take() {
            Some(inner) => inner,
            None => {
                set_last_error("Builder already consumed".into());
                return -1;
            }
        };
        let name_str = unsafe { CStr::from_ptr(name) }
            .to_string_lossy()
            .into_owned();
        let io_t = unsafe { &*io_type }.0.clone();
        let lay = unsafe { &*layout }.0.clone();
        let reg = unsafe { &*region }.0.clone();
        let s = unsafe { &*sort }.0.clone();

        match inner.with_input_sorted(name_str, io_t, lay, reg, s) {
            Ok(new_builder) => {
                b.0 = Some(new_builder);
                0
            }
            Err(e) => {
                set_last_error(e);
                -1
            }
        }
    }

    /// Adds an input with automatic layout inference.
    #[no_mangle]
    pub extern "C" fn circuit_builder_with_input_auto(
        builder: *mut CircuitBuilderWrapper,
        name: *const c_char,
        io_type: *const IoTypeWrapper,
        region: *const DefinitionRegionWrapper,
    ) -> c_int {
        if builder.is_null() || name.is_null() || io_type.is_null() || region.is_null() {
            return -1;
        }
        let b = unsafe { &mut *builder };
        let inner = match b.0.take() {
            Some(inner) => inner,
            None => {
                set_last_error("Builder already consumed".into());
                return -1;
            }
        };
        let name_str = unsafe { CStr::from_ptr(name) }
            .to_string_lossy()
            .into_owned();
        let io_t = unsafe { &*io_type }.0.clone();
        let reg = unsafe { &*region }.0.clone();

        match inner.with_input_auto(name_str, io_t, reg) {
            Ok(new_builder) => {
                b.0 = Some(new_builder);
                0
            }
            Err(e) => {
                set_last_error(e);
                -1
            }
        }
    }

    /// Adds an input with automatic layout inference and custom sort.
    #[no_mangle]
    pub extern "C" fn circuit_builder_with_input_auto_sorted(
        builder: *mut CircuitBuilderWrapper,
        name: *const c_char,
        io_type: *const IoTypeWrapper,
        region: *const DefinitionRegionWrapper,
        sort: *const SortStrategyWrapper,
    ) -> c_int {
        if builder.is_null()
            || name.is_null()
            || io_type.is_null()
            || region.is_null()
            || sort.is_null()
        {
            return -1;
        }
        let b = unsafe { &mut *builder };
        let inner = match b.0.take() {
            Some(inner) => inner,
            None => {
                set_last_error("Builder already consumed".into());
                return -1;
            }
        };
        let name_str = unsafe { CStr::from_ptr(name) }
            .to_string_lossy()
            .into_owned();
        let io_t = unsafe { &*io_type }.0.clone();
        let reg = unsafe { &*region }.0.clone();
        let s = unsafe { &*sort }.0.clone();

        match inner.with_input_auto_sorted(name_str, io_t, reg, s) {
            Ok(new_builder) => {
                b.0 = Some(new_builder);
                0
            }
            Err(e) => {
                set_last_error(e);
                -1
            }
        }
    }

    /// Adds an output with full control.
    #[no_mangle]
    pub extern "C" fn circuit_builder_with_output(
        builder: *mut CircuitBuilderWrapper,
        name: *const c_char,
        io_type: *const IoTypeWrapper,
        layout: *const LayoutFunctionWrapper,
        region: *const DefinitionRegionWrapper,
    ) -> c_int {
        if builder.is_null()
            || name.is_null()
            || io_type.is_null()
            || layout.is_null()
            || region.is_null()
        {
            return -1;
        }
        let b = unsafe { &mut *builder };
        let inner = match b.0.take() {
            Some(inner) => inner,
            None => {
                set_last_error("Builder already consumed".into());
                return -1;
            }
        };
        let name_str = unsafe { CStr::from_ptr(name) }
            .to_string_lossy()
            .into_owned();
        let io_t = unsafe { &*io_type }.0.clone();
        let lay = unsafe { &*layout }.0.clone();
        let reg = unsafe { &*region }.0.clone();

        match inner.with_output(name_str, io_t, lay, reg) {
            Ok(new_builder) => {
                b.0 = Some(new_builder);
                0
            }
            Err(e) => {
                set_last_error(e);
                -1
            }
        }
    }

    /// Adds an output with full control and custom sort strategy.
    #[no_mangle]
    pub extern "C" fn circuit_builder_with_output_sorted(
        builder: *mut CircuitBuilderWrapper,
        name: *const c_char,
        io_type: *const IoTypeWrapper,
        layout: *const LayoutFunctionWrapper,
        region: *const DefinitionRegionWrapper,
        sort: *const SortStrategyWrapper,
    ) -> c_int {
        if builder.is_null()
            || name.is_null()
            || io_type.is_null()
            || layout.is_null()
            || region.is_null()
            || sort.is_null()
        {
            return -1;
        }
        let b = unsafe { &mut *builder };
        let inner = match b.0.take() {
            Some(inner) => inner,
            None => {
                set_last_error("Builder already consumed".into());
                return -1;
            }
        };
        let name_str = unsafe { CStr::from_ptr(name) }
            .to_string_lossy()
            .into_owned();
        let io_t = unsafe { &*io_type }.0.clone();
        let lay = unsafe { &*layout }.0.clone();
        let reg = unsafe { &*region }.0.clone();
        let s = unsafe { &*sort }.0.clone();

        match inner.with_output_sorted(name_str, io_t, lay, reg, s) {
            Ok(new_builder) => {
                b.0 = Some(new_builder);
                0
            }
            Err(e) => {
                set_last_error(e);
                -1
            }
        }
    }

    /// Adds an output with automatic layout inference.
    #[no_mangle]
    pub extern "C" fn circuit_builder_with_output_auto(
        builder: *mut CircuitBuilderWrapper,
        name: *const c_char,
        io_type: *const IoTypeWrapper,
        region: *const DefinitionRegionWrapper,
    ) -> c_int {
        if builder.is_null() || name.is_null() || io_type.is_null() || region.is_null() {
            return -1;
        }
        let b = unsafe { &mut *builder };
        let inner = match b.0.take() {
            Some(inner) => inner,
            None => {
                set_last_error("Builder already consumed".into());
                return -1;
            }
        };
        let name_str = unsafe { CStr::from_ptr(name) }
            .to_string_lossy()
            .into_owned();
        let io_t = unsafe { &*io_type }.0.clone();
        let reg = unsafe { &*region }.0.clone();

        match inner.with_output_auto(name_str, io_t, reg) {
            Ok(new_builder) => {
                b.0 = Some(new_builder);
                0
            }
            Err(e) => {
                set_last_error(e);
                -1
            }
        }
    }

    /// Adds an output with automatic layout inference and custom sort.
    #[no_mangle]
    pub extern "C" fn circuit_builder_with_output_auto_sorted(
        builder: *mut CircuitBuilderWrapper,
        name: *const c_char,
        io_type: *const IoTypeWrapper,
        region: *const DefinitionRegionWrapper,
        sort: *const SortStrategyWrapper,
    ) -> c_int {
        if builder.is_null()
            || name.is_null()
            || io_type.is_null()
            || region.is_null()
            || sort.is_null()
        {
            return -1;
        }
        let b = unsafe { &mut *builder };
        let inner = match b.0.take() {
            Some(inner) => inner,
            None => {
                set_last_error("Builder already consumed".into());
                return -1;
            }
        };
        let name_str = unsafe { CStr::from_ptr(name) }
            .to_string_lossy()
            .into_owned();
        let io_t = unsafe { &*io_type }.0.clone();
        let reg = unsafe { &*region }.0.clone();
        let s = unsafe { &*sort }.0.clone();

        match inner.with_output_auto_sorted(name_str, io_t, reg, s) {
            Ok(new_builder) => {
                b.0 = Some(new_builder);
                0
            }
            Err(e) => {
                set_last_error(e);
                -1
            }
        }
    }

    /// Sets simulation options on the builder.
    /// `optimize`: 0=false, non-zero=true. `io_only`: 0=false, non-zero=true.
    #[no_mangle]
    pub extern "C" fn circuit_builder_with_options(
        builder: *mut CircuitBuilderWrapper,
        optimize: c_int,
        io_only: c_int,
    ) -> c_int {
        if builder.is_null() {
            return -1;
        }
        let b = unsafe { &mut *builder };
        let inner = match b.0.take() {
            Some(inner) => inner,
            None => {
                set_last_error("Builder already consumed".into());
                return -1;
            }
        };
        let options = SimulationOptions {
            optimize: optimize != 0,
            io_only: io_only != 0,
            custom_io: Vec::new(),
        };
        b.0 = Some(inner.with_options(options));
        0
    }

    /// Sets the state mode. `mode` is one of "stateless", "stateful", "manual".
    #[no_mangle]
    pub extern "C" fn circuit_builder_with_state_mode(
        builder: *mut CircuitBuilderWrapper,
        mode: *const c_char,
    ) -> c_int {
        if builder.is_null() || mode.is_null() {
            return -1;
        }
        let b = unsafe { &mut *builder };
        let inner = match b.0.take() {
            Some(inner) => inner,
            None => {
                set_last_error("Builder already consumed".into());
                return -1;
            }
        };
        let mode_str = unsafe { CStr::from_ptr(mode) }.to_string_lossy();
        let state_mode = match parse_state_mode(&mode_str) {
            Some(m) => m,
            None => {
                set_last_error(format!("Unknown state mode: {}", mode_str));
                return -1;
            }
        };
        b.0 = Some(inner.with_state_mode(state_mode));
        0
    }

    /// Validates the circuit builder configuration.
    /// Returns 0 if valid, -1 on validation error (check `schematic_last_error`).
    #[no_mangle]
    pub extern "C" fn circuit_builder_validate(builder: *const CircuitBuilderWrapper) -> c_int {
        if builder.is_null() {
            return -1;
        }
        let b = unsafe { &*builder };
        match &b.0 {
            Some(inner) => match inner.validate() {
                Ok(_) => 0,
                Err(e) => {
                    set_last_error(e.to_string());
                    -1
                }
            },
            None => {
                set_last_error("Builder already consumed".into());
                -1
            }
        }
    }

    /// Builds the TypedCircuitExecutor. Consumes the builder.
    /// Returns null on error.
    #[no_mangle]
    pub extern "C" fn circuit_builder_build(
        builder: *mut CircuitBuilderWrapper,
    ) -> *mut TypedCircuitExecutorWrapper {
        if builder.is_null() {
            return ptr::null_mut();
        }
        let b = unsafe { &mut *builder };
        let inner = match b.0.take() {
            Some(inner) => inner,
            None => {
                set_last_error("Builder already consumed".into());
                return ptr::null_mut();
            }
        };
        match inner.build() {
            Ok(executor) => {
                let ptr = Box::into_raw(Box::new(executor));
                Box::into_raw(Box::new(TypedCircuitExecutorWrapper(ptr)))
            }
            Err(e) => {
                set_last_error(e);
                ptr::null_mut()
            }
        }
    }

    /// Builds the TypedCircuitExecutor with validation. Consumes the builder.
    /// Returns null on error.
    #[no_mangle]
    pub extern "C" fn circuit_builder_build_validated(
        builder: *mut CircuitBuilderWrapper,
    ) -> *mut TypedCircuitExecutorWrapper {
        if builder.is_null() {
            return ptr::null_mut();
        }
        let b = unsafe { &mut *builder };
        let inner = match b.0.take() {
            Some(inner) => inner,
            None => {
                set_last_error("Builder already consumed".into());
                return ptr::null_mut();
            }
        };
        match inner.build_validated() {
            Ok(executor) => {
                let ptr = Box::into_raw(Box::new(executor));
                Box::into_raw(Box::new(TypedCircuitExecutorWrapper(ptr)))
            }
            Err(e) => {
                set_last_error(e);
                ptr::null_mut()
            }
        }
    }

    #[no_mangle]
    pub extern "C" fn circuit_builder_input_count(builder: *const CircuitBuilderWrapper) -> usize {
        if builder.is_null() {
            return 0;
        }
        match &unsafe { &*builder }.0 {
            Some(inner) => inner.input_count(),
            None => 0,
        }
    }

    #[no_mangle]
    pub extern "C" fn circuit_builder_output_count(builder: *const CircuitBuilderWrapper) -> usize {
        if builder.is_null() {
            return 0;
        }
        match &unsafe { &*builder }.0 {
            Some(inner) => inner.output_count(),
            None => 0,
        }
    }

    /// Returns input names as JSON array. Caller must free with `schematic_free_string`.
    #[no_mangle]
    pub extern "C" fn circuit_builder_input_names(
        builder: *const CircuitBuilderWrapper,
    ) -> *mut c_char {
        if builder.is_null() {
            return ptr::null_mut();
        }
        match &unsafe { &*builder }.0 {
            Some(inner) => {
                let names: Vec<&str> = inner.input_names();
                let json = serde_json::to_string(&names).unwrap_or_else(|_| "[]".to_string());
                CString::new(json).unwrap_or_default().into_raw()
            }
            None => ptr::null_mut(),
        }
    }

    /// Returns output names as JSON array. Caller must free with `schematic_free_string`.
    #[no_mangle]
    pub extern "C" fn circuit_builder_output_names(
        builder: *const CircuitBuilderWrapper,
    ) -> *mut c_char {
        if builder.is_null() {
            return ptr::null_mut();
        }
        match &unsafe { &*builder }.0 {
            Some(inner) => {
                let names: Vec<&str> = inner.output_names();
                let json = serde_json::to_string(&names).unwrap_or_else(|_| "[]".to_string());
                CString::new(json).unwrap_or_default().into_raw()
            }
            None => ptr::null_mut(),
        }
    }

    #[no_mangle]
    pub extern "C" fn circuit_builder_free(ptr: *mut CircuitBuilderWrapper) {
        if !ptr.is_null() {
            unsafe {
                let _ = Box::from_raw(ptr);
            }
        }
    }

    // =========================================================================
    // TypedCircuitExecutor FFI Bindings
    // =========================================================================

    /// Creates a TypedCircuitExecutor from a MchprsWorld and IoLayout.
    /// Takes ownership of world's data (clones internally).
    /// Returns null on error.
    #[no_mangle]
    pub extern "C" fn typed_executor_from_layout(
        world: *const MchprsWorldWrapper,
        layout: *const IoLayoutWrapper,
    ) -> *mut TypedCircuitExecutorWrapper {
        if world.is_null() || layout.is_null() {
            return ptr::null_mut();
        }
        let w = unsafe { &*(*world).0 };
        let l = unsafe { &*layout };
        // We need to create a new MchprsWorld from the schematic
        let schematic = w.get_schematic().clone();
        match MchprsWorld::new(schematic) {
            Ok(new_world) => {
                let executor = TypedCircuitExecutor::from_layout(new_world, l.0.clone());
                let ptr = Box::into_raw(Box::new(executor));
                Box::into_raw(Box::new(TypedCircuitExecutorWrapper(ptr)))
            }
            Err(e) => {
                set_last_error(e);
                ptr::null_mut()
            }
        }
    }

    /// Creates a TypedCircuitExecutor from a MchprsWorld and IoLayout with simulation options.
    #[no_mangle]
    pub extern "C" fn typed_executor_from_layout_with_options(
        world: *const MchprsWorldWrapper,
        layout: *const IoLayoutWrapper,
        optimize: c_int,
        io_only: c_int,
    ) -> *mut TypedCircuitExecutorWrapper {
        if world.is_null() || layout.is_null() {
            return ptr::null_mut();
        }
        let w = unsafe { &*(*world).0 };
        let l = unsafe { &*layout };
        let schematic = w.get_schematic().clone();
        let options = SimulationOptions {
            optimize: optimize != 0,
            io_only: io_only != 0,
            custom_io: Vec::new(),
        };
        match MchprsWorld::with_options(schematic, options.clone()) {
            Ok(new_world) => {
                let executor =
                    TypedCircuitExecutor::from_layout_with_options(new_world, l.0.clone(), options);
                let ptr = Box::into_raw(Box::new(executor));
                Box::into_raw(Box::new(TypedCircuitExecutorWrapper(ptr)))
            }
            Err(e) => {
                set_last_error(e);
                ptr::null_mut()
            }
        }
    }

    /// Creates a TypedCircuitExecutor from Insign annotations in a schematic.
    /// Returns null on error.
    #[no_mangle]
    pub extern "C" fn typed_executor_from_insign(
        schematic: *const SchematicWrapper,
    ) -> *mut TypedCircuitExecutorWrapper {
        if schematic.is_null() {
            return ptr::null_mut();
        }
        let s = unsafe { &*(*schematic).0 };
        match crate::simulation::circuit_builder::create_circuit_from_insign(s) {
            Ok(executor) => {
                let ptr = Box::into_raw(Box::new(executor));
                Box::into_raw(Box::new(TypedCircuitExecutorWrapper(ptr)))
            }
            Err(e) => {
                set_last_error(e);
                ptr::null_mut()
            }
        }
    }

    /// Creates a TypedCircuitExecutor from Insign annotations with options.
    #[no_mangle]
    pub extern "C" fn typed_executor_from_insign_with_options(
        schematic: *const SchematicWrapper,
        optimize: c_int,
        io_only: c_int,
    ) -> *mut TypedCircuitExecutorWrapper {
        if schematic.is_null() {
            return ptr::null_mut();
        }
        let s = unsafe { &*(*schematic).0 };
        let options = SimulationOptions {
            optimize: optimize != 0,
            io_only: io_only != 0,
            custom_io: Vec::new(),
        };
        match crate::simulation::circuit_builder::create_circuit_from_insign_with_options(
            s, options,
        ) {
            Ok(executor) => {
                let ptr = Box::into_raw(Box::new(executor));
                Box::into_raw(Box::new(TypedCircuitExecutorWrapper(ptr)))
            }
            Err(e) => {
                set_last_error(e);
                ptr::null_mut()
            }
        }
    }

    /// Sets the state mode. `mode` is "stateless", "stateful", or "manual".
    #[no_mangle]
    pub extern "C" fn typed_executor_set_state_mode(
        executor: *mut TypedCircuitExecutorWrapper,
        mode: *const c_char,
    ) -> c_int {
        if executor.is_null() || mode.is_null() {
            return -1;
        }
        let e = unsafe { &mut *(*executor).0 };
        let mode_str = unsafe { CStr::from_ptr(mode) }.to_string_lossy();
        match parse_state_mode(&mode_str) {
            Some(m) => {
                e.set_state_mode(m);
                0
            }
            None => {
                set_last_error(format!("Unknown state mode: {}", mode_str));
                -1
            }
        }
    }

    /// Resets the executor to its initial state.
    #[no_mangle]
    pub extern "C" fn typed_executor_reset(executor: *mut TypedCircuitExecutorWrapper) -> c_int {
        if executor.is_null() {
            return -1;
        }
        let e = unsafe { &mut *(*executor).0 };
        match e.reset() {
            Ok(()) => 0,
            Err(err) => {
                set_last_error(err);
                -1
            }
        }
    }

    /// Advances the simulation by the specified number of ticks.
    #[no_mangle]
    pub extern "C" fn typed_executor_tick(
        executor: *mut TypedCircuitExecutorWrapper,
        ticks: u32,
    ) -> c_int {
        if executor.is_null() {
            return -1;
        }
        let e = unsafe { &mut *(*executor).0 };
        e.tick(ticks);
        0
    }

    /// Flushes pending changes.
    #[no_mangle]
    pub extern "C" fn typed_executor_flush(executor: *mut TypedCircuitExecutorWrapper) -> c_int {
        if executor.is_null() {
            return -1;
        }
        let e = unsafe { &mut *(*executor).0 };
        e.flush();
        0
    }

    /// Sets a single input value. Returns 0 on success, -1 on error.
    #[no_mangle]
    pub extern "C" fn typed_executor_set_input(
        executor: *mut TypedCircuitExecutorWrapper,
        name: *const c_char,
        value: *const ValueWrapper,
    ) -> c_int {
        if executor.is_null() || name.is_null() || value.is_null() {
            return -1;
        }
        let e = unsafe { &mut *(*executor).0 };
        let name_str = unsafe { CStr::from_ptr(name) }.to_string_lossy();
        let v = &unsafe { &*value }.0;
        match e.set_input(&name_str, v) {
            Ok(()) => 0,
            Err(err) => {
                set_last_error(err);
                -1
            }
        }
    }

    /// Reads a single output value. Returns null on error.
    /// Caller must free with `value_free`.
    #[no_mangle]
    pub extern "C" fn typed_executor_read_output(
        executor: *mut TypedCircuitExecutorWrapper,
        name: *const c_char,
    ) -> *mut ValueWrapper {
        if executor.is_null() || name.is_null() {
            return ptr::null_mut();
        }
        let e = unsafe { &mut *(*executor).0 };
        let name_str = unsafe { CStr::from_ptr(name) }.to_string_lossy();
        match e.read_output(&name_str) {
            Ok(value) => Box::into_raw(Box::new(ValueWrapper(value))),
            Err(err) => {
                set_last_error(err);
                ptr::null_mut()
            }
        }
    }

    /// Executes the circuit with given inputs and execution mode.
    /// `inputs_json` is a JSON object like `{"input_name": {"type": "u32", "value": 42}}`.
    /// Returns a JSON string with the execution result, or null on error.
    /// Caller must free the returned string with `schematic_free_string`.
    #[no_mangle]
    pub extern "C" fn typed_executor_execute(
        executor: *mut TypedCircuitExecutorWrapper,
        inputs_json: *const c_char,
        mode: *const ExecutionModeWrapper,
    ) -> *mut c_char {
        if executor.is_null() || inputs_json.is_null() || mode.is_null() {
            return ptr::null_mut();
        }
        let e = unsafe { &mut *(*executor).0 };
        let json_str = unsafe { CStr::from_ptr(inputs_json) }.to_string_lossy();
        let exec_mode = unsafe { &*mode }.0.clone();

        // Parse inputs JSON
        let inputs = match parse_inputs_json(&json_str) {
            Ok(inputs) => inputs,
            Err(err) => {
                set_last_error(err);
                return ptr::null_mut();
            }
        };

        // Execute
        match e.execute(inputs, exec_mode) {
            Ok(result) => {
                // Serialize result to JSON
                let json = serialize_execution_result(&result);
                CString::new(json).unwrap_or_default().into_raw()
            }
            Err(err) => {
                set_last_error(err);
                ptr::null_mut()
            }
        }
    }

    /// Returns input names as JSON array. Caller must free with `schematic_free_string`.
    #[no_mangle]
    pub extern "C" fn typed_executor_input_names(
        executor: *const TypedCircuitExecutorWrapper,
    ) -> *mut c_char {
        if executor.is_null() {
            return ptr::null_mut();
        }
        let e = unsafe { &*(*executor).0 };
        let names: Vec<&str> = e.input_names();
        let json = serde_json::to_string(&names).unwrap_or_else(|_| "[]".to_string());
        CString::new(json).unwrap_or_default().into_raw()
    }

    /// Returns output names as JSON array. Caller must free with `schematic_free_string`.
    #[no_mangle]
    pub extern "C" fn typed_executor_output_names(
        executor: *const TypedCircuitExecutorWrapper,
    ) -> *mut c_char {
        if executor.is_null() {
            return ptr::null_mut();
        }
        let e = unsafe { &*(*executor).0 };
        let names: Vec<&str> = e.output_names();
        let json = serde_json::to_string(&names).unwrap_or_else(|_| "[]".to_string());
        CString::new(json).unwrap_or_default().into_raw()
    }

    /// Returns layout info as JSON. Caller must free with `schematic_free_string`.
    #[no_mangle]
    pub extern "C" fn typed_executor_get_layout_info(
        executor: *const TypedCircuitExecutorWrapper,
    ) -> *mut c_char {
        if executor.is_null() {
            return ptr::null_mut();
        }
        let e = unsafe { &*(*executor).0 };
        let info = e.get_layout_info();
        let json = serialize_layout_info(&info);
        CString::new(json).unwrap_or_default().into_raw()
    }

    /// Syncs the simulation state to a new schematic and returns it.
    /// Caller must free with `schematic_free`.
    #[no_mangle]
    pub extern "C" fn typed_executor_sync_to_schematic(
        executor: *mut TypedCircuitExecutorWrapper,
    ) -> *mut SchematicWrapper {
        if executor.is_null() {
            return ptr::null_mut();
        }
        let e = unsafe { &mut *(*executor).0 };
        let schematic = e.sync_and_get_schematic().clone();
        let boxed = Box::into_raw(Box::new(schematic));
        Box::into_raw(Box::new(SchematicWrapper(boxed)))
    }

    #[no_mangle]
    pub extern "C" fn typed_executor_free(ptr: *mut TypedCircuitExecutorWrapper) {
        if !ptr.is_null() {
            unsafe {
                let wrapper = Box::from_raw(ptr);
                let _ = Box::from_raw(wrapper.0);
            }
        }
    }

    // =========================================================================
    // Additional MchprsWorld Methods
    // =========================================================================

    /// Gets the redstone power level at a position.
    /// Returns 0-15, or 0 on null pointer.
    #[no_mangle]
    pub extern "C" fn mchprs_world_get_redstone_power(
        world: *const MchprsWorldWrapper,
        x: c_int,
        y: c_int,
        z: c_int,
    ) -> u8 {
        if world.is_null() {
            return 0;
        }
        let w = unsafe { &*(*world).0 };
        w.get_redstone_power(BlockPos::new(x, y, z))
    }

    /// Checks for custom IO changes since last check.
    /// Must be called before `poll_custom_io_changes` to detect changes.
    /// Returns 0 on success, -1 on null pointer.
    #[no_mangle]
    pub extern "C" fn mchprs_world_check_custom_io_changes(
        world: *mut MchprsWorldWrapper,
    ) -> c_int {
        if world.is_null() {
            return -1;
        }
        let w = unsafe { &mut *(*world).0 };
        w.check_custom_io_changes();
        0
    }

    /// Returns queued custom IO changes as a JSON array and clears the queue.
    /// JSON format: `[{"x":0,"y":0,"z":0,"old_power":0,"new_power":15}, ...]`
    /// Caller must free with `schematic_free_string`.
    #[no_mangle]
    pub extern "C" fn mchprs_world_poll_custom_io_changes(
        world: *mut MchprsWorldWrapper,
    ) -> *mut c_char {
        if world.is_null() {
            return ptr::null_mut();
        }
        let w = unsafe { &mut *(*world).0 };
        let changes = w.poll_custom_io_changes();
        let json = serialize_custom_io_changes(&changes);
        CString::new(json).unwrap_or_default().into_raw()
    }

    /// Returns queued custom IO changes as JSON without clearing the queue.
    /// Caller must free with `schematic_free_string`.
    #[no_mangle]
    pub extern "C" fn mchprs_world_peek_custom_io_changes(
        world: *const MchprsWorldWrapper,
    ) -> *mut c_char {
        if world.is_null() {
            return ptr::null_mut();
        }
        let w = unsafe { &*(*world).0 };
        let changes = w.peek_custom_io_changes();
        let json = serialize_custom_io_changes(changes);
        CString::new(json).unwrap_or_default().into_raw()
    }

    /// Clears all queued custom IO changes.
    /// Returns 0 on success, -1 on null pointer.
    #[no_mangle]
    pub extern "C" fn mchprs_world_clear_custom_io_changes(
        world: *mut MchprsWorldWrapper,
    ) -> c_int {
        if world.is_null() {
            return -1;
        }
        let w = unsafe { &mut *(*world).0 };
        w.clear_custom_io_changes();
        0
    }

    // =========================================================================
    // Helper Functions
    // =========================================================================

    /// Parse a flat [x,y,z,x,y,z,...] array into Vec<(i32,i32,i32)>.
    fn parse_positions(positions: *const c_int, count: usize) -> Vec<(i32, i32, i32)> {
        if positions.is_null() || count == 0 {
            return Vec::new();
        }
        let coords = unsafe { std::slice::from_raw_parts(positions, count * 3) };
        coords.chunks_exact(3).map(|c| (c[0], c[1], c[2])).collect()
    }

    /// Parse a state mode string.
    fn parse_state_mode(s: &str) -> Option<StateMode> {
        match s.to_lowercase().as_str() {
            "stateless" => Some(StateMode::Stateless),
            "stateful" => Some(StateMode::Stateful),
            "manual" => Some(StateMode::Manual),
            _ => None,
        }
    }

    /// Parse inputs JSON to HashMap<String, Value>.
    /// Format: `{"name": {"type": "u32", "value": 42}, ...}`
    fn parse_inputs_json(json: &str) -> Result<HashMap<String, Value>, String> {
        let parsed: serde_json::Value =
            serde_json::from_str(json).map_err(|e| format!("Invalid JSON: {}", e))?;

        let obj = parsed
            .as_object()
            .ok_or_else(|| "Inputs must be a JSON object".to_string())?;

        let mut inputs = HashMap::new();
        for (name, val) in obj {
            let value = parse_json_value(val)?;
            inputs.insert(name.clone(), value);
        }
        Ok(inputs)
    }

    /// Parse a single JSON value to a Value.
    /// Supports: `{"type": "u32", "value": 42}` or shorthand: just `42` (inferred as u32/i32/f32/bool/string).
    fn parse_json_value(v: &serde_json::Value) -> Result<Value, String> {
        // Try typed format first: {"type": "...", "value": ...}
        if let Some(obj) = v.as_object() {
            if let (Some(type_val), Some(value_val)) = (obj.get("type"), obj.get("value")) {
                if let Some(type_str) = type_val.as_str() {
                    return match type_str {
                        "u32" => {
                            let n = value_val
                                .as_u64()
                                .ok_or("Expected unsigned integer for u32")?;
                            Ok(Value::U32(n as u32))
                        }
                        "i32" => {
                            let n = value_val.as_i64().ok_or("Expected integer for i32")?;
                            Ok(Value::I32(n as i32))
                        }
                        "f32" => {
                            let n = value_val.as_f64().ok_or("Expected number for f32")?;
                            Ok(Value::F32(n as f32))
                        }
                        "bool" => {
                            let b = value_val.as_bool().ok_or("Expected boolean for bool")?;
                            Ok(Value::Bool(b))
                        }
                        "string" => {
                            let s = value_val.as_str().ok_or("Expected string for string")?;
                            Ok(Value::String(s.to_string()))
                        }
                        _ => Err(format!("Unknown value type: {}", type_str)),
                    };
                }
            }
        }

        // Shorthand: infer type from JSON value
        match v {
            serde_json::Value::Bool(b) => Ok(Value::Bool(*b)),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    if i >= 0 {
                        Ok(Value::U32(i as u32))
                    } else {
                        Ok(Value::I32(i as i32))
                    }
                } else if let Some(f) = n.as_f64() {
                    Ok(Value::F32(f as f32))
                } else {
                    Err("Cannot parse number".to_string())
                }
            }
            serde_json::Value::String(s) => Ok(Value::String(s.clone())),
            _ => Err(format!("Cannot convert JSON value to Value: {}", v)),
        }
    }

    /// Serialize a Value to a serde_json::Value.
    fn value_to_json(v: &Value) -> serde_json::Value {
        match v {
            Value::U32(n) => serde_json::json!({"type": "u32", "value": n}),
            Value::U64(n) => serde_json::json!({"type": "u64", "value": n}),
            Value::I32(n) => serde_json::json!({"type": "i32", "value": n}),
            Value::I64(n) => serde_json::json!({"type": "i64", "value": n}),
            Value::F32(n) => serde_json::json!({"type": "f32", "value": n}),
            Value::Bool(b) => serde_json::json!({"type": "bool", "value": b}),
            Value::String(s) => serde_json::json!({"type": "string", "value": s}),
            Value::BitArray(bits) => serde_json::json!({"type": "bit_array", "value": bits}),
            Value::Bytes(bytes) => serde_json::json!({"type": "bytes", "value": bytes}),
            Value::Array(arr) => {
                let vals: Vec<serde_json::Value> = arr.iter().map(value_to_json).collect();
                serde_json::json!({"type": "array", "value": vals})
            }
            Value::Struct(fields) => {
                let obj: serde_json::Map<String, serde_json::Value> = fields
                    .iter()
                    .map(|(k, v)| (k.clone(), value_to_json(v)))
                    .collect();
                serde_json::json!({"type": "struct", "value": obj})
            }
        }
    }

    /// Serialize an ExecutionResult to JSON string.
    fn serialize_execution_result(result: &ExecutionResult) -> String {
        let mut outputs = serde_json::Map::new();
        for (name, value) in &result.outputs {
            outputs.insert(name.clone(), value_to_json(value));
        }
        let json = serde_json::json!({
            "outputs": outputs,
            "ticks_elapsed": result.ticks_elapsed,
            "condition_met": result.condition_met,
        });
        serde_json::to_string(&json).unwrap_or_else(|_| "{}".to_string())
    }

    /// Serialize LayoutInfo to JSON string.
    fn serialize_layout_info(info: &LayoutInfo) -> String {
        let mut inputs = serde_json::Map::new();
        for (name, li) in &info.inputs {
            let positions: Vec<Vec<i32>> = li
                .positions
                .iter()
                .map(|&(x, y, z)| vec![x, y, z])
                .collect();
            inputs.insert(
                name.clone(),
                serde_json::json!({
                    "io_type": li.io_type,
                    "positions": positions,
                    "bit_count": li.bit_count,
                }),
            );
        }
        let mut outputs = serde_json::Map::new();
        for (name, li) in &info.outputs {
            let positions: Vec<Vec<i32>> = li
                .positions
                .iter()
                .map(|&(x, y, z)| vec![x, y, z])
                .collect();
            outputs.insert(
                name.clone(),
                serde_json::json!({
                    "io_type": li.io_type,
                    "positions": positions,
                    "bit_count": li.bit_count,
                }),
            );
        }
        let json = serde_json::json!({
            "inputs": inputs,
            "outputs": outputs,
        });
        serde_json::to_string(&json).unwrap_or_else(|_| "{}".to_string())
    }

    /// Serialize custom IO changes to JSON array string.
    fn serialize_custom_io_changes(changes: &[CustomIoChange]) -> String {
        let arr: Vec<serde_json::Value> = changes
            .iter()
            .map(|c| {
                serde_json::json!({
                    "x": c.x,
                    "y": c.y,
                    "z": c.z,
                    "old_power": c.old_power,
                    "new_power": c.new_power,
                })
            })
            .collect();
        serde_json::to_string(&arr).unwrap_or_else(|_| "[]".to_string())
    }
}

// ---------------------------------------------------------------------------
// Redstone graph: extraction + analysis (feature `simulation`)
// ---------------------------------------------------------------------------
/// Opaque handle to an extracted redstone logic graph.
/// Free with `redstonegraph_free`.
#[cfg(feature = "simulation")]
pub struct RedstoneGraphWrapper(*mut crate::simulation::graph::RedstoneGraph);

/// Extract the compiled redstone logic graph for a world.
/// Returns null on error. Caller frees with `redstonegraph_free`.
#[cfg(feature = "simulation")]
#[no_mangle]
pub extern "C" fn mchprs_world_export_graph(
    world: *const MchprsWorldWrapper,
) -> *mut RedstoneGraphWrapper {
    if world.is_null() {
        return ptr::null_mut();
    }
    let w = unsafe { &*(*world).0 };
    match w.export_graph() {
        Ok(g) => Box::into_raw(Box::new(RedstoneGraphWrapper(Box::into_raw(Box::new(g))))),
        Err(_) => ptr::null_mut(),
    }
}

/// Extract the structural (pre-fold, as-built) redstone logic graph for a world.
/// Returns null on error. Caller frees with `redstonegraph_free`.
#[cfg(feature = "simulation")]
#[no_mangle]
pub extern "C" fn mchprs_world_export_graph_structural(
    world: *const MchprsWorldWrapper,
) -> *mut RedstoneGraphWrapper {
    if world.is_null() {
        return ptr::null_mut();
    }
    let w = unsafe { &*(*world).0 };
    match w.export_graph_structural() {
        Ok(g) => Box::into_raw(Box::new(RedstoneGraphWrapper(Box::into_raw(Box::new(g))))),
        Err(_) => ptr::null_mut(),
    }
}

/// Frees a RedstoneGraph handle.
#[cfg(feature = "simulation")]
#[no_mangle]
pub extern "C" fn redstonegraph_free(graph: *mut RedstoneGraphWrapper) {
    if !graph.is_null() {
        unsafe {
            let w = Box::from_raw(graph);
            if !w.0.is_null() {
                let _ = Box::from_raw(w.0);
            }
        }
    }
}

/// Number of nodes in the graph (0 on null).
#[cfg(feature = "simulation")]
#[no_mangle]
pub extern "C" fn redstonegraph_node_count(graph: *const RedstoneGraphWrapper) -> usize {
    if graph.is_null() {
        return 0;
    }
    unsafe { (*(*graph).0).node_count() }
}

/// Total number of directed edges in the graph (0 on null).
#[cfg(feature = "simulation")]
#[no_mangle]
pub extern "C" fn redstonegraph_edge_count(graph: *const RedstoneGraphWrapper) -> usize {
    if graph.is_null() {
        return 0;
    }
    unsafe { (*(*graph).0).edge_count() }
}

/// The nodes as a JSON array string (free with `free_string`). Null on error.
#[cfg(feature = "simulation")]
#[no_mangle]
pub extern "C" fn redstonegraph_nodes(graph: *const RedstoneGraphWrapper) -> *mut c_char {
    if graph.is_null() {
        return ptr::null_mut();
    }
    let g = unsafe { &*(*graph).0 };
    match g.nodes_json() {
        Ok(json) => CString::new(json).unwrap_or_default().into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

/// The directed edges as a JSON array string (free with `free_string`). Null on error.
#[cfg(feature = "simulation")]
#[no_mangle]
pub extern "C" fn redstonegraph_edges(graph: *const RedstoneGraphWrapper) -> *mut c_char {
    if graph.is_null() {
        return ptr::null_mut();
    }
    let g = unsafe { &*(*graph).0 };
    match g.edges_json() {
        Ok(json) => CString::new(json).unwrap_or_default().into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

/// Computed graph features as a JSON object string (free with `free_string`). Null on error.
#[cfg(feature = "simulation")]
#[no_mangle]
pub extern "C" fn redstonegraph_features(graph: *const RedstoneGraphWrapper) -> *mut c_char {
    if graph.is_null() {
        return ptr::null_mut();
    }
    let g = unsafe { &*(*graph).0 };
    match g.features().to_json() {
        Ok(json) => CString::new(json).unwrap_or_default().into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

/// Computed graph features as a JSON string (alias of `redstonegraph_features`;
/// free with `free_string`). Null on error.
#[cfg(feature = "simulation")]
#[no_mangle]
pub extern "C" fn redstonegraph_features_json(graph: *const RedstoneGraphWrapper) -> *mut c_char {
    redstonegraph_features(graph)
}

/// Fingerprint (hex string) for `preset` ("structural"/"functional"/"exact";
/// null/empty defaults to "structural"). Free with `free_string`. Null on error.
#[cfg(feature = "simulation")]
#[no_mangle]
pub extern "C" fn redstonegraph_fingerprint(
    graph: *const RedstoneGraphWrapper,
    preset: *const c_char,
) -> *mut c_char {
    if graph.is_null() {
        return ptr::null_mut();
    }
    let g = unsafe { &*(*graph).0 };
    let preset = if preset.is_null() {
        "structural".to_string()
    } else {
        unsafe { CStr::from_ptr(preset) }
            .to_str()
            .unwrap_or("structural")
            .to_string()
    };
    let preset = if preset.is_empty() {
        "structural".to_string()
    } else {
        preset
    };
    match crate::simulation::fingerprint::GraphFingerprintSpec::from_preset(&preset) {
        Some(spec) => CString::new(g.fingerprint(&spec).to_hex())
            .unwrap_or_default()
            .into_raw(),
        None => ptr::null_mut(),
    }
}

/// Serialize the graph to JSON (free with `free_string`). Null on error.
#[cfg(feature = "simulation")]
#[no_mangle]
pub extern "C" fn redstonegraph_to_json(graph: *const RedstoneGraphWrapper) -> *mut c_char {
    if graph.is_null() {
        return ptr::null_mut();
    }
    let g = unsafe { &*(*graph).0 };
    match g.to_json() {
        Ok(json) => CString::new(json).unwrap_or_default().into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

/// Deserialize a graph from a JSON string. Returns null on error.
/// Caller frees with `redstonegraph_free`.
#[cfg(feature = "simulation")]
#[no_mangle]
pub extern "C" fn redstonegraph_from_json(json: *const c_char) -> *mut RedstoneGraphWrapper {
    if json.is_null() {
        return ptr::null_mut();
    }
    let s = match unsafe { CStr::from_ptr(json) }.to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    match crate::simulation::graph::RedstoneGraph::from_json(s) {
        Ok(g) => Box::into_raw(Box::new(RedstoneGraphWrapper(Box::into_raw(Box::new(g))))),
        Err(_) => ptr::null_mut(),
    }
}
