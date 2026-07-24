//! Typed signed-distance-field graphs plus legacy JSON sampling entry points.

#[diplomat::bridge]
pub mod ffi {
    use super::super::building::ffi::Shape;
    use super::super::schematic::ffi::Schematic;
    use super::super::shared::ffi::NucleationError;
    use diplomat_runtime::DiplomatWrite;
    use std::fmt::Write;

    /// Axis used by mirror operations.
    pub enum SdfAxis {
        X,
        Y,
        Z,
    }

    impl SdfAxis {
        fn to_core(self) -> crate::sdf::Axis {
            match self {
                SdfAxis::X => crate::sdf::Axis::X,
                SdfAxis::Y => crate::sdf::Axis::Y,
                SdfAxis::Z => crate::sdf::Axis::Z,
            }
        }
    }

    /// Cellular/Worley field output.
    pub enum SdfCellMode {
        F1,
        F2,
        F2MinusF1,
        CellValue,
    }

    impl SdfCellMode {
        fn to_core(self) -> crate::sdf::CellMode {
            match self {
                SdfCellMode::F1 => crate::sdf::CellMode::F1,
                SdfCellMode::F2 => crate::sdf::CellMode::F2,
                SdfCellMode::F2MinusF1 => crate::sdf::CellMode::F2MinusF1,
                SdfCellMode::CellValue => crate::sdf::CellMode::Value,
            }
        }
    }

    /// Continuous bounds of a bounded SDF graph.
    #[diplomat::out]
    pub struct SdfBounds {
        pub min_x: f32,
        pub min_y: f32,
        pub min_z: f32,
        pub max_x: f32,
        pub max_y: f32,
        pub max_z: f32,
    }

    /// Unit surface normal estimated from the SDF gradient.
    #[diplomat::out]
    pub struct SdfNormal {
        pub x: f32,
        pub y: f32,
        pub z: f32,
    }

    /// An immutable, composable signed-distance-field expression graph.
    ///
    /// Primitive constructors and every combinator return a new graph, so values
    /// can be shared safely between Flow nodes and across Kotlin/Java, JavaScript,
    /// and Python bindings. JSON is retained only for explicit import/export and
    /// the legacy sampling helpers.
    #[diplomat::opaque]
    pub struct Sdf(pub(crate) crate::sdf::SdfNode);

    impl Sdf {
        // ── Typed primitives ──────────────────────────────────────────────

        pub fn sphere(radius: f32) -> Result<Box<Sdf>, NucleationError> {
            positive(&[radius])?;
            Ok(Box::new(Sdf(crate::sdf::SdfNode::Sphere { radius })))
        }

        /// Axis-aligned rounded box, centered at the origin.
        pub fn box_shape(
            half_x: f32,
            half_y: f32,
            half_z: f32,
            rounding: f32,
        ) -> Result<Box<Sdf>, NucleationError> {
            positive(&[half_x, half_y, half_z])?;
            non_negative(&[rounding])?;
            if rounding > half_x.min(half_y).min(half_z) {
                return Err(NucleationError::InvalidArgument);
            }
            Ok(Box::new(Sdf(crate::sdf::SdfNode::Box {
                half_extents: [half_x, half_y, half_z],
                rounding,
            })))
        }

        pub fn ellipsoid(
            radius_x: f32,
            radius_y: f32,
            radius_z: f32,
        ) -> Result<Box<Sdf>, NucleationError> {
            positive(&[radius_x, radius_y, radius_z])?;
            Ok(Box::new(Sdf(crate::sdf::SdfNode::Ellipsoid {
                radii: [radius_x, radius_y, radius_z],
            })))
        }

        pub fn torus(major_radius: f32, minor_radius: f32) -> Result<Box<Sdf>, NucleationError> {
            positive(&[major_radius, minor_radius])?;
            Ok(Box::new(Sdf(crate::sdf::SdfNode::Torus {
                major_radius,
                minor_radius,
            })))
        }

        /// Torus ring cut down to an arc. `cap_angle_degrees` is the half-aperture
        /// in `(0, 180]`, measured from +X and mirrored across X; `180` is a full
        /// torus.
        pub fn capped_torus(
            major_radius: f32,
            minor_radius: f32,
            cap_angle_degrees: f32,
        ) -> Result<Box<Sdf>, NucleationError> {
            positive(&[major_radius, minor_radius])?;
            finite(&[cap_angle_degrees])?;
            if cap_angle_degrees <= 0.0 || cap_angle_degrees > 180.0 {
                return Err(NucleationError::InvalidArgument);
            }
            Ok(Box::new(Sdf(crate::sdf::SdfNode::CappedTorus {
                major_radius,
                minor_radius,
                cap_angle: cap_angle_degrees,
            })))
        }

        /// Chain-link shape: a torus stretched along Z by `half_length` and
        /// capped by two half-tori. `half_length: 0` is a plain torus.
        pub fn link(
            major_radius: f32,
            minor_radius: f32,
            half_length: f32,
        ) -> Result<Box<Sdf>, NucleationError> {
            positive(&[major_radius, minor_radius])?;
            non_negative(&[half_length])?;
            Ok(Box::new(Sdf(crate::sdf::SdfNode::Link {
                major_radius,
                minor_radius,
                half_length,
            })))
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
        ) -> Result<Box<Sdf>, NucleationError> {
            finite(&[ax, ay, az, bx, by, bz])?;
            positive(&[radius])?;
            Ok(Box::new(Sdf(crate::sdf::SdfNode::Capsule {
                a: [ax, ay, az],
                b: [bx, by, bz],
                radius,
            })))
        }

        pub fn capped_cylinder(radius: f32, half_height: f32) -> Result<Box<Sdf>, NucleationError> {
            positive(&[radius, half_height])?;
            Ok(Box::new(Sdf(crate::sdf::SdfNode::CappedCylinder {
                radius,
                half_height,
            })))
        }

