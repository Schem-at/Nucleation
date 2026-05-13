//! `Shape` + `Brush` JNI exports — mirrors `PyShape` / `PyBrush`.
//!
//! Where the Rust API takes f64 tuples for centers and axis vectors, the Java
//! side passes them through; convenience char-axis helpers live on the Java
//! side and translate to `(0,1,0)` / `(1,0,0)` / `(0,0,1)` unit vectors.

use crate::conv::jstring_to_string;
use crate::errors::{with_jni_context, JvmError};
use crate::handles::{as_ref, consume, to_handle};
use jni::objects::{JClass, JString};
use jni::sys::{jdouble, jint, jlong};
use jni::{JNIEnv, NativeMethod};
use nucleation::building::{
    BezierCurve, BrushEnum, ColorBrush, Cone, Cuboid, Cylinder, Difference, Disk, Ellipsoid,
    Hollow, Intersection, Line, Plane, Pyramid, ShapeEnum, SolidBrush, Sphere, Torus, Triangle,
    Union,
};
use nucleation::BlockState;
use std::ffi::c_void;

const NATIVE_CLASS: &str = "com/github/schemat/nucleation/NucleationNative";

pub fn register(env: &mut JNIEnv) -> jni::errors::Result<()> {
    let class = env.find_class(NATIVE_CLASS)?;
    let methods: &[NativeMethod] = &[
        nm("nShapeFree", "(J)V", n_shape_free as *mut _),
        nm("nShapeSphere", "(IIID)J", n_sphere as *mut _),
        nm("nShapeCuboid", "(IIIIII)J", n_cuboid as *mut _),
        nm("nShapeEllipsoid", "(IIIDDD)J", n_ellipsoid as *mut _),
        nm("nShapeCylinder", "(DDDDDDDD)J", n_cylinder as *mut _),
        nm("nShapeCone", "(DDDDDDDD)J", n_cone as *mut _),
        nm("nShapeTorus", "(DDDDDDDD)J", n_torus as *mut _),
        nm("nShapePyramid", "(DDDDDDDDD)J", n_pyramid as *mut _),
        nm("nShapeDisk", "(DDDDDDDD)J", n_disk as *mut _),
        nm("nShapePlane", "(DDDDDDDDDDDD)J", n_plane as *mut _),
        nm("nShapeTriangle", "(DDDDDDDDDD)J", n_triangle as *mut _),
        nm("nShapeLine", "(DDDDDDD)J", n_line as *mut _),
        nm("nShapeBezier", "([DDI)J", n_bezier as *mut _),
        nm("nShapeUnion", "(JJ)J", n_union as *mut _),
        nm("nShapeIntersection", "(JJ)J", n_intersection as *mut _),
        nm("nShapeDifference", "(JJ)J", n_difference as *mut _),
        nm("nShapeHollow", "(JI)J", n_hollow as *mut _),
        nm("nShapeContains", "(JIII)Z", n_contains as *mut _),
        nm("nShapeBounds", "(J)[I", n_bounds as *mut _),
        nm("nBrushFree", "(J)V", n_brush_free as *mut _),
        nm("nBrushSolid", "(Ljava/lang/String;)J", n_brush_solid as *mut _),
        nm("nBrushColor", "(III)J", n_brush_color as *mut _),
    ];
    env.register_native_methods(&class, methods)
}

fn nm(name: &str, sig: &str, ptr: *mut c_void) -> NativeMethod {
    NativeMethod {
        name: name.into(),
        sig: sig.into(),
        fn_ptr: ptr,
    }
}

unsafe extern "system" fn n_shape_free<'l>(_env: JNIEnv<'l>, _class: JClass<'l>, handle: jlong) {
    if handle != 0 {
        let _ = consume::<ShapeEnum>(handle);
    }
}

unsafe extern "system" fn n_sphere<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    cx: jint,
    cy: jint,
    cz: jint,
    radius: jdouble,
) -> jlong {
    with_jni_context(&mut env, 0, |_env| {
        Ok(to_handle(ShapeEnum::Sphere(Sphere::new(
            (cx, cy, cz),
            radius,
        ))))
    })
}

unsafe extern "system" fn n_cuboid<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    min_x: jint,
    min_y: jint,
    min_z: jint,
    max_x: jint,
    max_y: jint,
    max_z: jint,
) -> jlong {
    with_jni_context(&mut env, 0, |_env| {
        Ok(to_handle(ShapeEnum::Cuboid(Cuboid::new(
            (min_x, min_y, min_z),
            (max_x, max_y, max_z),
        ))))
    })
}

