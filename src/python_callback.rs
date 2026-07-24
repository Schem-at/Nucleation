//! Narrow C ABI used only by the handwritten Python callback adapter.
//!
//! Diplomat intentionally cannot retain foreign-language closures. This API
//! keeps the callback synchronous and borrows all objects for one transactional
//! fill, so no Python lifetime or GIL state enters the Rust object graph.

use crate::bridge::{building::ffi::Brush, schematic::ffi::Schematic};
use crate::building::BuildingTool;
use std::ffi::c_void;

#[cfg(test)]
thread_local! {
    static FORCE_INTERNAL_PANIC: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

pub type EvalCallback =
    unsafe extern "C" fn(context: *mut c_void, x: f64, y: f64, z: f64, output: *mut f64) -> bool;

pub type NormalCallback = unsafe extern "C" fn(
    context: *mut c_void,
    x: f64,
    y: f64,
    z: f64,
    output_x: *mut f64,
    output_y: *mut f64,
    output_z: *mut f64,
) -> bool;

/// Returns 0 on success, 1 for invalid arguments, 2 when a callback failed,
/// and 3 when an internal Rust panic was contained.
#[no_mangle]
pub unsafe extern "C" fn nucleation_python_fill_sdf_function(
    schematic: *mut Schematic,
    brush: *const Brush,
    min_x: i32,
    min_y: i32,
    min_z: i32,
    max_x: i32,
    max_y: i32,
    max_z: i32,
    epsilon: f64,
    context: *mut c_void,
    eval: Option<EvalCallback>,
    normal: Option<NormalCallback>,
) -> u8 {
    if schematic.is_null() || brush.is_null() || eval.is_none() {
        return 1;
    }
    let eval = eval.expect("validated above");
    if min_x > max_x || min_y > max_y || min_z > max_z || !epsilon.is_finite() || epsilon <= 0.0 {
        return 1;
    }

    let requested_min = [min_x, min_y, min_z];
    let requested_max = [max_x, max_y, max_z];
    if crate::sdf::checked_sample_volume(requested_min, requested_max).is_err() {
        return 1;
    }

    // Clone through short-lived raw-pointer dereferences. No Rust reference into
    // a Python-visible object may survive while arbitrary Python is executing.
    let entry = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
        (*schematic).0.clone()
    })) {
        Ok(value) => value,
        Err(_) => return 3,
    };

    let operation = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let brush = unsafe { (*brush).0.clone() };
        #[cfg(test)]
        if FORCE_INTERNAL_PANIC.with(|flag| flag.replace(false)) {
            panic!("injected callback ABI panic");
        }
        validate_destination_volume(&entry, requested_min, requested_max).map_err(|_| 1_u8)?;
        let mut candidate = entry.clone();
        let result = BuildingTool::new(&mut candidate).fill_sdf_function(
            (min_x, min_y, min_z, max_x, max_y, max_z),
            epsilon,
            |x, y, z| {
                let mut output = 0.0;
                if unsafe { eval(context, x, y, z, &mut output) } && output.is_finite() {
                    Ok(output)
                } else {
                    Err(())
                }
            },
            |x, y, z| {
                let Some(normal) = normal else {
                    return Ok(None);
                };
                let (mut nx, mut ny, mut nz) = (0.0, 0.0, 0.0);
                if unsafe { normal(context, x, y, z, &mut nx, &mut ny, &mut nz) }
                    && nx.is_finite()
                    && ny.is_finite()
                    && nz.is_finite()
                {
                    Ok(Some((nx, ny, nz)))
                } else {
                    Err(())
                }
            },
            &brush,
        );
        result.map_err(|_| 2_u8)?;
        Ok::<_, u8>(candidate)
    }));

    match operation {
        Ok(Ok(candidate)) => {
            unsafe { (*schematic).0 = candidate };
            0
        }
        Ok(Err(status)) => {
            unsafe { (*schematic).0 = entry };
            status
        }
        Err(_) => {
            unsafe { (*schematic).0 = entry };
            3
        }
    }
}