        /// Exact Y-axis cylinder with infinite extent. Sampling requires explicit bounds.
        pub fn infinite_cylinder(radius: f32) -> Result<Box<Sdf>, NucleationError> {
            positive(&[radius])?;
            Ok(Box::new(Sdf(crate::sdf::SdfNode::InfiniteCylinder {
                radius,
            })))
        }

        pub fn capped_cone(
            half_height: f32,
            bottom_radius: f32,
            top_radius: f32,
        ) -> Result<Box<Sdf>, NucleationError> {
            positive(&[half_height])?;
            non_negative(&[bottom_radius, top_radius])?;
            if bottom_radius == 0.0 && top_radius == 0.0 {
                return Err(NucleationError::InvalidArgument);
            }
            Ok(Box::new(Sdf(crate::sdf::SdfNode::CappedCone {
                half_height,
                r1: bottom_radius,
                r2: top_radius,
            })))
        }

        pub fn plane(
            normal_x: f32,
            normal_y: f32,
            normal_z: f32,
            offset: f32,
        ) -> Result<Box<Sdf>, NucleationError> {
            finite(&[normal_x, normal_y, normal_z, offset])?;
            let length =
                ((normal_x as f64).powi(2) + (normal_y as f64).powi(2) + (normal_z as f64).powi(2))
                    .sqrt();
            if !length.is_finite() || length <= f64::from(f32::EPSILON) {
                return Err(NucleationError::InvalidArgument);
            }
            let normal = [
                (f64::from(normal_x) / length) as f32,
                (f64::from(normal_y) / length) as f32,
                (f64::from(normal_z) / length) as f32,
            ];
            finite(&normal)?;
            Ok(Box::new(Sdf(crate::sdf::SdfNode::Plane { normal, offset })))
        }

        pub fn octahedron(size: f32) -> Result<Box<Sdf>, NucleationError> {
            positive(&[size])?;
            Ok(Box::new(Sdf(crate::sdf::SdfNode::Octahedron { size })))
        }

        pub fn hex_prism(radius: f32, half_height: f32) -> Result<Box<Sdf>, NucleationError> {
            positive(&[radius, half_height])?;
            Ok(Box::new(Sdf(crate::sdf::SdfNode::HexPrism {
                radius,
                half_height,
            })))
        }

        pub fn super_prism(
            half_x: f32,
            half_y: f32,
            half_z: f32,
            exponent: f32,
        ) -> Result<Box<Sdf>, NucleationError> {
            positive(&[half_x, half_y, half_z, exponent])?;
            Ok(Box::new(Sdf(crate::sdf::SdfNode::SuperPrism {
                half_extents: [half_x, half_y, half_z],
                exponent,
            })))
        }

        /// Hollow wireframe box: only the 12 edge beams are solid.
        pub fn box_frame(
            half_x: f32,
            half_y: f32,
            half_z: f32,
            thickness: f32,
        ) -> Result<Box<Sdf>, NucleationError> {
            positive(&[half_x, half_y, half_z])?;
            non_negative(&[thickness])?;
            if thickness > half_x.min(half_y).min(half_z) {
                return Err(NucleationError::InvalidArgument);
            }
            Ok(Box::new(Sdf(crate::sdf::SdfNode::BoxFrame {
                half_extents: [half_x, half_y, half_z],
                thickness,
            })))
        }

        pub fn cells(
            frequency: f32,
            seed: i32,
            jitter: f32,
            mode: SdfCellMode,
            threshold: f32,
        ) -> Result<Box<Sdf>, NucleationError> {
            positive(&[frequency])?;
            non_negative(&[jitter])?;
            finite(&[threshold])?;
            Ok(Box::new(Sdf(crate::sdf::SdfNode::Cells {
                frequency,
                seed,
                jitter,
                mode: mode.to_core(),
                threshold,
            })))
        }

        // ── Immutable composition ─────────────────────────────────────────

        pub fn union_with(&self, other: &Sdf) -> Box<Sdf> {
            Box::new(Sdf(crate::sdf::SdfNode::Union {
                children: vec![self.0.clone(), other.0.clone()],
            }))
        }

        pub fn intersection_with(&self, other: &Sdf) -> Box<Sdf> {
            Box::new(Sdf(crate::sdf::SdfNode::Intersect {
                children: vec![self.0.clone(), other.0.clone()],
            }))
        }

        pub fn subtract(&self, other: &Sdf) -> Box<Sdf> {
            Box::new(Sdf(crate::sdf::SdfNode::Subtract {
                a: Box::new(self.0.clone()),
                b: Box::new(other.0.clone()),
            }))
        }

        pub fn smooth_union(&self, other: &Sdf, radius: f32) -> Result<Box<Sdf>, NucleationError> {
            positive(&[radius])?;
            Ok(Box::new(Sdf(crate::sdf::SdfNode::SmoothUnion {
                a: Box::new(self.0.clone()),
                b: Box::new(other.0.clone()),
                k: radius,
            })))
        }

        pub fn smooth_subtract(
            &self,
            other: &Sdf,
            radius: f32,
        ) -> Result<Box<Sdf>, NucleationError> {
            positive(&[radius])?;
            Ok(Box::new(Sdf(crate::sdf::SdfNode::SmoothSubtract {
                a: Box::new(self.0.clone()),
                b: Box::new(other.0.clone()),
                k: radius,
            })))
        }

        pub fn smooth_intersection(
            &self,
            other: &Sdf,
            radius: f32,
        ) -> Result<Box<Sdf>, NucleationError> {
            positive(&[radius])?;
            Ok(Box::new(Sdf(crate::sdf::SdfNode::SmoothIntersect {
                a: Box::new(self.0.clone()),
                b: Box::new(other.0.clone()),
                k: radius,
            })))
        }

        pub fn rounded(&self, radius: f32) -> Result<Box<Sdf>, NucleationError> {
            non_negative(&[radius])?;
            Ok(Box::new(Sdf(crate::sdf::SdfNode::Round {
                child: Box::new(self.0.clone()),
                radius,
            })))
        }

