//! Unified spatial field / SDF DAG, infix expression compiler, composable
//! materials, and brick-pruned voxelization pipeline for all Diplomat bindings.

#[diplomat::bridge]
pub mod ffi {
    use super::super::building::ffi::{Brush, InterpolationSpace, Palette, Shape};
    use super::super::schematic::ffi::Schematic;
    use super::super::shared::ffi::NucleationError;
    use diplomat_runtime::{DiplomatStr, DiplomatWrite};
    use std::fmt::Write;
    use std::sync::Arc;

    /// Axis selector for mirror operations.
    pub enum FieldAxis {
        X,
        Y,
        Z,
    }

    impl FieldAxis {
        fn to_core(self) -> crate::sdf::Axis {
            match self {
                FieldAxis::X => crate::sdf::Axis::X,
                FieldAxis::Y => crate::sdf::Axis::Y,
                FieldAxis::Z => crate::sdf::Axis::Z,
            }
        }
    }

    /// Cellular / Worley noise output mode.
    pub enum FieldCellMode {
        F1,
        F2,
        F2MinusF1,
        CellValue,
    }

    impl FieldCellMode {
        fn to_core(self) -> crate::sdf::CellMode {
            match self {
                FieldCellMode::F1 => crate::sdf::CellMode::F1,
                FieldCellMode::F2 => crate::sdf::CellMode::F2,
                FieldCellMode::F2MinusF1 => crate::sdf::CellMode::F2MinusF1,
                FieldCellMode::CellValue => crate::sdf::CellMode::Value,
            }
        }
    }

    /// Noise basis for procedural 2D/3D noise and FBM.
    pub enum FieldNoiseBasis {
        ValueNoise,
        Perlin,
    }

    impl FieldNoiseBasis {
        fn to_core(self) -> crate::sdf::NoiseBasis {
            match self {
                FieldNoiseBasis::ValueNoise => crate::sdf::NoiseBasis::Value,
                FieldNoiseBasis::Perlin => crate::sdf::NoiseBasis::Perlin,
            }
        }
    }

    /// Distance semantics of a [`Field`].
    pub enum FieldDistanceKind {
        Exact,
        LowerBound,
        Estimate,
        Implicit,
    }

    impl FieldDistanceKind {
        fn to_core(self) -> crate::sdf::DistanceKind {
            match self {
                FieldDistanceKind::Exact => crate::sdf::DistanceKind::Exact,
                FieldDistanceKind::LowerBound => crate::sdf::DistanceKind::LowerBound,
                FieldDistanceKind::Estimate => crate::sdf::DistanceKind::Estimate,
                FieldDistanceKind::Implicit => crate::sdf::DistanceKind::Implicit,
            }
        }

        fn from_core(k: crate::sdf::DistanceKind) -> Self {
            match k {
                crate::sdf::DistanceKind::Exact => FieldDistanceKind::Exact,
                crate::sdf::DistanceKind::LowerBound => FieldDistanceKind::LowerBound,
                crate::sdf::DistanceKind::Estimate => FieldDistanceKind::Estimate,
                crate::sdf::DistanceKind::Implicit => FieldDistanceKind::Implicit,
            }
        }
    }

    /// Built-in voxel channels readable by material gradients, ramps, and conditions.
    pub enum FieldChannel {
        WorldX,
        WorldY,
        WorldZ,
        Distance,
        DepthBelowSurface,
        Slope,
        NormalY,
        Curvature,
    }

    impl FieldChannel {
        fn to_core(self) -> crate::sdf::Channel {
            match self {
                FieldChannel::WorldX => crate::sdf::Channel::WorldX,
                FieldChannel::WorldY => crate::sdf::Channel::WorldY,
                FieldChannel::WorldZ => crate::sdf::Channel::WorldZ,
                FieldChannel::Distance => crate::sdf::Channel::Distance,
                FieldChannel::DepthBelowSurface => crate::sdf::Channel::DepthBelowSurface,
                FieldChannel::Slope => crate::sdf::Channel::Slope,
                FieldChannel::NormalY => crate::sdf::Channel::NormalY,
                FieldChannel::Curvature => crate::sdf::Channel::Curvature,
            }
        }
    }

    /// Face direction for surface scatter rules.
    pub enum ScatterFace {
        Top,
        Bottom,
        North,
        South,
        East,
        West,
    }

    impl ScatterFace {
        fn to_core(self) -> crate::sdf::ScatterFace {
            match self {
                ScatterFace::Top => crate::sdf::ScatterFace::Top,
                ScatterFace::Bottom => crate::sdf::ScatterFace::Bottom,
                ScatterFace::North => crate::sdf::ScatterFace::North,
                ScatterFace::South => crate::sdf::ScatterFace::South,
                ScatterFace::East => crate::sdf::ScatterFace::East,
                ScatterFace::West => crate::sdf::ScatterFace::West,
            }
        }
    }

    /// Conservative continuous bounds of a bounded [`Field`].
    #[diplomat::out]
    pub struct FieldBounds {
        pub min_x: f32,
        pub min_y: f32,
        pub min_z: f32,
        pub max_x: f32,
        pub max_y: f32,
        pub max_z: f32,
    }

    /// Analytically proven inclusive scalar output range `[min, max]`.
    #[diplomat::out]
    pub struct FieldRange {
        pub min: f32,
        pub max: f32,
    }

    /// Unit surface normal estimated from the field gradient.
    #[diplomat::out]
    pub struct FieldNormal {
        pub x: f32,
        pub y: f32,
        pub z: f32,
    }

    /// Distance plus propagated CSG material tag and smooth seam blend weight.
    #[diplomat::out]
    pub struct FieldSample {
        pub distance: f32,
        pub material_id: u16,
        pub blend_material_id: u16,
        pub blend_weight: f32,
    }

    /// Unified immutable spatial field / SDF DAG node.
    ///
    /// Primitives, CSG (hard, smooth, chamfer, stepped), transforms, domain
    /// warps, procedural noise, arbitrary math, material tags, and sandboxed
    /// infix expressions all compose freely and share storage across consumers.
    #[diplomat::opaque]
    pub struct Field(pub(crate) crate::sdf::Field);

    impl Field {
        // ── Coordinates & Constants ───────────────────────────────────────

        pub fn constant(value: f32) -> Result<Box<Field>, NucleationError> {
            crate::sdf::Field::constant(value)
                .map(|f| Box::new(Field(f)))
                .map_err(map_field_error)
        }

