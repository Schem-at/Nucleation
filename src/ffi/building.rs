use super::*;

// --- Shape/Brush/BuildingTool FFI ---

pub struct ShapeWrapper(ShapeEnum);
pub struct BrushWrapper(BrushEnum);

#[no_mangle]
pub extern "C" fn shape_sphere(
    cx: c_float,
    cy: c_float,
    cz: c_float,
    radius: c_float,
) -> *mut ShapeWrapper {
    Box::into_raw(Box::new(ShapeWrapper(ShapeEnum::Sphere(Sphere::new(
        (cx as i32, cy as i32, cz as i32),
        radius as f64,
    )))))
}

#[no_mangle]
pub extern "C" fn shape_cuboid(
    min_x: c_int,
    min_y: c_int,
    min_z: c_int,
    max_x: c_int,
    max_y: c_int,
    max_z: c_int,
) -> *mut ShapeWrapper {
    Box::into_raw(Box::new(ShapeWrapper(ShapeEnum::Cuboid(Cuboid::new(
        (min_x, min_y, min_z),
        (max_x, max_y, max_z),
    )))))
}

#[no_mangle]
pub extern "C" fn shape_free(ptr: *mut ShapeWrapper) {
    if !ptr.is_null() {
        unsafe {
            let _ = Box::from_raw(ptr);
        }
    }
}

#[no_mangle]
pub extern "C" fn brush_solid(block_name: *const c_char) -> *mut BrushWrapper {
    if block_name.is_null() {
        return ptr::null_mut();
    }
    let name = unsafe { CStr::from_ptr(block_name).to_string_lossy().into_owned() };
    let block = BlockState::new(name);
    Box::into_raw(Box::new(BrushWrapper(BrushEnum::Solid(SolidBrush::new(
        block,
    )))))
}

#[no_mangle]
pub extern "C" fn brush_color(r: c_uchar, g: c_uchar, b: c_uchar) -> *mut BrushWrapper {
    Box::into_raw(Box::new(BrushWrapper(BrushEnum::Color(ColorBrush::new(
        r, g, b,
    )))))
}

#[no_mangle]
pub extern "C" fn brush_linear_gradient(
    x1: c_int,
    y1: c_int,
    z1: c_int,
    r1: c_uchar,
    g1: c_uchar,
    b1: c_uchar,
    x2: c_int,
    y2: c_int,
    z2: c_int,
    r2: c_uchar,
    g2: c_uchar,
    b2: c_uchar,
    space: c_int,
) -> *mut BrushWrapper {
    let interp = if space == 1 {
        InterpolationSpace::Oklab
    } else {
        InterpolationSpace::Rgb
    };
    let brush = LinearGradientBrush::new((x1, y1, z1), (r1, g1, b1), (x2, y2, z2), (r2, g2, b2))
        .with_space(interp);
    Box::into_raw(Box::new(BrushWrapper(BrushEnum::Linear(brush))))
}

#[no_mangle]
pub extern "C" fn brush_shaded(
    r: c_uchar,
    g: c_uchar,
    b: c_uchar,
    lx: c_float,
    ly: c_float,
    lz: c_float,
) -> *mut BrushWrapper {
    let brush = ShadedBrush::new((r, g, b), (lx as f64, ly as f64, lz as f64));
    Box::into_raw(Box::new(BrushWrapper(BrushEnum::Shaded(brush))))
}

#[no_mangle]
pub extern "C" fn brush_bilinear_gradient(
    ox: c_int,
    oy: c_int,
    oz: c_int,
    ux: c_int,
    uy: c_int,
    uz: c_int,
    vx: c_int,
    vy: c_int,
    vz: c_int,
    r00: c_uchar,
    g00: c_uchar,
    b00: c_uchar,
    r10: c_uchar,
    g10: c_uchar,
    b10: c_uchar,
    r01: c_uchar,
    g01: c_uchar,
    b01: c_uchar,
    r11: c_uchar,
    g11: c_uchar,
    b11: c_uchar,
    space: c_int,
) -> *mut BrushWrapper {
    let interp = if space == 1 {
        InterpolationSpace::Oklab
    } else {
        InterpolationSpace::Rgb
    };
    let brush = BilinearGradientBrush::new(
        (ox, oy, oz),
        (ux, uy, uz),
        (vx, vy, vz),
        (r00, g00, b00),
        (r10, g10, b10),
        (r01, g01, b01),
        (r11, g11, b11),
    )
    .with_space(interp);
    Box::into_raw(Box::new(BrushWrapper(BrushEnum::Bilinear(brush))))
}