        pub fn shell(&self, thickness: f32) -> Result<Box<Sdf>, NucleationError> {
            positive(&[thickness])?;
            Ok(Box::new(Sdf(crate::sdf::SdfNode::Shell {
                child: Box::new(self.0.clone()),
                thickness,
            })))
        }

        // ── Immutable transforms and modifiers ────────────────────────────

        pub fn translate(&self, x: f32, y: f32, z: f32) -> Result<Box<Sdf>, NucleationError> {
            finite(&[x, y, z])?;
            Ok(Box::new(Sdf(crate::sdf::SdfNode::Translate {
                child: Box::new(self.0.clone()),
                offset: [x, y, z],
            })))
        }

        pub fn rotate(
            &self,
            x_degrees: f32,
            y_degrees: f32,
            z_degrees: f32,
        ) -> Result<Box<Sdf>, NucleationError> {
            finite(&[x_degrees, y_degrees, z_degrees])?;
            Ok(Box::new(Sdf(crate::sdf::SdfNode::Rotate {
                child: Box::new(self.0.clone()),
                angles: [x_degrees, y_degrees, z_degrees],
            })))
        }

        pub fn scale(&self, factor: f32) -> Result<Box<Sdf>, NucleationError> {
            positive(&[factor])?;
            Ok(Box::new(Sdf(crate::sdf::SdfNode::Scale {
                child: Box::new(self.0.clone()),
                factor,
            })))
        }

        pub fn mirror(&self, axis: SdfAxis) -> Box<Sdf> {
            Box::new(Sdf(crate::sdf::SdfNode::Mirror {
                child: Box::new(self.0.clone()),
                axis: axis.to_core(),
            }))
        }

        pub fn repeat_infinite(
            &self,
            spacing_x: f32,
            spacing_y: f32,
            spacing_z: f32,
        ) -> Result<Box<Sdf>, NucleationError> {
            repeat(self, spacing_x, spacing_y, spacing_z, None)
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
        ) -> Result<Box<Sdf>, NucleationError> {
            repeat(
                self,
                spacing_x,
                spacing_y,
                spacing_z,
                Some([count_x, count_y, count_z]),
            )
        }

        pub fn displace(
            &self,
            amplitude: f32,
            frequency: f32,
            seed: i32,
            octaves: u32,
        ) -> Result<Box<Sdf>, NucleationError> {
            non_negative(&[amplitude])?;
            positive(&[frequency])?;
            if !(1..=8).contains(&octaves) {
                return Err(NucleationError::InvalidArgument);
            }
            Ok(Box::new(Sdf(crate::sdf::SdfNode::Displace {
                child: Box::new(self.0.clone()),
                amplitude,
                frequency,
                seed,
                octaves,
            })))
        }

        pub fn warp(
            &self,
            amplitude: f32,
            frequency: f32,
            seed: i32,
        ) -> Result<Box<Sdf>, NucleationError> {
            non_negative(&[amplitude])?;
            positive(&[frequency])?;
            Ok(Box::new(Sdf(crate::sdf::SdfNode::Warp {
                child: Box::new(self.0.clone()),
                amplitude,
                frequency,
                seed,
            })))
        }

        // ── Evaluation, conversion, and explicit serialization ────────────

        pub fn eval_at(&self, x: f32, y: f32, z: f32) -> f32 {
            self.0.eval(x, y, z)
        }

        pub fn normal(
            &self,
            x: f32,
            y: f32,
            z: f32,
            epsilon: f32,
        ) -> Result<SdfNormal, NucleationError> {
            finite(&[x, y, z])?;
            positive(&[epsilon])?;
            let [nx, ny, nz] = crate::sdf::numerical_normal(&self.0, [x, y, z], epsilon)
                .ok_or(NucleationError::InvalidArgument)?;
            Ok(SdfNormal {
                x: nx as f32,
                y: ny as f32,
                z: nz as f32,
            })
        }

        pub fn bounds(&self) -> Result<SdfBounds, NucleationError> {
            self.0
                .bounds()
                .map(|bounds| SdfBounds {
                    min_x: bounds.min[0],
                    min_y: bounds.min[1],
                    min_z: bounds.min[2],
                    max_x: bounds.max[0],
                    max_y: bounds.max[1],
                    max_z: bounds.max[2],
                })
                .ok_or(NucleationError::InvalidArgument)
        }

        pub fn to_shape(&self) -> Result<Box<Shape>, NucleationError> {
            crate::building::SdfShape::new(self.0.clone())
                .map(|shape| Box::new(Shape(crate::building::ShapeEnum::Sdf(shape))))
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
            let shape = crate::building::SdfShape::with_bounds(
                self.0.clone(),
                (min_x, min_y, min_z),
                (max_x, max_y, max_z),
            )
            .ok_or(NucleationError::InvalidArgument)?;
            Ok(Box::new(Shape(crate::building::ShapeEnum::Sdf(shape))))
        }

        pub fn from_json_string(json: &DiplomatStr) -> Result<Box<Sdf>, NucleationError> {
            let json = std::str::from_utf8(json).map_err(|_| NucleationError::InvalidArgument)?;
            crate::sdf::SdfNode::from_json(json)
                .map(|node| Box::new(Sdf(node)))
                .map_err(|_| NucleationError::Parse)
        }

        pub fn to_json(&self, write: &mut DiplomatWrite) -> Result<(), NucleationError> {
            let json = self.0.to_json().map_err(|_| NucleationError::Serialize)?;
            let _ = write!(write, "{json}");
            Ok(())
        }

        /// Wrap a validated [`FieldProgram`] as an `Sdf` graph (cloning it,
        /// with its own explicit bounds and distance-kind metadata), so it
        /// composes with every other combinator.
        pub fn from_program(program: &FieldProgram) -> Box<Sdf> {
            Box::new(Sdf(crate::sdf::SdfNode::Program {
                program: Box::new(program.0.clone()),
            }))
        }

        // ── Legacy JSON-first API ─────────────────────────────────────────