        pub fn coord_x() -> Box<Field> {
            Box::new(Field(crate::sdf::Field::x()))
        }

        pub fn coord_y() -> Box<Field> {
            Box::new(Field(crate::sdf::Field::y()))
        }

        pub fn coord_z() -> Box<Field> {
            Box::new(Field(crate::sdf::Field::z()))
        }

        // ── Primitives ────────────────────────────────────────────────────

        pub fn sphere(radius: f32) -> Result<Box<Field>, NucleationError> {
            wrap(crate::sdf::Field::sphere(radius))
        }

        pub fn box_shape(
            half_x: f32,
            half_y: f32,
            half_z: f32,
            rounding: f32,
        ) -> Result<Box<Field>, NucleationError> {
            wrap(crate::sdf::Field::box_shape([half_x, half_y, half_z], rounding))
        }

        pub fn ellipsoid(
            radius_x: f32,
            radius_y: f32,
            radius_z: f32,
        ) -> Result<Box<Field>, NucleationError> {
            wrap(crate::sdf::Field::ellipsoid([radius_x, radius_y, radius_z]))
        }

        pub fn torus(major_radius: f32, minor_radius: f32) -> Result<Box<Field>, NucleationError> {
            wrap(crate::sdf::Field::torus(major_radius, minor_radius))
        }

        pub fn capped_torus(
            major_radius: f32,
            minor_radius: f32,
            cap_angle_degrees: f32,
        ) -> Result<Box<Field>, NucleationError> {
            wrap(crate::sdf::Field::capped_torus(
                major_radius,
                minor_radius,
                cap_angle_degrees,
            ))
        }

        pub fn link(
            major_radius: f32,
            minor_radius: f32,
            half_length: f32,
        ) -> Result<Box<Field>, NucleationError> {
            wrap(crate::sdf::Field::link(
                major_radius,
                minor_radius,
                half_length,
            ))
        }

        #[allow(clippy::too_many_arguments)]
        pub fn capsule(
            ax: f32,
            ay: f32,
            az: f32,
            bx: f32,
            by: f32,
            bz: f32,
            radius: f32,
        ) -> Result<Box<Field>, NucleationError> {
            wrap(crate::sdf::Field::capsule([ax, ay, az], [bx, by, bz], radius))
        }

        #[allow(clippy::too_many_arguments)]
        pub fn round_cone(
            ax: f32,
            ay: f32,
            az: f32,
            bx: f32,
            by: f32,
            bz: f32,
            r1: f32,
            r2: f32,
        ) -> Result<Box<Field>, NucleationError> {
            wrap(crate::sdf::Field::round_cone(
                [ax, ay, az],
                [bx, by, bz],
                r1,
                r2,
            ))
        }

        pub fn solid_angle(radius: f32, angle_degrees: f32) -> Result<Box<Field>, NucleationError> {
            wrap(crate::sdf::Field::solid_angle(radius, angle_degrees))
        }

        pub fn cut_sphere(radius: f32, height: f32) -> Result<Box<Field>, NucleationError> {
            wrap(crate::sdf::Field::cut_sphere(radius, height))
        }

        pub fn cut_hollow_sphere(
            radius: f32,
            height: f32,
            thickness: f32,
        ) -> Result<Box<Field>, NucleationError> {
            wrap(crate::sdf::Field::cut_hollow_sphere(
                radius, height, thickness,
            ))
        }

        pub fn capped_cylinder(
            radius: f32,
            half_height: f32,
        ) -> Result<Box<Field>, NucleationError> {
            wrap(crate::sdf::Field::capped_cylinder(radius, half_height))
        }

        pub fn infinite_cylinder(radius: f32) -> Result<Box<Field>, NucleationError> {
            wrap(crate::sdf::Field::infinite_cylinder(radius))
        }

        pub fn capped_cone(
            half_height: f32,
            bottom_radius: f32,
            top_radius: f32,
        ) -> Result<Box<Field>, NucleationError> {
            wrap(crate::sdf::Field::capped_cone(
                half_height,
                bottom_radius,
                top_radius,
            ))
        }

        pub fn plane(
            normal_x: f32,
            normal_y: f32,
            normal_z: f32,
            offset: f32,
        ) -> Result<Box<Field>, NucleationError> {
            wrap(crate::sdf::Field::plane(
                [normal_x, normal_y, normal_z],
                offset,
            ))
        }

        pub fn octahedron(size: f32) -> Result<Box<Field>, NucleationError> {
            wrap(crate::sdf::Field::octahedron(size))
        }

        pub fn hex_prism(radius: f32, half_height: f32) -> Result<Box<Field>, NucleationError> {
            wrap(crate::sdf::Field::hex_prism(radius, half_height))
        }

        pub fn super_prism(
            half_x: f32,
            half_y: f32,
            half_z: f32,
            exponent: f32,
        ) -> Result<Box<Field>, NucleationError> {
            wrap(crate::sdf::Field::super_prism(
                [half_x, half_y, half_z],
                exponent,
            ))
        }

        pub fn box_frame(
            half_x: f32,
            half_y: f32,
            half_z: f32,
            thickness: f32,
        ) -> Result<Box<Field>, NucleationError> {
            wrap(crate::sdf::Field::box_frame(
                [half_x, half_y, half_z],
                thickness,
            ))
        }

        pub fn infinite_cone(angle_degrees: f32) -> Result<Box<Field>, NucleationError> {
            wrap(crate::sdf::Field::infinite_cone(angle_degrees))
        }

        pub fn square_pyramid(half_base: f32, height: f32) -> Result<Box<Field>, NucleationError> {
            wrap(crate::sdf::Field::square_pyramid(half_base, height))
        }

        /// Heightmap terrain surface: solid where `y <= base_y + height(x, 0, z)`.
        pub fn heightmap(base_y: f32, height: &Field) -> Result<Box<Field>, NucleationError> {
            wrap(crate::sdf::Field::heightmap(base_y, height.0.clone()))
        }

        // ── Procedural Noise & Cellular ───────────────────────────────────

        pub fn value_noise2(frequency: f32, seed: i32) -> Result<Box<Field>, NucleationError> {
            wrap(crate::sdf::Field::value_noise2(frequency, seed))
        }

        pub fn value_noise3(frequency: f32, seed: i32) -> Result<Box<Field>, NucleationError> {
            wrap(crate::sdf::Field::value_noise3(frequency, seed))
        }

        pub fn perlin2(frequency: f32, seed: i32) -> Result<Box<Field>, NucleationError> {
            wrap(crate::sdf::Field::perlin2(frequency, seed))
        }