fn validate_destination_volume(
    schematic: &crate::UniversalSchematic,
    requested_min: [i32; 3],
    requested_max: [i32; 3],
) -> Result<u64, String> {
    if schematic.get_content_bounds().is_none() {
        return crate::sdf::checked_sample_volume(requested_min, requested_max);
    }
    let existing = schematic.get_bounding_box();
    crate::sdf::checked_sample_volume(
        [
            existing.min.0.min(requested_min[0]),
            existing.min.1.min(requested_min[1]),
            existing.min.2.min(requested_min[2]),
        ],
        [
            existing.max.0.max(requested_max[0]),
            existing.max.1.max(requested_max[1]),
            existing.max.2.max(requested_max[2]),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bridge::building::ffi::Brush;
    use crate::bridge::schematic::ffi::Schematic;
    use crate::building::{BrushEnum, SolidBrush};
    use crate::{BlockState, UniversalSchematic};

    struct CallbackContext {
        calls: usize,
        schematic: *mut Schematic,
    }

    unsafe extern "C" fn counting_eval(
        context: *mut c_void,
        _x: f64,
        _y: f64,
        _z: f64,
        output: *mut f64,
    ) -> bool {
        let context = unsafe { &mut *context.cast::<CallbackContext>() };
        context.calls += 1;
        unsafe { *output = -1.0 };
        true
    }

    unsafe extern "C" fn reentrant_failing_eval(
        context: *mut c_void,
        _x: f64,
        _y: f64,
        _z: f64,
        _output: *mut f64,
    ) -> bool {
        let context = unsafe { &mut *context.cast::<CallbackContext>() };
        context.calls += 1;
        unsafe {
            (*context.schematic).0.set_block(
                99,
                99,
                99,
                &BlockState::new("minecraft:diamond_block"),
            );
        }
        false
    }

    fn solid_brush() -> Brush {
        Brush(BrushEnum::Solid(SolidBrush::new(BlockState::new(
            "minecraft:stone",
        ))))
    }

    #[test]
    fn distant_destination_is_rejected_before_callback() {
        let mut schematic = Schematic(UniversalSchematic::new("bounds".to_string()));
        schematic
            .0
            .set_block(i32::MIN + 1, 0, 0, &BlockState::new("minecraft:gold_block"));
        let brush = solid_brush();
        let mut context = CallbackContext {
            calls: 0,
            schematic: &mut schematic,
        };
        let status = unsafe {
            nucleation_python_fill_sdf_function(
                &mut schematic,
                &brush,
                i32::MAX - 1,
                0,
                0,
                i32::MAX - 1,
                0,
                0,
                0.5,
                (&mut context as *mut CallbackContext).cast(),
                Some(counting_eval),
                None,
            )
        };
        assert_eq!(status, 1);
        assert_eq!(context.calls, 0);
        assert_eq!(schematic.0.total_blocks(), 1);
    }

    #[test]
    fn reentrant_callback_mutation_is_rolled_back_on_failure() {
        let mut schematic = Schematic(UniversalSchematic::new("reentrant".to_string()));
        schematic
            .0
            .set_block(0, 0, 0, &BlockState::new("minecraft:gold_block"));
        let brush = solid_brush();
        let mut context = CallbackContext {
            calls: 0,
            schematic: &mut schematic,
        };
        let status = unsafe {
            nucleation_python_fill_sdf_function(
                &mut schematic,
                &brush,
                -1,
                -1,
                -1,
                1,
                1,
                1,
                0.5,
                (&mut context as *mut CallbackContext).cast(),
                Some(reentrant_failing_eval),
                None,
            )
        };
        assert_eq!(status, 2);
        assert_eq!(context.calls, 1);
        assert_eq!(schematic.0.total_blocks(), 1);
        assert!(schematic.0.get_block(99, 99, 99).is_none());
        assert_eq!(
            schematic.0.get_block(0, 0, 0).unwrap().get_name(),
            "minecraft:gold_block"
        );
    }

    #[test]
    fn internal_panic_is_contained_at_the_abi() {
        let mut schematic = Schematic(UniversalSchematic::new("panic".to_string()));
        schematic
            .0
            .set_block(0, 0, 0, &BlockState::new("minecraft:gold_block"));
        let brush = solid_brush();
        let mut context = CallbackContext {
            calls: 0,
            schematic: &mut schematic,
        };
        FORCE_INTERNAL_PANIC.with(|flag| flag.set(true));
        let status = unsafe {
            nucleation_python_fill_sdf_function(
                &mut schematic,
                &brush,
                0,
                0,
                0,
                0,
                0,
                0,
                0.5,
                (&mut context as *mut CallbackContext).cast(),
                Some(counting_eval),
                None,
            )
        };
        assert_eq!(status, 3);
        assert_eq!(context.calls, 0);
        assert_eq!(schematic.0.total_blocks(), 1);
    }
}