        /// Legacy JSON-first terrain helper. Prefer typed constructors and
        /// `to_shape()` with `BuildingTool.fill()` for new code.
        pub fn schematic_from_sdf_auto(
            sdf_json: &DiplomatStr,
            rules_json: &DiplomatStr,
        ) -> Result<Box<Schematic>, NucleationError> {
            Self::schematic_from_sdf(sdf_json, rules_json, false, 0, 0, 0, 0, 0, 0)
        }

        /// Legacy JSON-first terrain helper with optional explicit bounds.
        #[allow(clippy::too_many_arguments)]
        pub fn schematic_from_sdf(
            sdf_json: &DiplomatStr,
            rules_json: &DiplomatStr,
            has_bounds: bool,
            min_x: i32,
            min_y: i32,
            min_z: i32,
            max_x: i32,
            max_y: i32,
            max_z: i32,
        ) -> Result<Box<Schematic>, NucleationError> {
            let sdf_str =
                std::str::from_utf8(sdf_json).map_err(|_| NucleationError::InvalidArgument)?;
            let rules_str =
                std::str::from_utf8(rules_json).map_err(|_| NucleationError::InvalidArgument)?;
            let node =
                crate::sdf::SdfNode::from_json(sdf_str).map_err(|_| NucleationError::Parse)?;
            let rules = crate::sdf::MaterialRules::from_json(rules_str)
                .map_err(|_| NucleationError::Parse)?;
            let bounds = if has_bounds {
                Some(crate::sdf::SampleBounds {
                    min: [min_x, min_y, min_z],
                    max: [max_x, max_y, max_z],
                })
            } else {
                None
            };
            crate::sdf::sample_to_schematic(&node, &rules, bounds, "sdf")
                .map(|schematic| Box::new(Schematic(schematic)))
                .map_err(|_| NucleationError::InvalidArgument)
        }

        /// Legacy JSON-first evaluator. Prefer `Sdf.from_json_string(...).eval_at(...)`.
        pub fn eval(
            sdf_json: &DiplomatStr,
            x: f32,
            y: f32,
            z: f32,
        ) -> Result<f32, NucleationError> {
            let sdf_str =
                std::str::from_utf8(sdf_json).map_err(|_| NucleationError::InvalidArgument)?;
            let node =
                crate::sdf::SdfNode::from_json(sdf_str).map_err(|_| NucleationError::Parse)?;
            Ok(node.eval(x, y, z))
        }
    }

    // ── Portable custom field programs ─────────────────────────────────────

    /// The type of a value on a [`FieldProgramBuilder`]'s stack or in a slot.
    pub enum FieldProgramValueType {
        Scalar,
        Vec3,
        Bool,
    }

    impl FieldProgramValueType {
        fn to_core(self) -> crate::sdf::ValueType {
            match self {
                FieldProgramValueType::Scalar => crate::sdf::ValueType::Scalar,
                FieldProgramValueType::Vec3 => crate::sdf::ValueType::Vec3,
                FieldProgramValueType::Bool => crate::sdf::ValueType::Bool,
            }
        }
    }

    /// Unary field-program operations (see `crate::sdf::UnaryOp`).
    pub enum FieldProgramUnaryOp {
        Neg,
        Abs,
        Sqrt,
        Log,
        Sin,
        Cos,
        Acos,
        VecX,
        VecY,
        VecZ,
        Length,
        Normalize,
    }

    impl FieldProgramUnaryOp {
        fn to_core(self) -> crate::sdf::UnaryOp {
            use crate::sdf::UnaryOp as U;
            match self {
                FieldProgramUnaryOp::Neg => U::Neg,
                FieldProgramUnaryOp::Abs => U::Abs,
                FieldProgramUnaryOp::Sqrt => U::Sqrt,
                FieldProgramUnaryOp::Log => U::Log,
                FieldProgramUnaryOp::Sin => U::Sin,
                FieldProgramUnaryOp::Cos => U::Cos,
                FieldProgramUnaryOp::Acos => U::Acos,
                FieldProgramUnaryOp::VecX => U::VecX,
                FieldProgramUnaryOp::VecY => U::VecY,
                FieldProgramUnaryOp::VecZ => U::VecZ,
                FieldProgramUnaryOp::Length => U::Length,
                FieldProgramUnaryOp::Normalize => U::Normalize,
            }
        }
    }

    /// Binary field-program operations (see `crate::sdf::BinaryOp`). `Add`
    /// and `Sub` accept either two scalars or two vec3s.
    pub enum FieldProgramBinaryOp {
        Add,
        Sub,
        Mul,
        Div,
        Min,
        Max,
        Pow,
        Atan2,
        Lt,
        Le,
        Gt,
        Ge,
        Eq,
        Dot,
        Cross,
        /// `vec3 * scalar`, componentwise.
        Scale,
    }

    impl FieldProgramBinaryOp {
        fn to_core(self) -> crate::sdf::BinaryOp {
            use crate::sdf::BinaryOp as B;
            match self {
                FieldProgramBinaryOp::Add => B::Add,
                FieldProgramBinaryOp::Sub => B::Sub,
                FieldProgramBinaryOp::Mul => B::Mul,
                FieldProgramBinaryOp::Div => B::Div,
                FieldProgramBinaryOp::Min => B::Min,
                FieldProgramBinaryOp::Max => B::Max,
                FieldProgramBinaryOp::Pow => B::Pow,
                FieldProgramBinaryOp::Atan2 => B::Atan2,
                FieldProgramBinaryOp::Lt => B::Lt,
                FieldProgramBinaryOp::Le => B::Le,
                FieldProgramBinaryOp::Gt => B::Gt,
                FieldProgramBinaryOp::Ge => B::Ge,
                FieldProgramBinaryOp::Eq => B::Eq,
                FieldProgramBinaryOp::Dot => B::Dot,
                FieldProgramBinaryOp::Cross => B::Cross,
                FieldProgramBinaryOp::Scale => B::Scale,
            }
        }
    }