        pub fn perlin3(frequency: f32, seed: i32) -> Result<Box<Field>, NucleationError> {
            wrap(crate::sdf::Field::perlin3(frequency, seed))
        }

        #[allow(clippy::too_many_arguments)]
        pub fn fbm2(
            basis: FieldNoiseBasis,
            frequency: f32,
            seed: i32,
            octaves: u32,
            lacunarity: f32,
            gain: f32,
            ridged: bool,
        ) -> Result<Box<Field>, NucleationError> {
            wrap(crate::sdf::Field::fbm2(
                basis.to_core(),
                frequency,
                seed,
                octaves,
                lacunarity,
                gain,
                ridged,
            ))
        }

        #[allow(clippy::too_many_arguments)]
        pub fn fbm3(
            basis: FieldNoiseBasis,
            frequency: f32,
            seed: i32,
            octaves: u32,
            lacunarity: f32,
            gain: f32,
            ridged: bool,
        ) -> Result<Box<Field>, NucleationError> {
            wrap(crate::sdf::Field::fbm3(
                basis.to_core(),
                frequency,
                seed,
                octaves,
                lacunarity,
                gain,
                ridged,
            ))
        }

        pub fn value_fbm3(
            frequency: f32,
            seed: i32,
            octaves: u32,
        ) -> Result<Box<Field>, NucleationError> {
            wrap(crate::sdf::Field::value_fbm3(frequency, seed, octaves))
        }

        pub fn perlin_fbm3(
            frequency: f32,
            seed: i32,
            octaves: u32,
        ) -> Result<Box<Field>, NucleationError> {
            wrap(crate::sdf::Field::perlin_fbm3(frequency, seed, octaves))
        }

        pub fn ridged_fbm3(
            frequency: f32,
            seed: i32,
            octaves: u32,
        ) -> Result<Box<Field>, NucleationError> {
            wrap(crate::sdf::Field::ridged_fbm3(frequency, seed, octaves))
        }

        pub fn perlin_fbm2(
            frequency: f32,
            seed: i32,
            octaves: u32,
        ) -> Result<Box<Field>, NucleationError> {
            wrap(crate::sdf::Field::perlin_fbm2(frequency, seed, octaves))
        }

        pub fn cells(
            frequency: f32,
            seed: i32,
            jitter: f32,
            mode: FieldCellMode,
            threshold: f32,
        ) -> Result<Box<Field>, NucleationError> {
            wrap(crate::sdf::Field::cells(
                frequency,
                seed,
                jitter,
                mode.to_core(),
                threshold,
            ))
        }

        // ── Scalar Math & Combinators ─────────────────────────────────────

        pub fn neg(&self) -> Box<Field> {
            Box::new(Field(self.0.neg()))
        }

        pub fn abs(&self) -> Box<Field> {
            Box::new(Field(self.0.abs()))
        }

        pub fn sqrt(&self) -> Box<Field> {
            Box::new(Field(self.0.sqrt()))
        }

        pub fn sin(&self) -> Box<Field> {
            Box::new(Field(self.0.sin()))
        }

        pub fn cos(&self) -> Box<Field> {
            Box::new(Field(self.0.cos()))
        }

        pub fn saturate(&self) -> Box<Field> {
            Box::new(Field(self.0.saturate()))
        }

        pub fn add(&self, other: &Field) -> Box<Field> {
            Box::new(Field(self.0.add(&other.0)))
        }

        pub fn sub(&self, other: &Field) -> Box<Field> {
            Box::new(Field(self.0.sub(&other.0)))
        }

        pub fn mul(&self, other: &Field) -> Box<Field> {
            Box::new(Field(self.0.mul(&other.0)))
        }

        pub fn div(&self, other: &Field) -> Box<Field> {
            Box::new(Field(self.0.div(&other.0)))
        }

        pub fn min_with(&self, other: &Field) -> Box<Field> {
            Box::new(Field(self.0.min_with(&other.0)))
        }

        pub fn max_with(&self, other: &Field) -> Box<Field> {
            Box::new(Field(self.0.max_with(&other.0)))
        }

        pub fn add_const(&self, value: f32) -> Result<Box<Field>, NucleationError> {
            wrap(self.0.add_const(value))
        }

        pub fn mul_const(&self, value: f32) -> Result<Box<Field>, NucleationError> {
            wrap(self.0.mul_const(value))
        }

        pub fn clamp(&self, lo: f32, hi: f32) -> Result<Box<Field>, NucleationError> {
            wrap(self.0.clamp(lo, hi))
        }

        pub fn smoothstep(&self, edge0: f32, edge1: f32) -> Result<Box<Field>, NucleationError> {
            wrap(self.0.smoothstep(edge0, edge1))
        }

        pub fn step(&self, edge: f32) -> Result<Box<Field>, NucleationError> {
            wrap(self.0.step(edge))
        }

        pub fn remap(
            &self,
            in_lo: f32,
            in_hi: f32,
            out_lo: f32,
            out_hi: f32,
            clamp: bool,
        ) -> Result<Box<Field>, NucleationError> {
            wrap(self.0.remap(in_lo, in_hi, out_lo, out_hi, clamp))
        }

        pub fn lerp_with(&self, other: &Field, t: &Field) -> Box<Field> {
            Box::new(Field(self.0.lerp(&other.0, &t.0)))
        }

        pub fn select(cond: &Field, then_field: &Field, else_field: &Field) -> Box<Field> {
            Box::new(Field(crate::sdf::Field::select(
                &cond.0,
                &then_field.0,
                &else_field.0,
            )))
        }

        // ── CSG & Surface Modifiers ───────────────────────────────────────

        pub fn union_with(&self, other: &Field) -> Box<Field> {
            Box::new(Field(self.0.union(&other.0)))
        }

        pub fn intersection_with(&self, other: &Field) -> Box<Field> {
            Box::new(Field(self.0.intersect(&other.0)))
        }

        pub fn subtract(&self, other: &Field) -> Box<Field> {
            Box::new(Field(self.0.subtract(&other.0)))
        }

        pub fn xor_with(&self, other: &Field) -> Box<Field> {
            Box::new(Field(self.0.xor(&other.0)))
        }

        pub fn smooth_union(
            &self,
            other: &Field,
            radius: f32,
        ) -> Result<Box<Field>, NucleationError> {
            wrap(self.0.smooth_union(&other.0, radius))
        }