#[no_mangle]
pub extern "C" fn brush_point_gradient(
    positions: *const c_int,
    colors: *const c_uchar,
    count: usize,
    falloff: c_float,
    space: c_int,
) -> *mut BrushWrapper {
    if positions.is_null() || colors.is_null() || count == 0 {
        return ptr::null_mut();
    }
    let pos_slice = unsafe { std::slice::from_raw_parts(positions, count * 3) };
    let col_slice = unsafe { std::slice::from_raw_parts(colors, count * 3) };
    let points: Vec<((i32, i32, i32), (u8, u8, u8))> = (0..count)
        .map(|i| {
            (
                (pos_slice[i * 3], pos_slice[i * 3 + 1], pos_slice[i * 3 + 2]),
                (col_slice[i * 3], col_slice[i * 3 + 1], col_slice[i * 3 + 2]),
            )
        })
        .collect();
    let interp = if space == 1 {
        InterpolationSpace::Oklab
    } else {
        InterpolationSpace::Rgb
    };
    let brush = PointGradientBrush::new(points)
        .with_space(interp)
        .with_falloff(falloff as f64);
    Box::into_raw(Box::new(BrushWrapper(BrushEnum::Point(brush))))
}

#[no_mangle]
pub extern "C" fn brush_free(ptr: *mut BrushWrapper) {
    if !ptr.is_null() {
        unsafe {
            let _ = Box::from_raw(ptr);
        }
    }
}

#[no_mangle]
pub extern "C" fn buildingtool_fill(
    schematic: *mut SchematicWrapper,
    shape: *const ShapeWrapper,
    brush: *const BrushWrapper,
) -> c_int {
    if schematic.is_null() || shape.is_null() || brush.is_null() {
        return -1;
    }
    let s = unsafe { &mut *(*schematic).0 };
    let sh = unsafe { &(*shape).0 };
    let br = unsafe { &(*brush).0 };
    let mut tool = BuildingTool::new(s);
    tool.fill_enum(sh, br);
    0
}

#[no_mangle]
pub extern "C" fn buildingtool_rstack(
    schematic: *mut SchematicWrapper,
    shape: *const ShapeWrapper,
    brush: *const BrushWrapper,
    count: usize,
    offset_x: c_int,
    offset_y: c_int,
    offset_z: c_int,
) -> c_int {
    if schematic.is_null() || shape.is_null() || brush.is_null() {
        return -1;
    }
    let s = unsafe { &mut *(*schematic).0 };
    let sh = unsafe { &(*shape).0 };
    let br = unsafe { &(*brush).0 };
    let mut tool = BuildingTool::new(s);
    tool.rstack(sh, br, count, (offset_x, offset_y, offset_z));
    0
}

// --- New Shape Constructors ---

#[no_mangle]
pub extern "C" fn shape_ellipsoid(
    cx: c_int,
    cy: c_int,
    cz: c_int,
    rx: c_float,
    ry: c_float,
    rz: c_float,
) -> *mut ShapeWrapper {
    Box::into_raw(Box::new(ShapeWrapper(ShapeEnum::Ellipsoid(
        Ellipsoid::new((cx, cy, cz), (rx as f64, ry as f64, rz as f64)),
    ))))
}

#[no_mangle]
pub extern "C" fn shape_cylinder(
    bx: c_float,
    by: c_float,
    bz: c_float,
    ax: c_float,
    ay: c_float,
    az: c_float,
    radius: c_float,
    height: c_float,
) -> *mut ShapeWrapper {
    Box::into_raw(Box::new(ShapeWrapper(ShapeEnum::Cylinder(Cylinder::new(
        (bx as f64, by as f64, bz as f64),
        (ax as f64, ay as f64, az as f64),
        radius as f64,
        height as f64,
    )))))
}