    /// What kind of distance a field program's output represents.
    pub enum FieldProgramDistanceKind {
        Exact,
        LowerBound,
        Estimate,
        Implicit,
    }

    impl FieldProgramDistanceKind {
        fn to_core(self) -> crate::sdf::DistanceKind {
            match self {
                FieldProgramDistanceKind::Exact => crate::sdf::DistanceKind::Exact,
                FieldProgramDistanceKind::LowerBound => crate::sdf::DistanceKind::LowerBound,
                FieldProgramDistanceKind::Estimate => crate::sdf::DistanceKind::Estimate,
                FieldProgramDistanceKind::Implicit => crate::sdf::DistanceKind::Implicit,
            }
        }

        fn from_core(kind: crate::sdf::DistanceKind) -> Self {
            match kind {
                crate::sdf::DistanceKind::Exact => FieldProgramDistanceKind::Exact,
                crate::sdf::DistanceKind::LowerBound => FieldProgramDistanceKind::LowerBound,
                crate::sdf::DistanceKind::Estimate => FieldProgramDistanceKind::Estimate,
                crate::sdf::DistanceKind::Implicit => FieldProgramDistanceKind::Implicit,
            }
        }
    }

    /// A validated, sandboxed custom SDF field program: deterministic typed
    /// bytecode over scalar/vec3/bool values with bounded loops, carrying
    /// its own explicit finite bounds and distance-kind metadata. Build one
    /// with [`FieldProgramBuilder`] or import it from JSON.
    #[diplomat::opaque]
    pub struct FieldProgram(pub(crate) crate::sdf::Program);

    impl FieldProgram {
        pub fn from_json_string(json: &DiplomatStr) -> Result<Box<FieldProgram>, NucleationError> {
            let json = std::str::from_utf8(json).map_err(|_| NucleationError::InvalidArgument)?;
            crate::sdf::Program::from_json(json)
                .map(|program| Box::new(FieldProgram(program)))
                .map_err(|_| NucleationError::Parse)
        }

        pub fn to_json(&self, write: &mut DiplomatWrite) -> Result<(), NucleationError> {
            let json = self.0.to_json().map_err(|_| NucleationError::Serialize)?;
            let _ = write!(write, "{json}");
            Ok(())
        }

        pub fn eval_at(&self, x: f32, y: f32, z: f32) -> f32 {
            self.0.eval(x, y, z)
        }

        /// Unit-length gradient of the scalar output at `(x, y, z)`: the
        /// program's own forward-mode analytic gradient where it's
        /// differentiable there, falling back to a numerical estimate
        /// (central differences via `epsilon`) otherwise.
        pub fn gradient(
            &self,
            x: f32,
            y: f32,
            z: f32,
            epsilon: f32,
        ) -> Result<SdfNormal, NucleationError> {
            finite(&[x, y, z])?;
            positive(&[epsilon])?;
            let normal = self.0.analytic_gradient(x, y, z).or_else(|| {
                let node = crate::sdf::SdfNode::Program {
                    program: Box::new(self.0.clone()),
                };
                crate::sdf::numerical_normal(&node, [x, y, z], epsilon)
                    .map(|n| [n[0] as f32, n[1] as f32, n[2] as f32])
            });
            let normal = normal.ok_or(NucleationError::InvalidArgument)?;
            Ok(SdfNormal {
                x: normal[0],
                y: normal[1],
                z: normal[2],
            })
        }

        pub fn bounds(&self) -> SdfBounds {
            let aabb = self.0.aabb();
            SdfBounds {
                min_x: aabb.min[0],
                min_y: aabb.min[1],
                min_z: aabb.min[2],
                max_x: aabb.max[0],
                max_y: aabb.max[1],
                max_z: aabb.max[2],
            }
        }

        pub fn distance_kind(&self) -> FieldProgramDistanceKind {
            FieldProgramDistanceKind::from_core(self.0.distance_kind())
        }
    }

    /// Programmatic builder for a [`FieldProgram`]: append typed stack
    /// instructions, then [`FieldProgramBuilder::build`] to validate and
    /// obtain a [`FieldProgram`]. Consuming: every method after `build()`
    /// (successful or not) returns `AlreadyConsumed`.
    #[diplomat::opaque_mut]
    pub struct FieldProgramBuilder(Option<crate::sdf::ProgramBuilder>);

    impl FieldProgramBuilder {
        pub fn create() -> Box<FieldProgramBuilder> {
            Box::new(FieldProgramBuilder(Some(crate::sdf::ProgramBuilder::new())))
        }

        /// Declare a new typed local slot and return its index.
        pub fn add_slot(
            &mut self,
            value_type: FieldProgramValueType,
        ) -> Result<u16, NucleationError> {
            let slot = self.inner_mut()?.add_slot(value_type.to_core());
            if slot == u16::MAX {
                Err(NucleationError::InvalidArgument)
            } else {
                Ok(slot)
            }
        }

        pub fn push_const_scalar(&mut self, value: f32) -> Result<(), NucleationError> {
            finite(&[value])?;
            self.inner_mut()?
                .push_const(crate::sdf::Const::Scalar(value));
            Ok(())
        }

        pub fn push_const_vec3(&mut self, x: f32, y: f32, z: f32) -> Result<(), NucleationError> {
            finite(&[x, y, z])?;
            self.inner_mut()?
                .push_const(crate::sdf::Const::Vec3([x, y, z]));
            Ok(())
        }

        pub fn push_const_bool(&mut self, value: bool) -> Result<(), NucleationError> {
            self.inner_mut()?.push_const(crate::sdf::Const::Bool(value));
            Ok(())
        }

        /// Push the `Vec3` position the program is being evaluated at.
        pub fn push_pos(&mut self) -> Result<(), NucleationError> {
            self.inner_mut()?.push_pos();
            Ok(())
        }

        pub fn load_local(&mut self, slot: u16) -> Result<(), NucleationError> {
            self.inner_mut()?.load_local(slot);
            Ok(())
        }