        pub fn smooth_subtract(
            &self,
            other: &Field,
            radius: f32,
        ) -> Result<Box<Field>, NucleationError> {
            wrap(self.0.smooth_subtract(&other.0, radius))
        }

        pub fn smooth_intersection(
            &self,
            other: &Field,
            radius: f32,
        ) -> Result<Box<Field>, NucleationError> {
            wrap(self.0.smooth_intersect(&other.0, radius))
        }

        pub fn chamfer_union(
            &self,
            other: &Field,
            radius: f32,
        ) -> Result<Box<Field>, NucleationError> {
            wrap(self.0.chamfer_union(&other.0, radius))
        }

        pub fn chamfer_subtract(
            &self,
            other: &Field,
            radius: f32,
        ) -> Result<Box<Field>, NucleationError> {
            wrap(self.0.chamfer_subtract(&other.0, radius))
        }

        pub fn chamfer_intersection(
            &self,
            other: &Field,
            radius: f32,
        ) -> Result<Box<Field>, NucleationError> {
            wrap(self.0.chamfer_intersect(&other.0, radius))
        }

        pub fn steps_union(
            &self,
            other: &Field,
            radius: f32,
            steps: f32,
        ) -> Result<Box<Field>, NucleationError> {
            wrap(self.0.steps_union(&other.0, radius, steps))
        }

        pub fn rounded(&self, radius: f32) -> Result<Box<Field>, NucleationError> {
            wrap(self.0.round(radius))
        }

        pub fn shell(&self, thickness: f32) -> Result<Box<Field>, NucleationError> {
            wrap(self.0.shell(thickness))
        }

        pub fn elongate(
            &self,
            half_x: f32,
            half_y: f32,
            half_z: f32,
        ) -> Result<Box<Field>, NucleationError> {
            wrap(self.0.elongate([half_x, half_y, half_z]))
        }

        pub fn displace(&self, offset: &Field, amplitude: f32) -> Result<Box<Field>, NucleationError> {
            wrap(self.0.displace(&offset.0, amplitude))
        }

        pub fn displace_noise(
            &self,
            amplitude: f32,
            frequency: f32,
            seed: i32,
            octaves: u32,
        ) -> Result<Box<Field>, NucleationError> {
            wrap(self.0.displace_noise(amplitude, frequency, seed, octaves))
        }

        // ── Domain Transforms & Space Warps ───────────────────────────────

        pub fn translate(&self, x: f32, y: f32, z: f32) -> Result<Box<Field>, NucleationError> {
            wrap(self.0.translate([x, y, z]))
        }

        pub fn rotate(
            &self,
            x_degrees: f32,
            y_degrees: f32,
            z_degrees: f32,
        ) -> Result<Box<Field>, NucleationError> {
            wrap(self.0.rotate([x_degrees, y_degrees, z_degrees]))
        }

        pub fn scale(&self, factor: f32) -> Result<Box<Field>, NucleationError> {
            wrap(self.0.scale(factor))
        }

        pub fn scale_xyz(&self, sx: f32, sy: f32, sz: f32) -> Result<Box<Field>, NucleationError> {
            wrap(self.0.scale_xyz([sx, sy, sz]))
        }

        pub fn mirror(&self, axis: FieldAxis) -> Box<Field> {
            Box::new(Field(self.0.mirror(axis.to_core())))
        }

        pub fn mirror_plane(
            &self,
            nx: f32,
            ny: f32,
            nz: f32,
            offset: f32,
        ) -> Result<Box<Field>, NucleationError> {
            wrap(self.0.mirror_plane([nx, ny, nz], offset))
        }

        pub fn repeat_infinite(
            &self,
            spacing_x: f32,
            spacing_y: f32,
            spacing_z: f32,
        ) -> Result<Box<Field>, NucleationError> {
            wrap(self.0.repeat_infinite([spacing_x, spacing_y, spacing_z]))
        }

        #[allow(clippy::too_many_arguments)]
        pub fn repeat_counted(
            &self,
            spacing_x: f32,
            spacing_y: f32,
            spacing_z: f32,
            count_x: u32,
            count_y: u32,
            count_z: u32,
        ) -> Result<Box<Field>, NucleationError> {
            wrap(self.0.repeat_counted(
                [spacing_x, spacing_y, spacing_z],
                [count_x, count_y, count_z],
            ))
        }

        pub fn repeat_polar(&self, count: u32) -> Result<Box<Field>, NucleationError> {
            wrap(self.0.repeat_polar(count))
        }

        pub fn repeat_points(&self, offsets: &[f32]) -> Result<Box<Field>, NucleationError> {
            if offsets.is_empty() || offsets.len() % 3 != 0 {
                return Err(NucleationError::InvalidArgument);
            }
            let pts: Vec<[f32; 3]> = offsets
                .chunks_exact(3)
                .map(|c| [c[0], c[1], c[2]])
                .collect();
            wrap(self.0.repeat_points(pts))
        }

        pub fn twist(&self, amount: f32) -> Result<Box<Field>, NucleationError> {
            wrap(self.0.twist(amount))
        }

        pub fn bend(&self, amount: f32) -> Result<Box<Field>, NucleationError> {
            wrap(self.0.bend(amount))
        }

        pub fn warp(
            &self,
            warp_x: &Field,
            warp_y: &Field,
            warp_z: &Field,
            amplitude: f32,
        ) -> Result<Box<Field>, NucleationError> {
            wrap(self.0.warp(&warp_x.0, &warp_y.0, &warp_z.0, amplitude))
        }

        pub fn warp_noise(
            &self,
            amplitude: f32,
            frequency: f32,
            seed: i32,
            octaves: u32,
        ) -> Result<Box<Field>, NucleationError> {
            wrap(self.0.warp_noise(amplitude, frequency, seed, octaves))
        }

        // ── Tagging, Bounds & Inline Expressions ──────────────────────────

        pub fn tag(&self, material_id: u16) -> Result<Box<Field>, NucleationError> {
            wrap(self.0.tag(material_id))
        }

        #[allow(clippy::too_many_arguments)]
        pub fn bounded(
            &self,
            min_x: f32,
            min_y: f32,
            min_z: f32,
            max_x: f32,
            max_y: f32,
            max_z: f32,
        ) -> Result<Box<Field>, NucleationError> {
            let aabb = crate::sdf::Aabb::new([min_x, min_y, min_z], [max_x, max_y, max_z])
                .map_err(map_field_error)?;
            wrap(self.0.bounded(aabb))
        }