#[no_mangle]
pub extern "C" fn shape_cylinder_between(
    x1: c_float,
    y1: c_float,
    z1: c_float,
    x2: c_float,
    y2: c_float,
    z2: c_float,
    radius: c_float,
) -> *mut ShapeWrapper {
    Box::into_raw(Box::new(ShapeWrapper(ShapeEnum::Cylinder(
        Cylinder::between(
            (x1 as f64, y1 as f64, z1 as f64),
            (x2 as f64, y2 as f64, z2 as f64),
            radius as f64,
        ),
    ))))
}

#[no_mangle]
pub extern "C" fn shape_cone(
    ax: c_float,
    ay: c_float,
    az: c_float,
    dx: c_float,
    dy: c_float,
    dz: c_float,
    radius: c_float,
    height: c_float,
) -> *mut ShapeWrapper {
    Box::into_raw(Box::new(ShapeWrapper(ShapeEnum::Cone(Cone::new(
        (ax as f64, ay as f64, az as f64),
        (dx as f64, dy as f64, dz as f64),
        radius as f64,
        height as f64,
    )))))
}

#[no_mangle]
pub extern "C" fn shape_torus(
    cx: c_float,
    cy: c_float,
    cz: c_float,
    major_r: c_float,
    minor_r: c_float,
    ax: c_float,
    ay: c_float,
    az: c_float,
) -> *mut ShapeWrapper {
    Box::into_raw(Box::new(ShapeWrapper(ShapeEnum::Torus(Torus::new(
        (cx as f64, cy as f64, cz as f64),
        major_r as f64,
        minor_r as f64,
        (ax as f64, ay as f64, az as f64),
    )))))
}

#[no_mangle]
pub extern "C" fn shape_pyramid(
    bx: c_float,
    by: c_float,
    bz: c_float,
    half_w: c_float,
    half_d: c_float,
    height: c_float,
    ax: c_float,
    ay: c_float,
    az: c_float,
) -> *mut ShapeWrapper {
    Box::into_raw(Box::new(ShapeWrapper(ShapeEnum::Pyramid(Pyramid::new(
        (bx as f64, by as f64, bz as f64),
        (half_w as f64, half_d as f64),
        height as f64,
        (ax as f64, ay as f64, az as f64),
    )))))
}

#[no_mangle]
pub extern "C" fn shape_disk(
    cx: c_float,
    cy: c_float,
    cz: c_float,
    radius: c_float,
    nx: c_float,
    ny: c_float,
    nz: c_float,
    thickness: c_float,
) -> *mut ShapeWrapper {
    Box::into_raw(Box::new(ShapeWrapper(ShapeEnum::Disk(Disk::new(
        (cx as f64, cy as f64, cz as f64),
        radius as f64,
        (nx as f64, ny as f64, nz as f64),
        thickness as f64,
    )))))
}

#[no_mangle]
pub extern "C" fn shape_plane(
    ox: c_float,
    oy: c_float,
    oz: c_float,
    ux: c_float,
    uy: c_float,
    uz: c_float,
    vx: c_float,
    vy: c_float,
    vz: c_float,
    u_ext: c_float,
    v_ext: c_float,
    thickness: c_float,
) -> *mut ShapeWrapper {
    Box::into_raw(Box::new(ShapeWrapper(ShapeEnum::Plane(Plane::new(
        (ox as f64, oy as f64, oz as f64),
        (ux as f64, uy as f64, uz as f64),
        (vx as f64, vy as f64, vz as f64),
        u_ext as f64,
        v_ext as f64,
        thickness as f64,
    )))))
}

#[no_mangle]
pub extern "C" fn shape_triangle(
    ax: c_float,
    ay: c_float,
    az: c_float,
    bx: c_float,
    by: c_float,
    bz: c_float,
    cx: c_float,
    cy: c_float,
    cz: c_float,
    thickness: c_float,
) -> *mut ShapeWrapper {
    Box::into_raw(Box::new(ShapeWrapper(ShapeEnum::Triangle(Triangle::new(
        (ax as f64, ay as f64, az as f64),
        (bx as f64, by as f64, bz as f64),
        (cx as f64, cy as f64, cz as f64),
        thickness as f64,
    )))))
}

#[no_mangle]
pub extern "C" fn shape_line(
    x1: c_float,
    y1: c_float,
    z1: c_float,
    x2: c_float,
    y2: c_float,
    z2: c_float,
    thickness: c_float,
) -> *mut ShapeWrapper {
    Box::into_raw(Box::new(ShapeWrapper(ShapeEnum::Line(Line::new(
        (x1 as f64, y1 as f64, z1 as f64),
        (x2 as f64, y2 as f64, z2 as f64),
        thickness as f64,
    )))))
}