        pub fn store_local(&mut self, slot: u16) -> Result<(), NucleationError> {
            self.inner_mut()?.store_local(slot);
            Ok(())
        }

        /// Discard the top of the stack.
        pub fn pop(&mut self) -> Result<(), NucleationError> {
            self.inner_mut()?.pop();
            Ok(())
        }

        pub fn unary_op(&mut self, op: FieldProgramUnaryOp) -> Result<(), NucleationError> {
            self.inner_mut()?.unary(op.to_core());
            Ok(())
        }

        pub fn binary_op(&mut self, op: FieldProgramBinaryOp) -> Result<(), NucleationError> {
            self.inner_mut()?.binary(op.to_core());
            Ok(())
        }

        /// Pop `(x, lo, hi)`, push `x` clamped to `[lo, hi]`.
        pub fn clamp(&mut self) -> Result<(), NucleationError> {
            self.inner_mut()?.clamp();
            Ok(())
        }

        /// Pop `(a, b, cond)`, push `a` if `cond` else `b`.
        pub fn select(&mut self) -> Result<(), NucleationError> {
            self.inner_mut()?.select();
            Ok(())
        }

        /// Pop `(x, y, z)`, push `Vec3([x, y, z])`.
        pub fn make_vec3(&mut self) -> Result<(), NucleationError> {
            self.inner_mut()?.make_vec3();
            Ok(())
        }

        /// Pop a `Bool`; if true, stop the nearest enclosing repeat after
        /// this iteration. Only valid inside `beginRepeat`/`endRepeat`.
        pub fn break_if(&mut self) -> Result<(), NucleationError> {
            self.inner_mut()?.break_if();
            Ok(())
        }

        /// Open a new statically bounded repeat block; subsequent
        /// instructions append to its body until `endRepeat`.
        pub fn begin_repeat(&mut self, count: u32) -> Result<(), NucleationError> {
            self.inner_mut()?.begin_repeat(count);
            Ok(())
        }

        /// Close the innermost open repeat block.
        pub fn end_repeat(&mut self) -> Result<(), NucleationError> {
            self.inner_mut()?
                .end_repeat()
                .map_err(|_| NucleationError::InvalidArgument)?;
            Ok(())
        }

        /// Declare which scalar slot holds the program's output.
        pub fn set_output(&mut self, slot: u16) -> Result<(), NucleationError> {
            self.inner_mut()?.set_output(slot);
            Ok(())
        }

        /// Set the program's explicit, author-asserted finite bounds.
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
            finite(&[min_x, min_y, min_z, max_x, max_y, max_z])?;
            self.inner_mut()?
                .set_bounds([min_x, min_y, min_z], [max_x, max_y, max_z]);
            Ok(())
        }

        pub fn set_distance_kind(
            &mut self,
            kind: FieldProgramDistanceKind,
        ) -> Result<(), NucleationError> {
            self.inner_mut()?.set_distance_kind(kind.to_core());
            Ok(())
        }

        /// Validate and finalize. Consumes the builder even on failure.
        pub fn build(&mut self) -> Result<Box<FieldProgram>, NucleationError> {
            let inner = self.0.take().ok_or(NucleationError::AlreadyConsumed)?;
            inner
                .build()
                .map(|program| Box::new(FieldProgram(program)))
                .map_err(|_| NucleationError::InvalidArgument)
        }

        fn inner_mut(&mut self) -> Result<&mut crate::sdf::ProgramBuilder, NucleationError> {
            self.0.as_mut().ok_or(NucleationError::AlreadyConsumed)
        }
    }

    fn finite(values: &[f32]) -> Result<(), NucleationError> {
        if values.iter().all(|value| value.is_finite()) {
            Ok(())
        } else {
            Err(NucleationError::InvalidArgument)
        }
    }

    fn positive(values: &[f32]) -> Result<(), NucleationError> {
        finite(values)?;
        if values.iter().all(|value| *value > 0.0) {
            Ok(())
        } else {
            Err(NucleationError::InvalidArgument)
        }
    }

    fn non_negative(values: &[f32]) -> Result<(), NucleationError> {
        finite(values)?;
        if values.iter().all(|value| *value >= 0.0) {
            Ok(())
        } else {
            Err(NucleationError::InvalidArgument)
        }
    }

    fn repeat(
        sdf: &Sdf,
        spacing_x: f32,
        spacing_y: f32,
        spacing_z: f32,
        count: Option<[u32; 3]>,
    ) -> Result<Box<Sdf>, NucleationError> {
        finite(&[spacing_x, spacing_y, spacing_z])?;
        if spacing_x < 0.0
            || spacing_y < 0.0
            || spacing_z < 0.0
            || (spacing_x == 0.0 && spacing_y == 0.0 && spacing_z == 0.0)
        {
            return Err(NucleationError::InvalidArgument);
        }
        Ok(Box::new(Sdf(crate::sdf::SdfNode::Repeat {
            child: Box::new(sdf.0.clone()),
            spacing: [spacing_x, spacing_y, spacing_z],
            count,
        })))
    }
}

#[cfg(test)]
mod tests {
    use super::ffi::Sdf;

    #[test]
    fn typed_sdf_graph_composes_without_json() {
        let left = Sdf::sphere(4.0).unwrap().translate(-3.0, 0.0, 0.0).unwrap();
        let right = Sdf::box_shape(3.0, 3.0, 3.0, 0.5)
            .unwrap()
            .translate(3.0, 0.0, 0.0)
            .unwrap();
        let field = left.smooth_union(&right, 1.25).unwrap();

        assert!(field.eval_at(-3.0, 0.0, 0.0) < 0.0);
        assert!(field.eval_at(3.0, 0.0, 0.0) < 0.0);
        assert!(field.eval_at(20.0, 0.0, 0.0) > 0.0);

        let bounds = field.bounds().unwrap();
        assert!(bounds.min_x <= -7.0 && bounds.max_x >= 6.5);
        assert!(field.to_shape().is_ok());
    }