        /// Parse a sandboxed infix expression (supports `x, y, z, p`, math,
        /// noise, primitives, `let`, `if/else`, and bounded `for` loops).
        pub fn expr(source: &DiplomatStr) -> Result<Box<Field>, NucleationError> {
            let src = std::str::from_utf8(source).map_err(|_| NucleationError::InvalidArgument)?;
            wrap(crate::sdf::Field::expr(src))
        }

        // ── Evaluation, Inspection, Conversion & Serialization ────────────

        pub fn eval_at(&self, x: f32, y: f32, z: f32) -> f32 {
            self.0.eval(x, y, z)
        }

        pub fn sample_tagged(&self, x: f32, y: f32, z: f32) -> FieldSample {
            let s = self.0.compile().eval_tagged(x, y, z);
            FieldSample {
                distance: s.distance,
                material_id: s.material_id,
                blend_material_id: s.blend_material_id,
                blend_weight: s.blend_weight,
            }
        }

        pub fn normal(
            &self,
            x: f32,
            y: f32,
            z: f32,
            epsilon: f32,
        ) -> Result<FieldNormal, NucleationError> {
            let [nx, ny, nz] = self
                .0
                .normal(x, y, z, epsilon)
                .ok_or(NucleationError::InvalidArgument)?;
            Ok(FieldNormal {
                x: nx,
                y: ny,
                z: nz,
            })
        }

        pub fn bounds(&self) -> Result<FieldBounds, NucleationError> {
            self.0
                .bounds()
                .map(|b| FieldBounds {
                    min_x: b.min[0],
                    min_y: b.min[1],
                    min_z: b.min[2],
                    max_x: b.max[0],
                    max_y: b.max[1],
                    max_z: b.max[2],
                })
                .ok_or(NucleationError::NotFound)
        }

        pub fn output_range(&self) -> Result<FieldRange, NucleationError> {
            self.0
                .output_range()
                .map(|r| FieldRange { min: r[0], max: r[1] })
                .ok_or(NucleationError::NotFound)
        }

        pub fn distance_kind(&self) -> FieldDistanceKind {
            FieldDistanceKind::from_core(self.0.distance_kind())
        }

        pub fn to_shape(&self) -> Result<Box<Shape>, NucleationError> {
            crate::building::SdfShape::new(self.0.clone())
                .map(|s| Box::new(Shape(crate::building::ShapeEnum::Sdf(s))))
                .ok_or(NucleationError::InvalidArgument)
        }

        #[allow(clippy::too_many_arguments)]
        pub fn to_shape_bounded(
            &self,
            min_x: i32,
            min_y: i32,
            min_z: i32,
            max_x: i32,
            max_y: i32,
            max_z: i32,
        ) -> Result<Box<Shape>, NucleationError> {
            crate::building::SdfShape::with_bounds(
                self.0.clone(),
                (min_x, min_y, min_z),
                (max_x, max_y, max_z),
            )
            .map(|s| Box::new(Shape(crate::building::ShapeEnum::Sdf(s))))
            .ok_or(NucleationError::InvalidArgument)
        }

        pub fn from_json(json: &DiplomatStr) -> Result<Box<Field>, NucleationError> {
            let s = std::str::from_utf8(json).map_err(|_| NucleationError::InvalidArgument)?;
            wrap(crate::sdf::Field::from_json(s))
        }

        pub fn to_json(&self, write: &mut DiplomatWrite) -> Result<(), NucleationError> {
            let json = self.0.to_json().map_err(|_| NucleationError::Serialize)?;
            let _ = write!(write, "{json}");
            Ok(())
        }
    }

    /// Builder for compiling infix expressions that reference named external [`Field`]s.
    #[diplomat::opaque_mut]
    pub struct FieldExprBuilder {
        bindings: Vec<(String, crate::sdf::Field)>,
        bounds: Option<crate::sdf::Aabb>,
        distance_kind: crate::sdf::DistanceKind,
    }

    impl FieldExprBuilder {
        pub fn create() -> Box<FieldExprBuilder> {
            Box::new(FieldExprBuilder {
                bindings: Vec::new(),
                bounds: None,
                distance_kind: crate::sdf::DistanceKind::Estimate,
            })
        }

        pub fn bind(&mut self, name: &DiplomatStr, field: &Field) -> Result<(), NucleationError> {
            let n = std::str::from_utf8(name).map_err(|_| NucleationError::InvalidArgument)?;
            if n.is_empty() {
                return Err(NucleationError::InvalidArgument);
            }
            self.bindings.push((n.to_string(), field.0.clone()));
            Ok(())
        }

        #[allow(clippy::too_many_arguments)]
        pub fn set_bounds(
            &mut self,
            min_x: f32,
            min_y: f32,
            min_z: f32,
            max_x: f32,
            max_y: f32,
            max_z: f32,
        ) -> Result<(), NucleationError> {
            let aabb = crate::sdf::Aabb::new([min_x, min_y, min_z], [max_x, max_y, max_z])
                .map_err(map_field_error)?;
            self.bounds = Some(aabb);
            Ok(())
        }

        pub fn set_distance_kind(&mut self, kind: FieldDistanceKind) {
            self.distance_kind = kind.to_core();
        }

        pub fn compile(&self, source: &DiplomatStr) -> Result<Box<Field>, NucleationError> {
            let src = std::str::from_utf8(source).map_err(|_| NucleationError::InvalidArgument)?;
            let refs: Vec<(&str, crate::sdf::Field)> = self
                .bindings
                .iter()
                .map(|(n, f)| (n.as_str(), f.clone()))
                .collect();
            wrap(crate::sdf::Field::expr_with(
                src,
                &refs,
                self.bounds,
                self.distance_kind,
            ))
        }
    }

    /// Composable voxel predicate for [`Material`] layers and [`ScatterRule`]s.
    #[diplomat::opaque]
    pub struct MaterialCondition(pub(crate) crate::sdf::Condition);

    impl MaterialCondition {
        pub fn always() -> Box<MaterialCondition> {
            Box::new(MaterialCondition(crate::sdf::Condition::Always))
        }

        pub fn and_with(&self, other: &MaterialCondition) -> Box<MaterialCondition> {
            Box::new(MaterialCondition(self.0.clone().and(other.0.clone())))
        }

        pub fn or_with(&self, other: &MaterialCondition) -> Box<MaterialCondition> {
            Box::new(MaterialCondition(self.0.clone().or(other.0.clone())))
        }