#[no_mangle]
pub extern "C" fn shape_bezier(
    control_points: *const c_float,
    num_points: usize,
    thickness: c_float,
    resolution: c_int,
) -> *mut ShapeWrapper {
    if control_points.is_null() || num_points == 0 {
        return ptr::null_mut();
    }
    let pts_slice = unsafe { std::slice::from_raw_parts(control_points, num_points * 3) };
    let points: Vec<(f64, f64, f64)> = (0..num_points)
        .map(|i| {
            (
                pts_slice[i * 3] as f64,
                pts_slice[i * 3 + 1] as f64,
                pts_slice[i * 3 + 2] as f64,
            )
        })
        .collect();
    Box::into_raw(Box::new(ShapeWrapper(ShapeEnum::BezierCurve(
        BezierCurve::new(points, thickness as f64, resolution as u32),
    ))))
}

#[no_mangle]
pub extern "C" fn shape_hollow(inner: *const ShapeWrapper, thickness: c_int) -> *mut ShapeWrapper {
    if inner.is_null() {
        return ptr::null_mut();
    }
    let inner_shape = unsafe { &(*inner).0 };
    Box::into_raw(Box::new(ShapeWrapper(ShapeEnum::Hollow(Hollow::new(
        inner_shape.clone(),
        thickness as u32,
    )))))
}

#[no_mangle]
pub extern "C" fn shape_union(a: *const ShapeWrapper, b: *const ShapeWrapper) -> *mut ShapeWrapper {
    if a.is_null() || b.is_null() {
        return ptr::null_mut();
    }
    let a_shape = unsafe { &(*a).0 };
    let b_shape = unsafe { &(*b).0 };
    Box::into_raw(Box::new(ShapeWrapper(ShapeEnum::Union(Union::new(
        a_shape.clone(),
        b_shape.clone(),
    )))))
}

#[no_mangle]
pub extern "C" fn shape_intersection(
    a: *const ShapeWrapper,
    b: *const ShapeWrapper,
) -> *mut ShapeWrapper {
    if a.is_null() || b.is_null() {
        return ptr::null_mut();
    }
    let a_shape = unsafe { &(*a).0 };
    let b_shape = unsafe { &(*b).0 };
    Box::into_raw(Box::new(ShapeWrapper(ShapeEnum::Intersection(
        Intersection::new(a_shape.clone(), b_shape.clone()),
    ))))
}

#[no_mangle]
pub extern "C" fn shape_difference(
    a: *const ShapeWrapper,
    b: *const ShapeWrapper,
) -> *mut ShapeWrapper {
    if a.is_null() || b.is_null() {
        return ptr::null_mut();
    }
    let a_shape = unsafe { &(*a).0 };
    let b_shape = unsafe { &(*b).0 };
    Box::into_raw(Box::new(ShapeWrapper(ShapeEnum::Difference(
        Difference::new(a_shape.clone(), b_shape.clone()),
    ))))
}

// --- New Brush Constructors ---

#[no_mangle]
pub extern "C" fn brush_curve_gradient(
    positions: *const c_float,
    colors: *const c_uchar,
    count: usize,
    space: c_int,
) -> *mut BrushWrapper {
    if positions.is_null() || colors.is_null() || count == 0 {
        return ptr::null_mut();
    }
    let pos_slice = unsafe { std::slice::from_raw_parts(positions, count) };
    let col_slice = unsafe { std::slice::from_raw_parts(colors, count * 3) };
    let stops: Vec<(f64, (u8, u8, u8))> = (0..count)
        .map(|i| {
            (
                pos_slice[i] as f64,
                (col_slice[i * 3], col_slice[i * 3 + 1], col_slice[i * 3 + 2]),
            )
        })
        .collect();
    let interp = if space == 1 {
        InterpolationSpace::Oklab
    } else {
        InterpolationSpace::Rgb
    };
    let brush = CurveGradientBrush::new(stops).with_space(interp);
    Box::into_raw(Box::new(BrushWrapper(BrushEnum::CurveGradient(brush))))
}