unsafe extern "system" fn n_ellipsoid<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    cx: jint,
    cy: jint,
    cz: jint,
    rx: jdouble,
    ry: jdouble,
    rz: jdouble,
) -> jlong {
    with_jni_context(&mut env, 0, |_env| {
        Ok(to_handle(ShapeEnum::Ellipsoid(Ellipsoid::new(
            (cx, cy, cz),
            (rx, ry, rz),
        ))))
    })
}

unsafe extern "system" fn n_cylinder<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    bx: jdouble,
    by: jdouble,
    bz: jdouble,
    ax: jdouble,
    ay: jdouble,
    az: jdouble,
    radius: jdouble,
    height: jdouble,
) -> jlong {
    with_jni_context(&mut env, 0, |_env| {
        Ok(to_handle(ShapeEnum::Cylinder(Cylinder::new(
            (bx, by, bz),
            (ax, ay, az),
            radius,
            height,
        ))))
    })
}

unsafe extern "system" fn n_cone<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    apx: jdouble,
    apy: jdouble,
    apz: jdouble,
    ax: jdouble,
    ay: jdouble,
    az: jdouble,
    radius: jdouble,
    height: jdouble,
) -> jlong {
    with_jni_context(&mut env, 0, |_env| {
        Ok(to_handle(ShapeEnum::Cone(Cone::new(
            (apx, apy, apz),
            (ax, ay, az),
            radius,
            height,
        ))))
    })
}

unsafe extern "system" fn n_torus<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    cx: jdouble,
    cy: jdouble,
    cz: jdouble,
    major_radius: jdouble,
    minor_radius: jdouble,
    ax: jdouble,
    ay: jdouble,
    az: jdouble,
) -> jlong {
    with_jni_context(&mut env, 0, |_env| {
        Ok(to_handle(ShapeEnum::Torus(Torus::new(
            (cx, cy, cz),
            major_radius,
            minor_radius,
            (ax, ay, az),
        ))))
    })
}

unsafe extern "system" fn n_pyramid<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    bx: jdouble,
    by: jdouble,
    bz: jdouble,
    half_x: jdouble,
    half_z: jdouble,
    height: jdouble,
    ax: jdouble,
    ay: jdouble,
    az: jdouble,
) -> jlong {
    with_jni_context(&mut env, 0, |_env| {
        Ok(to_handle(ShapeEnum::Pyramid(Pyramid::new(
            (bx, by, bz),
            (half_x, half_z),
            height,
            (ax, ay, az),
        ))))
    })
}

unsafe extern "system" fn n_disk<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    cx: jdouble,
    cy: jdouble,
    cz: jdouble,
    radius: jdouble,
    nx: jdouble,
    ny: jdouble,
    nz: jdouble,
    thickness: jdouble,
) -> jlong {
    with_jni_context(&mut env, 0, |_env| {
        Ok(to_handle(ShapeEnum::Disk(Disk::new(
            (cx, cy, cz),
            radius,
            (nx, ny, nz),
            thickness,
        ))))
    })
}

unsafe extern "system" fn n_plane<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    ox: jdouble,
    oy: jdouble,
    oz: jdouble,
    ux: jdouble,
    uy: jdouble,
    uz: jdouble,
    vx: jdouble,
    vy: jdouble,
    vz: jdouble,
    u_extent: jdouble,
    v_extent: jdouble,
    thickness: jdouble,
) -> jlong {
    with_jni_context(&mut env, 0, |_env| {
        Ok(to_handle(ShapeEnum::Plane(Plane::new(
            (ox, oy, oz),
            (ux, uy, uz),
            (vx, vy, vz),
            u_extent,
            v_extent,
            thickness,
        ))))
    })
}

unsafe extern "system" fn n_triangle<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    ax: jdouble,
    ay: jdouble,
    az: jdouble,
    bx: jdouble,
    by: jdouble,
    bz: jdouble,
    cx: jdouble,
    cy: jdouble,
    cz: jdouble,
    thickness: jdouble,
) -> jlong {
    with_jni_context(&mut env, 0, |_env| {
        Ok(to_handle(ShapeEnum::Triangle(Triangle::new(
            (ax, ay, az),
            (bx, by, bz),
            (cx, cy, cz),
            thickness,
        ))))
    })
}

unsafe extern "system" fn n_line<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    x1: jdouble,
    y1: jdouble,
    z1: jdouble,
    x2: jdouble,
    y2: jdouble,
    z2: jdouble,
    thickness: jdouble,
) -> jlong {
    with_jni_context(&mut env, 0, |_env| {
        Ok(to_handle(ShapeEnum::Line(Line::new(
            (x1, y1, z1),
            (x2, y2, z2),
            thickness,
        ))))
    })
}