        pub fn negated(&self) -> Box<MaterialCondition> {
            Box::new(MaterialCondition(self.0.clone().not()))
        }

        pub fn tag(material_id: u16) -> Box<MaterialCondition> {
            Box::new(MaterialCondition(crate::sdf::Condition::tag(material_id)))
        }

        pub fn channel_range(
            channel: FieldChannel,
            has_min: bool,
            min: f32,
            has_max: bool,
            max: f32,
        ) -> Result<Box<MaterialCondition>, NucleationError> {
            let lo = opt_finite(has_min, min)?;
            let hi = opt_finite(has_max, max)?;
            Ok(Box::new(MaterialCondition(
                crate::sdf::Condition::channel_range(channel.to_core(), lo, hi),
            )))
        }

        pub fn field_range(
            field: &Field,
            has_min: bool,
            min: f32,
            has_max: bool,
            max: f32,
        ) -> Result<Box<MaterialCondition>, NucleationError> {
            let lo = opt_finite(has_min, min)?;
            let hi = opt_finite(has_max, max)?;
            Ok(Box::new(MaterialCondition(
                crate::sdf::Condition::field_range(field.0.clone(), lo, hi),
            )))
        }

        pub fn y_range(
            has_min: bool,
            min: f32,
            has_max: bool,
            max: f32,
        ) -> Result<Box<MaterialCondition>, NucleationError> {
            let lo = opt_finite(has_min, min)?;
            let hi = opt_finite(has_max, max)?;
            Ok(Box::new(MaterialCondition(crate::sdf::Condition::y_range(
                lo, hi,
            ))))
        }

        pub fn depth_below_surface(
            has_min: bool,
            min: i32,
            has_max: bool,
            max: i32,
        ) -> Box<MaterialCondition> {
            Box::new(MaterialCondition(
                crate::sdf::Condition::depth_below_surface(
                    has_min.then_some(min),
                    has_max.then_some(max),
                ),
            ))
        }

        pub fn slope_range(
            has_min: bool,
            min: f32,
            has_max: bool,
            max: f32,
        ) -> Result<Box<MaterialCondition>, NucleationError> {
            let lo = opt_finite(has_min, min)?;
            let hi = opt_finite(has_max, max)?;
            Ok(Box::new(MaterialCondition(
                crate::sdf::Condition::slope_range(lo, hi),
            )))
        }

        pub fn curvature_range(
            has_min: bool,
            min: f32,
            has_max: bool,
            max: f32,
        ) -> Result<Box<MaterialCondition>, NucleationError> {
            let lo = opt_finite(has_min, min)?;
            let hi = opt_finite(has_max, max)?;
            Ok(Box::new(MaterialCondition(
                crate::sdf::Condition::curvature_range(lo, hi),
            )))
        }

        #[allow(clippy::too_many_arguments)]
        pub fn normal_dot(
            dx: f32,
            dy: f32,
            dz: f32,
            has_min: bool,
            min: f32,
            has_max: bool,
            max: f32,
        ) -> Result<Box<MaterialCondition>, NucleationError> {
            let lo = opt_finite(has_min, min)?;
            let hi = opt_finite(has_max, max)?;
            crate::sdf::Condition::normal_dot([dx, dy, dz], lo, hi)
                .map(|c| Box::new(MaterialCondition(c)))
                .map_err(map_field_error)
        }

        pub fn exposed_face(mask: u8, require_all: bool) -> Box<MaterialCondition> {
            Box::new(MaterialCondition(crate::sdf::Condition::exposed_face(
                mask,
                require_all,
            )))
        }

        pub fn chance(probability: f32, seed: i32) -> Result<Box<MaterialCondition>, NucleationError> {
            crate::sdf::Condition::chance(probability, seed)
                .map(|c| Box::new(MaterialCondition(c)))
                .map_err(map_field_error)
        }
    }

    /// Composable block shader for [`VoxelPipeline`].
    #[diplomat::opaque]
    pub struct Material(pub(crate) crate::sdf::Material);

    impl Material {
        pub fn solid(block: &DiplomatStr) -> Result<Box<Material>, NucleationError> {
            let s = std::str::from_utf8(block).map_err(|_| NucleationError::InvalidArgument)?;
            Ok(Box::new(Material(crate::sdf::Material::solid(s))))
        }

        pub fn skip() -> Box<Material> {
            Box::new(Material(crate::sdf::Material::skip()))
        }

        pub fn color(r: u8, g: u8, b: u8, palette: &Palette) -> Box<Material> {
            Box::new(Material(crate::sdf::Material::color(
                [r, g, b],
                Arc::clone(&palette.0),
            )))
        }

        pub fn from_brush(brush: &Brush) -> Box<Material> {
            Box::new(Material(crate::sdf::Material::from_brush(brush.0.clone())))
        }

        #[allow(clippy::too_many_arguments)]
        pub fn channel_gradient(
            channel: FieldChannel,
            stops: &[f32],
            colors: &[u8],
            lo: f32,
            hi: f32,
            palette: &Palette,
            space: InterpolationSpace,
        ) -> Result<Box<Material>, NucleationError> {
            let gstops = parse_stops(stops, colors)?;
            crate::sdf::Material::channel_gradient(
                channel.to_core(),
                gstops,
                lo,
                hi,
                Arc::clone(&palette.0),
                interp_space(space),
            )
            .map(|m| Box::new(Material(m)))
            .map_err(map_field_error)
        }

        #[allow(clippy::too_many_arguments)]
        pub fn field_gradient(
            field: &Field,
            stops: &[f32],
            colors: &[u8],
            lo: f32,
            hi: f32,
            palette: &Palette,
            space: InterpolationSpace,
        ) -> Result<Box<Material>, NucleationError> {
            let gstops = parse_stops(stops, colors)?;
            crate::sdf::Material::channel_gradient(
                crate::sdf::Channel::Field(field.0.clone()),
                gstops,
                lo,
                hi,
                Arc::clone(&palette.0),
                interp_space(space),
            )
            .map(|m| Box::new(Material(m)))
            .map_err(map_field_error)
        }

        pub fn channel_ramp(
            channel: FieldChannel,
            lo: f32,
            hi: f32,
            blocks_json: &DiplomatStr,
            dither: bool,
        ) -> Result<Box<Material>, NucleationError> {
            let blocks = parse_blocks_json(blocks_json)?;
            crate::sdf::Material::channel_ramp(channel.to_core(), lo, hi, blocks, dither)
                .map(|m| Box::new(Material(m)))
                .map_err(map_field_error)
        }