    #[test]
    fn typed_sdf_exposes_normal_and_explicit_json_roundtrip() {
        let field = Sdf::ellipsoid(2.0, 3.0, 4.0).unwrap();
        let normal = field.normal(2.0, 0.0, 0.0, 0.01).unwrap();
        assert!((normal.x - 1.0).abs() < 0.01);
        assert!(normal.y.abs() < 0.01);
        assert!(normal.z.abs() < 0.01);

        let json = field.0.to_json().unwrap();
        let restored = Sdf::from_json_string(json.as_bytes()).unwrap();
        assert!((restored.eval_at(2.0, 0.0, 0.0) - field.eval_at(2.0, 0.0, 0.0)).abs() < 1e-6);
    }

    #[test]
    fn typed_shapes_reject_unsafe_inferred_and_explicit_bounds() {
        assert!(Sdf::sphere(20_000_000.0).unwrap().to_shape().is_err());
        let plane = Sdf::plane(0.0, 1.0, 0.0, 0.0).unwrap();
        assert!(plane
            .to_shape_bounded(i32::MIN, 0, 0, i32::MAX, 0, 0)
            .is_err());
        assert!(plane.to_shape_bounded(0, 0, 0, 255, 255, 255).is_ok());
        assert!(plane.to_shape_bounded(0, 0, 0, 256, 255, 255).is_err());
    }

    #[test]
    fn box_frame_validates_and_bounds_match_outer_extents() {
        assert!(Sdf::box_frame(2.0, 2.0, 2.0, 0.25).is_ok());
        // Thickness cannot exceed the smallest half-extent.
        assert!(Sdf::box_frame(2.0, 2.0, 2.0, 2.5).is_err());
        assert!(Sdf::box_frame(-1.0, 2.0, 2.0, 0.25).is_err());
        assert!(Sdf::box_frame(f32::NAN, 2.0, 2.0, 0.25).is_err());
        assert!(Sdf::box_frame(f32::INFINITY, 2.0, 2.0, 0.25).is_err());

        let huge = Sdf::box_frame(1.0e30, 1.0e30, 1.0e30, 0.1).unwrap();
        assert!(huge.to_shape().is_err(), "huge frame exceeds voxel budget");

        let frame = Sdf::box_frame(2.0, 2.0, 2.0, 0.25).unwrap();
        let bounds = frame.bounds().unwrap();
        assert!((bounds.min_x + 2.0).abs() < 1e-6);
        assert!((bounds.max_x - 2.0).abs() < 1e-6);
    }

    #[test]
    fn capped_torus_validates_angle_and_rejects_huge_radii() {
        assert!(Sdf::capped_torus(5.0, 1.0, 90.0).is_ok());
        assert!(Sdf::capped_torus(5.0, 1.0, 0.0).is_err());
        assert!(Sdf::capped_torus(5.0, 1.0, 181.0).is_err());
        assert!(Sdf::capped_torus(5.0, 1.0, f32::NAN).is_err());
        assert!(Sdf::capped_torus(-1.0, 1.0, 90.0).is_err());

        let huge = Sdf::capped_torus(1.0e30, 1.0, 90.0).unwrap();
        assert!(huge.to_shape().is_err(), "huge torus exceeds voxel budget");
    }

    #[test]
    fn link_validates_and_rejects_huge_radii() {
        assert!(Sdf::link(3.0, 0.75, 4.0).is_ok());
        assert!(Sdf::link(3.0, 0.75, 0.0).is_ok());
        assert!(Sdf::link(-1.0, 0.75, 4.0).is_err());
        assert!(Sdf::link(3.0, 0.75, -1.0).is_err());
        assert!(Sdf::link(3.0, 0.75, f32::NAN).is_err());

        let huge = Sdf::link(1.0e30, 1.0, 1.0e30).unwrap();
        assert!(huge.to_shape().is_err(), "huge link exceeds voxel budget");
    }

    #[test]
    fn infinite_cylinder_is_typed_unbounded_and_validated() {
        assert!(Sdf::infinite_cylinder(2.0).is_ok());
        assert!(Sdf::infinite_cylinder(0.0).is_err());
        assert!(Sdf::infinite_cylinder(-1.0).is_err());
        assert!(Sdf::infinite_cylinder(f32::NAN).is_err());
        let cylinder = Sdf::infinite_cylinder(2.0).unwrap();
        assert!(cylinder.bounds().is_err());
        assert!((cylinder.eval_at(5.0, 1.0e9, 0.0) - 3.0).abs() < 1e-3);
    }

    #[test]
    fn plane_normalization_is_robust_for_large_finite_components() {
        let plane = Sdf::plane(f32::MAX, 0.0, 0.0, 0.0).unwrap();
        assert!((plane.eval_at(2.0, 0.0, 0.0) - 2.0).abs() < 1e-6);
        let normal = plane.normal(0.0, 0.0, 0.0, 0.01).unwrap();
        assert!((normal.x - 1.0).abs() < 1e-6);
        assert!(normal.y.abs() < 1e-6);
        assert!(normal.z.abs() < 1e-6);
        let extreme = plane.normal(i32::MAX as f32, 0.0, 0.0, 0.5).unwrap();
        assert!((extreme.x - 1.0).abs() < 1e-6);
        assert!(extreme.y.abs() < 1e-6);
        assert!(extreme.z.abs() < 1e-6);
        let endpoint = plane.normal(f32::MAX, 0.0, 0.0, 0.5).unwrap();
        assert!((endpoint.x - 1.0).abs() < 1e-6);
    }

    use super::ffi::{
        FieldProgram, FieldProgramBinaryOp, FieldProgramBuilder, FieldProgramDistanceKind,
        FieldProgramUnaryOp, FieldProgramValueType,
    };