unsafe extern "system" fn n_bezier<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    points: jni::objects::JDoubleArray<'l>,
    thickness: jdouble,
    resolution: jint,
) -> jlong {
    with_jni_context(&mut env, 0, |env| {
        let len = env
            .get_array_length(&points)
            .map_err(|e| JvmError::Generic(format!("array length: {e}")))?;
        if len % 3 != 0 {
            return Err(JvmError::InvalidBlockState(
                "bezier points must be flat triples: [x0,y0,z0,...]".into(),
            ));
        }
        let mut buf = vec![0f64; len as usize];
        env.get_double_array_region(&points, 0, &mut buf)
            .map_err(|e| JvmError::Generic(format!("array copy: {e}")))?;
        let ctrl: Vec<(f64, f64, f64)> = buf
            .chunks_exact(3)
            .map(|c| (c[0], c[1], c[2]))
            .collect();
        Ok(to_handle(ShapeEnum::BezierCurve(BezierCurve::new(
            ctrl,
            thickness,
            resolution.max(2) as u32,
        ))))
    })
}

unsafe extern "system" fn n_union<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    a: jlong,
    b: jlong,
) -> jlong {
    with_jni_context(&mut env, 0, |_env| {
        let a = as_ref::<ShapeEnum>(a).clone();
        let b = as_ref::<ShapeEnum>(b).clone();
        Ok(to_handle(ShapeEnum::Union(Union::new(a, b))))
    })
}

unsafe extern "system" fn n_intersection<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    a: jlong,
    b: jlong,
) -> jlong {
    with_jni_context(&mut env, 0, |_env| {
        let a = as_ref::<ShapeEnum>(a).clone();
        let b = as_ref::<ShapeEnum>(b).clone();
        Ok(to_handle(ShapeEnum::Intersection(Intersection::new(a, b))))
    })
}

unsafe extern "system" fn n_difference<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    a: jlong,
    b: jlong,
) -> jlong {
    with_jni_context(&mut env, 0, |_env| {
        let a = as_ref::<ShapeEnum>(a).clone();
        let b = as_ref::<ShapeEnum>(b).clone();
        Ok(to_handle(ShapeEnum::Difference(Difference::new(a, b))))
    })
}

unsafe extern "system" fn n_hollow<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    inner: jlong,
    thickness: jint,
) -> jlong {
    with_jni_context(&mut env, 0, |_env| {
        let inner = as_ref::<ShapeEnum>(inner).clone();
        Ok(to_handle(ShapeEnum::Hollow(Hollow::new(
            inner,
            thickness.max(0) as u32,
        ))))
    })
}

unsafe extern "system" fn n_contains<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
    x: jint,
    y: jint,
    z: jint,
) -> u8 {
    use nucleation::building::Shape;
    with_jni_context(&mut env, 0u8, |_env| {
        let s = as_ref::<ShapeEnum>(handle);
        Ok(s.contains(x, y, z) as u8)
    })
}

unsafe extern "system" fn n_bounds<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    handle: jlong,
) -> jni::sys::jintArray {
    use nucleation::building::Shape;
    with_jni_context(&mut env, std::ptr::null_mut(), |env| {
        let s = as_ref::<ShapeEnum>(handle);
        let (a, b, c, d, e, f) = s.bounds();
        let arr = env
            .new_int_array(6)
            .map_err(|err| JvmError::Generic(format!("new_int_array: {err}")))?;
        env.set_int_array_region(&arr, 0, &[a, b, c, d, e, f])
            .map_err(|err| JvmError::Generic(format!("set_int_array_region: {err}")))?;
        Ok(arr.into_raw())
    })
}

unsafe extern "system" fn n_brush_free<'l>(_env: JNIEnv<'l>, _class: JClass<'l>, handle: jlong) {
    if handle != 0 {
        let _ = consume::<BrushEnum>(handle);
    }
}

unsafe extern "system" fn n_brush_solid<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    block_name: JString<'l>,
) -> jlong {
    with_jni_context(&mut env, 0, |env| {
        let name = jstring_to_string(env, &block_name)?;
        let bs = BlockState::new(name);
        Ok(to_handle(BrushEnum::Solid(SolidBrush::new(bs))))
    })
}

unsafe extern "system" fn n_brush_color<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    r: jint,
    g: jint,
    b: jint,
) -> jlong {
    with_jni_context(&mut env, 0, |_env| {
        Ok(to_handle(BrushEnum::Color(ColorBrush::new(
            r.clamp(0, 255) as u8,
            g.clamp(0, 255) as u8,
            b.clamp(0, 255) as u8,
        ))))
    })
}