        pub fn field_ramp(
            field: &Field,
            lo: f32,
            hi: f32,
            blocks_json: &DiplomatStr,
            dither: bool,
        ) -> Result<Box<Material>, NucleationError> {
            let blocks = parse_blocks_json(blocks_json)?;
            crate::sdf::Material::channel_ramp(
                crate::sdf::Channel::Field(field.0.clone()),
                lo,
                hi,
                blocks,
                dither,
            )
            .map(|m| Box::new(Material(m)))
            .map_err(map_field_error)
        }

        pub fn weighted(
            blocks_json: &DiplomatStr,
            weights: &[f32],
            seed: i32,
        ) -> Result<Box<Material>, NucleationError> {
            let blocks = parse_blocks_json(blocks_json)?;
            if blocks.len() != weights.len() {
                return Err(NucleationError::InvalidArgument);
            }
            let entries = blocks.into_iter().zip(weights.iter().copied()).collect();
            crate::sdf::Material::weighted(entries, seed)
                .map(|m| Box::new(Material(m)))
                .map_err(map_field_error)
        }

        #[allow(clippy::too_many_arguments)]
        pub fn shaded(
            r: u8,
            g: u8,
            b: u8,
            lx: f32,
            ly: f32,
            lz: f32,
            ambient: f32,
            palette: &Palette,
        ) -> Result<Box<Material>, NucleationError> {
            crate::sdf::Material::shaded(
                [r, g, b],
                [lx, ly, lz],
                ambient,
                Arc::clone(&palette.0),
            )
            .map(|m| Box::new(Material(m)))
            .map_err(map_field_error)
        }

        #[allow(clippy::too_many_arguments)]
        pub fn spotlight(
            px: f32,
            py: f32,
            pz: f32,
            dx: f32,
            dy: f32,
            dz: f32,
            cone_angle_deg: f32,
            r: u8,
            g: u8,
            b: u8,
            ambient: f32,
            palette: &Palette,
        ) -> Result<Box<Material>, NucleationError> {
            crate::sdf::Material::spotlight(
                [px, py, pz],
                [dx, dy, dz],
                cone_angle_deg,
                [r, g, b],
                ambient,
                Arc::clone(&palette.0),
            )
            .map(|m| Box::new(Material(m)))
            .map_err(map_field_error)
        }
    }

    /// Builder for layered conditional materials and CSG tag-dispatched materials.
    #[diplomat::opaque_mut]
    pub struct MaterialBuilder {
        is_tag_matcher: bool,
        blend_seams: bool,
        seed: i32,
        layers: Vec<crate::sdf::MaterialLayer>,
        cases: Vec<(u16, crate::sdf::Material)>,
        default: Option<crate::sdf::Material>,
    }

    impl MaterialBuilder {
        pub fn create_layered() -> Box<MaterialBuilder> {
            Box::new(MaterialBuilder {
                is_tag_matcher: false,
                blend_seams: false,
                seed: 0,
                layers: Vec::new(),
                cases: Vec::new(),
                default: None,
            })
        }

        pub fn create_tag_matcher(blend_seams: bool, seed: i32) -> Box<MaterialBuilder> {
            Box::new(MaterialBuilder {
                is_tag_matcher: true,
                blend_seams,
                seed,
                layers: Vec::new(),
                cases: Vec::new(),
                default: None,
            })
        }

        pub fn add_layer(
            &mut self,
            condition: &MaterialCondition,
            material: &Material,
        ) -> Result<(), NucleationError> {
            if self.is_tag_matcher {
                return Err(NucleationError::InvalidArgument);
            }
            self.layers.push(crate::sdf::MaterialLayer {
                condition: condition.0.clone(),
                material: material.0.clone(),
            });
            Ok(())
        }

        pub fn add_tag_case(
            &mut self,
            material_id: u16,
            material: &Material,
        ) -> Result<(), NucleationError> {
            if !self.is_tag_matcher {
                return Err(NucleationError::InvalidArgument);
            }
            self.cases.push((material_id, material.0.clone()));
            Ok(())
        }

        pub fn set_default(&mut self, material: &Material) {
            self.default = Some(material.0.clone());
        }

        pub fn build(&self) -> Box<Material> {
            if self.is_tag_matcher {
                let default = self
                    .default
                    .clone()
                    .unwrap_or_else(crate::sdf::Material::skip);
                Box::new(Material(crate::sdf::Material::match_tag(
                    self.cases.clone(),
                    default,
                    self.blend_seams,
                    self.seed,
                )))
            } else {
                Box::new(Material(crate::sdf::Material::layered(
                    self.layers.clone(),
                    self.default.clone(),
                )))
            }
        }
    }

    /// Surface scatter rule attached to a [`VoxelPipeline`].
    #[diplomat::opaque_mut]
    pub struct ScatterRule(pub(crate) crate::sdf::ScatterRule);

    impl ScatterRule {
        pub fn create(
            face: ScatterFace,
            density: f32,
            blocks_json: &DiplomatStr,
            weights: &[f32],
            seed: i32,
        ) -> Result<Box<ScatterRule>, NucleationError> {
            let blocks = parse_blocks_json(blocks_json)?;
            if blocks.len() != weights.len() {
                return Err(NucleationError::InvalidArgument);
            }
            let entries = blocks.into_iter().zip(weights.iter().copied()).collect();
            crate::sdf::ScatterRule::new(face.to_core(), density, entries, seed)
                .map(|r| Box::new(ScatterRule(r)))
                .map_err(map_field_error)
        }

        pub fn set_condition(&mut self, condition: &MaterialCondition) {
            self.0.condition = condition.0.clone();
        }

        pub fn set_on_block(&mut self, prefix: &DiplomatStr) -> Result<(), NucleationError> {
            let s = std::str::from_utf8(prefix).map_err(|_| NucleationError::InvalidArgument)?;
            self.0.on_block = if s.is_empty() {
                None
            } else {
                Some(s.to_string())
            };
            Ok(())
        }

        pub fn set_height_range(&mut self, min: u32, max: u32) -> Result<(), NucleationError> {
            if min == 0 || min > max || max > 64 {
                return Err(NucleationError::InvalidArgument);
            }
            self.0.height_min = min;
            self.0.height_max = max;
            Ok(())
        }
    }

    /// Unified brick-pruned, parallel voxelization pipeline.
    #[diplomat::opaque_mut]
    pub struct VoxelPipeline(pub(crate) crate::sdf::VoxelPipeline);