    fn sphere_builder(radius: f32) -> (Box<FieldProgramBuilder>, u16) {
        let mut b = FieldProgramBuilder::create();
        let out = b.add_slot(FieldProgramValueType::Scalar).unwrap();
        b.set_output(out).unwrap();
        b.set_bounds(-radius, -radius, -radius, radius, radius, radius)
            .unwrap();
        b.set_distance_kind(FieldProgramDistanceKind::Exact)
            .unwrap();
        b.push_pos().unwrap();
        b.unary_op(FieldProgramUnaryOp::Length).unwrap();
        b.push_const_scalar(radius).unwrap();
        b.binary_op(FieldProgramBinaryOp::Sub).unwrap();
        b.store_local(out).unwrap();
        (b, out)
    }

    #[test]
    fn field_program_builder_builds_sphere_and_evaluates() {
        let (mut b, _) = sphere_builder(2.0);
        let program = b.build().unwrap();
        assert!(program.eval_at(0.0, 0.0, 0.0) < 0.0);
        assert!((program.eval_at(2.0, 0.0, 0.0)).abs() < 1e-5);
        assert!(program.eval_at(10.0, 0.0, 0.0) > 0.0);
        let bounds = program.bounds();
        assert_eq!(bounds.min_x, -2.0);
        assert_eq!(bounds.max_x, 2.0);
    }

    #[test]
    fn field_program_builder_rejects_invalid_output_slot() {
        let mut b = FieldProgramBuilder::create();
        let flag = b.add_slot(FieldProgramValueType::Bool).unwrap();
        b.set_output(flag).unwrap();
        b.set_bounds(-1.0, -1.0, -1.0, 1.0, 1.0, 1.0).unwrap();
        b.push_const_bool(true).unwrap();
        b.store_local(flag).unwrap();
        assert!(b.build().is_err());
    }

    #[test]
    fn field_program_builder_rejects_slots_past_the_limit_immediately() {
        let mut builder = FieldProgramBuilder::create();
        for _ in 0..crate::sdf::MAX_SLOTS {
            builder.add_slot(FieldProgramValueType::Scalar).unwrap();
        }
        assert!(builder.add_slot(FieldProgramValueType::Scalar).is_err());
    }

    #[test]
    fn field_program_builder_after_build_is_already_consumed() {
        let (mut b, out) = sphere_builder(1.0);
        assert!(b.build().is_ok());
        assert!(b.store_local(out).is_err());
        assert!(b.build().is_err());
    }

    #[test]
    fn field_program_json_roundtrip() {
        let (mut b, _) = sphere_builder(3.5);
        let program = b.build().unwrap();
        let json = program.0.to_json().unwrap();
        let restored = FieldProgram::from_json_string(json.as_bytes()).unwrap();
        assert!((program.eval_at(1.0, 1.0, 1.0) - restored.eval_at(1.0, 1.0, 1.0)).abs() < 1e-6);
        assert!(FieldProgram::from_json_string(b"{ not json").is_err());
    }

    #[test]
    fn field_program_builder_bounded_repeat_break_if() {
        let mut b = FieldProgramBuilder::create();
        let counter = b.add_slot(FieldProgramValueType::Scalar).unwrap();
        b.set_output(counter).unwrap();
        b.set_bounds(-1.0, -1.0, -1.0, 1.0, 1.0, 1.0).unwrap();

        b.begin_repeat(1000).unwrap();
        b.load_local(counter).unwrap();
        b.push_const_scalar(1.0).unwrap();
        b.binary_op(FieldProgramBinaryOp::Add).unwrap();
        b.store_local(counter).unwrap();

        b.load_local(counter).unwrap();
        b.push_const_scalar(5.0).unwrap();
        b.binary_op(FieldProgramBinaryOp::Ge).unwrap();
        b.break_if().unwrap();
        b.end_repeat().unwrap();

        let program = b.build().unwrap();
        assert!((program.eval_at(0.0, 0.0, 0.0) - 5.0).abs() < 1e-6);
    }

    #[test]
    fn field_program_gradient_returns_unit_normal() {
        let (mut b, _) = sphere_builder(2.0);
        let program = b.build().unwrap();
        let normal = program.gradient(2.0, 0.0, 0.0, 0.01).unwrap();
        assert!((normal.x - 1.0).abs() < 1e-3);
        assert!(normal.y.abs() < 1e-3);
        assert!(normal.z.abs() < 1e-3);
    }

    #[test]
    fn field_program_eval_rejects_non_finite_inputs_and_results() {
        let mut builder = FieldProgramBuilder::create();
        let output = builder.add_slot(FieldProgramValueType::Scalar).unwrap();
        builder.set_output(output).unwrap();
        builder.set_bounds(-1.0, -1.0, -1.0, 1.0, 1.0, 1.0).unwrap();
        builder.push_const_scalar(0.0).unwrap();
        builder.push_const_scalar(0.0).unwrap();
        builder.binary_op(FieldProgramBinaryOp::Div).unwrap();
        builder.store_local(output).unwrap();
        let program = builder.build().unwrap();

        assert_eq!(program.eval_at(0.0, 0.0, 0.0), f32::INFINITY);
        assert_eq!(program.eval_at(f32::NAN, 0.0, 0.0), f32::INFINITY);
        assert_eq!(program.eval_at(f32::INFINITY, 0.0, 0.0), f32::INFINITY);
    }

    #[test]
    fn sdf_from_program_composes_and_reports_bounds() {
        let (mut b, _) = sphere_builder(2.0);
        let program = b.build().unwrap();
        let field = Sdf::from_program(&program);
        let other = Sdf::sphere(1.0).unwrap().translate(10.0, 0.0, 0.0).unwrap();
        let union = field.union_with(&other);

        assert!(union.eval_at(0.0, 0.0, 0.0) < 0.0);
        assert!(union.eval_at(10.0, 0.0, 0.0) < 0.0);
        assert!(union.eval_at(5.0, 0.0, 0.0) > 0.0);

        let bounds = union.bounds().unwrap();
        assert!(bounds.min_x <= -2.0);
        assert!(bounds.max_x >= 11.0);
    }
}