    impl VoxelPipeline {
        pub fn create(geometry: &Field, material: &Material) -> Box<VoxelPipeline> {
            Box::new(VoxelPipeline(crate::sdf::VoxelPipeline::new(
                geometry.0.clone(),
                material.0.clone(),
            )))
        }

        pub fn add_fluid(
            &mut self,
            field: &Field,
            block: &DiplomatStr,
        ) -> Result<(), NucleationError> {
            let b = std::str::from_utf8(block).map_err(|_| NucleationError::InvalidArgument)?;
            self.0
                .fluids
                .push(crate::sdf::FluidLayer::new(field.0.clone(), b));
            Ok(())
        }

        pub fn add_fluid_level(
            &mut self,
            y_level: f32,
            block: &DiplomatStr,
        ) -> Result<(), NucleationError> {
            let b = std::str::from_utf8(block).map_err(|_| NucleationError::InvalidArgument)?;
            let layer = crate::sdf::FluidLayer::level(y_level, b).map_err(map_field_error)?;
            self.0.fluids.push(layer);
            Ok(())
        }

        pub fn add_scatter(&mut self, rule: &ScatterRule) {
            self.0.scatter.push(rule.0.clone());
        }

        pub fn set_fill_replace(&mut self) {
            self.0.fill_mode = crate::building::FillMode::Replace;
        }

        pub fn set_fill_keep_existing(&mut self) {
            self.0.fill_mode = crate::building::FillMode::KeepExisting;
        }

        pub fn set_fill_replacing(
            &mut self,
            targets_json: &DiplomatStr,
        ) -> Result<(), NucleationError> {
            let json =
                std::str::from_utf8(targets_json).map_err(|_| NucleationError::InvalidArgument)?;
            let targets: Vec<String> =
                serde_json::from_str(json).map_err(|_| NucleationError::Parse)?;
            self.0.fill_mode = crate::building::FillMode::ReplaceOnly(targets);
            Ok(())
        }

        pub fn to_schematic(&self, name: &DiplomatStr) -> Result<Box<Schematic>, NucleationError> {
            let n = std::str::from_utf8(name).map_err(|_| NucleationError::InvalidArgument)?;
            self.0
                .to_schematic(None, n)
                .map(|s| Box::new(Schematic(s)))
                .map_err(|_| NucleationError::InvalidArgument)
        }

        #[allow(clippy::too_many_arguments)]
        pub fn to_schematic_bounded(
            &self,
            name: &DiplomatStr,
            min_x: i32,
            min_y: i32,
            min_z: i32,
            max_x: i32,
            max_y: i32,
            max_z: i32,
        ) -> Result<Box<Schematic>, NucleationError> {
            let n = std::str::from_utf8(name).map_err(|_| NucleationError::InvalidArgument)?;
            let bounds = crate::sdf::SampleBounds::new([min_x, min_y, min_z], [max_x, max_y, max_z])
                .map_err(|_| NucleationError::InvalidArgument)?;
            self.0
                .to_schematic(Some(bounds), n)
                .map(|s| Box::new(Schematic(s)))
                .map_err(|_| NucleationError::InvalidArgument)
        }

        pub fn apply_to_schematic(&self, schematic: &mut Schematic) -> Result<(), NucleationError> {
            self.0
                .apply_to_schematic(&mut schematic.0, None)
                .map_err(|_| NucleationError::InvalidArgument)
        }

        #[allow(clippy::too_many_arguments)]
        pub fn apply_to_schematic_bounded(
            &self,
            schematic: &mut Schematic,
            min_x: i32,
            min_y: i32,
            min_z: i32,
            max_x: i32,
            max_y: i32,
            max_z: i32,
        ) -> Result<(), NucleationError> {
            let bounds = crate::sdf::SampleBounds::new([min_x, min_y, min_z], [max_x, max_y, max_z])
                .map_err(|_| NucleationError::InvalidArgument)?;
            self.0
                .apply_to_schematic(&mut schematic.0, Some(bounds))
                .map_err(|_| NucleationError::InvalidArgument)
        }
    }

    // ── Helpers ───────────────────────────────────────────────────────────

    fn wrap(res: Result<crate::sdf::Field, crate::sdf::FieldError>) -> Result<Box<Field>, NucleationError> {
        res.map(|f| Box::new(Field(f))).map_err(map_field_error)
    }

    fn map_field_error(err: crate::sdf::FieldError) -> NucleationError {
        crate::bridge::set_last_error_detail(err.to_string());
        match err {
            crate::sdf::FieldError::Parse(_) => NucleationError::Parse,
            crate::sdf::FieldError::InputTooLarge
            | crate::sdf::FieldError::InvalidArgument(_)
            | crate::sdf::FieldError::BudgetExceeded(_) => NucleationError::InvalidArgument,
        }
    }

    fn opt_finite(has: bool, val: f32) -> Result<Option<f32>, NucleationError> {
        if !has {
            Ok(None)
        } else if val.is_finite() {
            Ok(Some(val))
        } else {
            Err(NucleationError::InvalidArgument)
        }
    }

    fn interp_space(space: InterpolationSpace) -> crate::building::InterpolationSpace {
        match space {
            InterpolationSpace::Rgb => crate::building::InterpolationSpace::Rgb,
            InterpolationSpace::Oklab => crate::building::InterpolationSpace::Oklab,
        }
    }

    fn parse_stops(
        stops: &[f32],
        colors: &[u8],
    ) -> Result<Vec<crate::building::GradientStop>, NucleationError> {
        let expected = stops
            .len()
            .checked_mul(3)
            .ok_or(NucleationError::InvalidArgument)?;
        if stops.is_empty() || colors.len() != expected {
            return Err(NucleationError::InvalidArgument);
        }
        Ok(stops
            .iter()
            .zip(colors.chunks_exact(3))
            .map(|(p, c)| crate::building::GradientStop {
                position: *p as f64,
                color: crate::blockpedia::ExtendedColorData::from_rgb(c[0], c[1], c[2]),
            })
            .collect())
    }

    fn parse_blocks_json(json: &DiplomatStr) -> Result<Vec<crate::BlockState>, NucleationError> {
        let s = std::str::from_utf8(json).map_err(|_| NucleationError::InvalidArgument)?;
        let ids: Vec<String> = serde_json::from_str(s).map_err(|_| NucleationError::Parse)?;
        Ok(ids.into_iter().map(crate::BlockState::new).collect())
    }
}
