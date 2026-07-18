//! Shapes, brushes and the building tool. Port of `ffi/building.rs`.
//!
//! One `Shape` opaque wraps [`crate::building::ShapeEnum`]; one `Brush` opaque wraps
//! [`crate::building::BrushEnum`]. Combinators (`union_with`, `intersection_with`,
//! `difference_with`, `hollow`) clone their inputs, exactly like the old
//! `shape_union`/`shape_hollow` did.

#[diplomat::bridge]
pub mod ffi {
    use super::super::schematic::ffi::Schematic;
    use super::super::shared::ffi::NucleationError;
    use diplomat_runtime::DiplomatWrite;
    use std::fmt::Write;

    /// Color interpolation space for gradient brushes. The old ABI passed this as
    /// `space: c_int` (`1` = Oklab, anything else = RGB).
    pub enum InterpolationSpace {
        Rgb,
        Oklab,
    }

    impl InterpolationSpace {
        fn to_core(self) -> crate::building::InterpolationSpace {
            match self {
                InterpolationSpace::Rgb => crate::building::InterpolationSpace::Rgb,
                InterpolationSpace::Oklab => crate::building::InterpolationSpace::Oklab,
            }
        }
    }

    /// A solid region of blocks: primitives (sphere, cuboid, …) and boolean
    /// combinations thereof. Wraps `ShapeEnum`.
    #[diplomat::opaque]
    pub struct Shape(pub(crate) crate::building::ShapeEnum);

    impl Shape {
        /// Sphere centered at (`cx`, `cy`, `cz`) (truncated to block coordinates,
        /// matching the old `shape_sphere`).
        pub fn sphere(cx: f32, cy: f32, cz: f32, radius: f32) -> Box<Shape> {
            Box::new(Shape(crate::building::ShapeEnum::Sphere(
                crate::building::Sphere::new((cx as i32, cy as i32, cz as i32), radius as f64),
            )))
        }

        /// Axis-aligned box spanning the two corners (inclusive).
        pub fn cuboid(
            min_x: i32,
            min_y: i32,
            min_z: i32,
            max_x: i32,
            max_y: i32,
            max_z: i32,
        ) -> Box<Shape> {
            Box::new(Shape(crate::building::ShapeEnum::Cuboid(
                crate::building::Cuboid::new((min_x, min_y, min_z), (max_x, max_y, max_z)),
            )))
        }

        /// Ellipsoid centered at (`cx`, `cy`, `cz`) with per-axis radii.
        pub fn ellipsoid(cx: i32, cy: i32, cz: i32, rx: f32, ry: f32, rz: f32) -> Box<Shape> {
            Box::new(Shape(crate::building::ShapeEnum::Ellipsoid(
                crate::building::Ellipsoid::new((cx, cy, cz), (rx as f64, ry as f64, rz as f64)),
            )))
        }

        /// Cylinder from base point along an axis vector.
        #[allow(clippy::too_many_arguments)]
        pub fn cylinder(
            bx: f32,
            by: f32,
            bz: f32,
            ax: f32,
            ay: f32,
            az: f32,
            radius: f32,
            height: f32,
        ) -> Box<Shape> {
            Box::new(Shape(crate::building::ShapeEnum::Cylinder(
                crate::building::Cylinder::new(
                    (bx as f64, by as f64, bz as f64),
                    (ax as f64, ay as f64, az as f64),
                    radius as f64,
                    height as f64,
                ),
            )))
        }

        /// Cylinder spanning the segment between two points.
        pub fn cylinder_between(
            x1: f32,
            y1: f32,
            z1: f32,
            x2: f32,
            y2: f32,
            z2: f32,
            radius: f32,
        ) -> Box<Shape> {
            Box::new(Shape(crate::building::ShapeEnum::Cylinder(
                crate::building::Cylinder::between(
                    (x1 as f64, y1 as f64, z1 as f64),
                    (x2 as f64, y2 as f64, z2 as f64),
                    radius as f64,
                ),
            )))
        }

        /// Cone with apex at (`ax`, `ay`, `az`) opening along direction (`dx`, `dy`, `dz`).
        #[allow(clippy::too_many_arguments)]
        pub fn cone(
            ax: f32,
            ay: f32,
            az: f32,
            dx: f32,
            dy: f32,
            dz: f32,
            radius: f32,
            height: f32,
        ) -> Box<Shape> {
            Box::new(Shape(crate::building::ShapeEnum::Cone(
                crate::building::Cone::new(
                    (ax as f64, ay as f64, az as f64),
                    (dx as f64, dy as f64, dz as f64),
                    radius as f64,
                    height as f64,
                ),
            )))
        }

        /// Torus centered at (`cx`, `cy`, `cz`) with the given major/minor radii and
        /// axis (`ax`, `ay`, `az`).
        #[allow(clippy::too_many_arguments)]
        pub fn torus(
            cx: f32,
            cy: f32,
            cz: f32,
            major_r: f32,
            minor_r: f32,
            ax: f32,
            ay: f32,
            az: f32,
        ) -> Box<Shape> {
            Box::new(Shape(crate::building::ShapeEnum::Torus(
                crate::building::Torus::new(
                    (cx as f64, cy as f64, cz as f64),
                    major_r as f64,
                    minor_r as f64,
                    (ax as f64, ay as f64, az as f64),
                ),
            )))
        }

        /// Rectangular pyramid: base center, half-extents, height, up-axis.
        #[allow(clippy::too_many_arguments)]
        pub fn pyramid(
            bx: f32,
            by: f32,
            bz: f32,
            half_w: f32,
            half_d: f32,
            height: f32,
            ax: f32,
            ay: f32,
            az: f32,
        ) -> Box<Shape> {
            Box::new(Shape(crate::building::ShapeEnum::Pyramid(
                crate::building::Pyramid::new(
                    (bx as f64, by as f64, bz as f64),
                    (half_w as f64, half_d as f64),
                    height as f64,
                    (ax as f64, ay as f64, az as f64),
                ),
            )))
        }

        /// Flat disk: center, radius, plane normal, thickness.
        #[allow(clippy::too_many_arguments)]
        pub fn disk(
            cx: f32,
            cy: f32,
            cz: f32,
            radius: f32,
            nx: f32,
            ny: f32,
            nz: f32,
            thickness: f32,
        ) -> Box<Shape> {
            Box::new(Shape(crate::building::ShapeEnum::Disk(
                crate::building::Disk::new(
                    (cx as f64, cy as f64, cz as f64),
                    radius as f64,
                    (nx as f64, ny as f64, nz as f64),
                    thickness as f64,
                ),
            )))
        }

        /// Finite plane patch: origin, two spanning vectors `u`/`v`, extents along
        /// each, thickness.
        #[allow(clippy::too_many_arguments)]
        pub fn plane(
            ox: f32,
            oy: f32,
            oz: f32,
            ux: f32,
            uy: f32,
            uz: f32,
            vx: f32,
            vy: f32,
            vz: f32,
            u_ext: f32,
            v_ext: f32,
            thickness: f32,
        ) -> Box<Shape> {
            Box::new(Shape(crate::building::ShapeEnum::Plane(
                crate::building::Plane::new(
                    (ox as f64, oy as f64, oz as f64),
                    (ux as f64, uy as f64, uz as f64),
                    (vx as f64, vy as f64, vz as f64),
                    u_ext as f64,
                    v_ext as f64,
                    thickness as f64,
                ),
            )))
        }

        /// Filled triangle between three vertices, thickened by `thickness`.
        #[allow(clippy::too_many_arguments)]
        pub fn triangle(
            ax: f32,
            ay: f32,
            az: f32,
            bx: f32,
            by: f32,
            bz: f32,
            cx: f32,
            cy: f32,
            cz: f32,
            thickness: f32,
        ) -> Box<Shape> {
            Box::new(Shape(crate::building::ShapeEnum::Triangle(
                crate::building::Triangle::new(
                    (ax as f64, ay as f64, az as f64),
                    (bx as f64, by as f64, bz as f64),
                    (cx as f64, cy as f64, cz as f64),
                    thickness as f64,
                ),
            )))
        }

        /// Line segment between two points, thickened by `thickness`.
        pub fn line(
            x1: f32,
            y1: f32,
            z1: f32,
            x2: f32,
            y2: f32,
            z2: f32,
            thickness: f32,
        ) -> Box<Shape> {
            Box::new(Shape(crate::building::ShapeEnum::Line(
                crate::building::Line::new(
                    (x1 as f64, y1 as f64, z1 as f64),
                    (x2 as f64, y2 as f64, z2 as f64),
                    thickness as f64,
                ),
            )))
        }

        /// Bézier curve through `control_points` (flat `[x0, y0, z0, x1, y1, z1, …]`,
        /// so the length must be a non-zero multiple of 3), thickened by `thickness`
        /// and sampled at `resolution` steps.
        pub fn bezier(
            control_points: &[f32],
            thickness: f32,
            resolution: u32,
        ) -> Result<Box<Shape>, NucleationError> {
            if control_points.is_empty() || !control_points.len().is_multiple_of(3) {
                return Err(NucleationError::InvalidArgument);
            }
            let points: Vec<(f64, f64, f64)> = control_points
                .chunks_exact(3)
                .map(|p| (p[0] as f64, p[1] as f64, p[2] as f64))
                .collect();
            Ok(Box::new(Shape(crate::building::ShapeEnum::BezierCurve(
                crate::building::BezierCurve::new(points, thickness as f64, resolution),
            ))))
        }

        /// Any SDF tree as a Shape: the same JSON the terrain sampler takes
        /// (primitives, smooth booleans, noise — see the SDF guide) becomes
        /// fillable with every brush, combinable with other shapes, and
        /// usable in masked fills. Normals come from the field gradient, so
        /// the shaded brush shades smooth blends smoothly. Errors with
        /// `Parse` on invalid JSON and `InvalidArgument` for unbounded trees
        /// (use `sdf_bounded`).
        pub fn sdf(sdf_json: &DiplomatStr) -> Result<Box<Shape>, NucleationError> {
            let json =
                std::str::from_utf8(sdf_json).map_err(|_| NucleationError::InvalidArgument)?;
            let node =
                crate::sdf::SdfNode::from_json(json).map_err(|_| NucleationError::Parse)?;
            let shape =
                crate::building::SdfShape::new(node).ok_or(NucleationError::InvalidArgument)?;
            Ok(Box::new(Shape(crate::building::ShapeEnum::Sdf(shape))))
        }

        /// Like `sdf`, with explicit sampling bounds (inclusive block
        /// coordinates) for unbounded trees such as planes.
        #[allow(clippy::too_many_arguments)]
        pub fn sdf_bounded(
            sdf_json: &DiplomatStr,
            min_x: i32,
            min_y: i32,
            min_z: i32,
            max_x: i32,
            max_y: i32,
            max_z: i32,
        ) -> Result<Box<Shape>, NucleationError> {
            let json =
                std::str::from_utf8(sdf_json).map_err(|_| NucleationError::InvalidArgument)?;
            let node =
                crate::sdf::SdfNode::from_json(json).map_err(|_| NucleationError::Parse)?;
            let shape = crate::building::SdfShape::with_bounds(
                node,
                (min_x, min_y, min_z),
                (max_x, max_y, max_z),
            );
            Ok(Box::new(Shape(crate::building::ShapeEnum::Sdf(shape))))
        }

        /// Hollowed-out copy of this shape with the given wall thickness (clones the
        /// input, like the old `shape_hollow`).
        pub fn hollow(&self, thickness: u32) -> Box<Shape> {
            Box::new(Shape(crate::building::ShapeEnum::Hollow(
                crate::building::Hollow::new(self.0.clone(), thickness),
            )))
        }

        /// Boolean union of this shape and `other` (clones both inputs).
        pub fn union_with(&self, other: &Shape) -> Box<Shape> {
            Box::new(Shape(crate::building::ShapeEnum::Union(
                crate::building::Union::new(self.0.clone(), other.0.clone()),
            )))
        }

        /// Boolean intersection of this shape and `other` (clones both inputs).
        pub fn intersection_with(&self, other: &Shape) -> Box<Shape> {
            Box::new(Shape(crate::building::ShapeEnum::Intersection(
                crate::building::Intersection::new(self.0.clone(), other.0.clone()),
            )))
        }

        /// Boolean difference: this shape minus `other` (clones both inputs).
        pub fn difference_with(&self, other: &Shape) -> Box<Shape> {
            Box::new(Shape(crate::building::ShapeEnum::Difference(
                crate::building::Difference::new(self.0.clone(), other.0.clone()),
            )))
        }
    }

    /// A set of colored blocks that color/gradient brushes snap their computed
    /// colors to (nearest neighbor in Oklab space). Wraps an Arc'd
    /// [`crate::building::BlockPalette`]; sharing one palette across many
    /// brushes is cheap.
    #[diplomat::opaque]
    pub struct Palette(pub(crate) std::sync::Arc<crate::building::BlockPalette>);

    impl Palette {
        /// Every block blockpedia knows a color for (the default palette
        /// brushes use when none is set).
        pub fn all() -> Box<Palette> {
            Box::new(Palette(std::sync::Arc::new(
                crate::building::BlockPalette::new_all(),
            )))
        }

        /// Only solid blocks: no transparency, gravity, tile entities, or
        /// support requirements.
        pub fn solid() -> Box<Palette> {
            Box::new(Palette(std::sync::Arc::new(
                crate::building::BlockPalette::new_solid(),
            )))
        }

        /// Conservative structural set (full building blocks).
        pub fn structural() -> Box<Palette> {
            Box::new(Palette(std::sync::Arc::new(
                crate::building::BlockPalette::new_structural(),
            )))
        }

        /// Decorative set: allows stairs/slabs but no tile entities.
        pub fn decorative() -> Box<Palette> {
            Box::new(Palette(std::sync::Arc::new(
                crate::building::BlockPalette::new_decorative(),
            )))
        }

        /// The 16 concrete colors (excludes concrete powder).
        pub fn concrete() -> Box<Palette> {
            Box::new(Palette(std::sync::Arc::new(
                crate::building::BlockPalette::new_concrete(),
            )))
        }

        /// The 16 wool colors.
        pub fn wool() -> Box<Palette> {
            Box::new(Palette(std::sync::Arc::new(
                crate::building::BlockPalette::new_wool(),
            )))
        }

        /// Terracotta colors (excludes glazed variants).
        pub fn terracotta() -> Box<Palette> {
            Box::new(Palette(std::sync::Arc::new(
                crate::building::BlockPalette::new_terracotta(),
            )))
        }

        /// Genuinely gray blocks: opaque full cubes whose measured color
        /// is near-neutral (low Oklab chroma) — judged from color data,
        /// not names.
        pub fn grayscale() -> Box<Palette> {
            Box::new(Palette(std::sync::Arc::new(
                crate::building::BlockPalette::new_grayscale(),
            )))
        }

        /// The planks family — a natural light→dark wood ramp.
        pub fn wood() -> Box<Palette> {
            Box::new(Palette(std::sync::Arc::new(
                crate::building::BlockPalette::new_wood(),
            )))
        }

        /// A copy of this palette ordered by perceptual lightness (Oklab L,
        /// dark → light). Combined with `block_ids_json`, gives a
        /// ready-to-index ramp: `ids[i]` for intensity `i / (len - 1)`.
        pub fn sorted_by_lightness(&self) -> Box<Palette> {
            Box::new(Palette(std::sync::Arc::new(self.0.sorted_by_lightness())))
        }

        /// JSON array of exactly `steps` DISTINCT block ids forming the
        /// smoothest ramp this palette can make from (`r1`,`g1`,`b1`) to
        /// (`r2`,`g2`,`b2`): targets are evenly spaced along the Oklab line
        /// and blocks are chosen by a minimum-cost monotonic matching, so
        /// off-hue blocks are penalized and no block repeats. Errors with
        /// `InvalidArgument` when the palette has fewer than `steps` blocks,
        /// `steps` is 0, or start equals end.
        #[allow(clippy::too_many_arguments)]
        pub fn ramp_ids_json(
            &self,
            r1: u8,
            g1: u8,
            b1: u8,
            r2: u8,
            g2: u8,
            b2: u8,
            steps: u32,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            let ids = self
                .0
                .ramp_ids((r1, g1, b1), (r2, g2, b2), steps as usize)
                .ok_or(NucleationError::InvalidArgument)?;
            let _ = write!(out, "{}", serde_json::to_string(&ids).unwrap_or_default());
            Ok(())
        }

        /// JSON array of exactly `steps` block ids sampling the color
        /// gradient from (`r1`,`g1`,`b1`) to (`r2`,`g2`,`b2`) in Oklab
        /// space, each step snapped to this palette's closest block. Built
        /// for value→block lookups (heatmaps, fractals): index the returned
        /// list by `intensity * (steps - 1)`. Entries may repeat on coarse
        /// palettes; errors with `NotFound` on an empty palette.
        #[allow(clippy::too_many_arguments)]
        pub fn gradient_ids_json(
            &self,
            r1: u8,
            g1: u8,
            b1: u8,
            r2: u8,
            g2: u8,
            b2: u8,
            steps: u32,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            if self.0.is_empty() {
                return Err(NucleationError::NotFound);
            }
            let ids = self.0.gradient_ids((r1, g1, b1), (r2, g2, b2), steps as usize);
            let _ = write!(out, "{}", serde_json::to_string(&ids).unwrap_or_default());
            Ok(())
        }

        /// Custom palette from a JSON array of block ids, e.g.
        /// `["minecraft:stone", "minecraft:oak_planks"]`. Ids blockpedia has
        /// no color for are silently skipped — check `len` afterwards.
        pub fn from_block_ids(ids_json: &DiplomatStr) -> Result<Box<Palette>, NucleationError> {
            let json =
                std::str::from_utf8(ids_json).map_err(|_| NucleationError::InvalidArgument)?;
            let ids: Vec<String> =
                serde_json::from_str(json).map_err(|_| NucleationError::Parse)?;
            let palette =
                crate::building::BlockPalette::from_block_ids(ids.iter().map(|s| s.as_str()));
            Ok(Box::new(Palette(std::sync::Arc::new(palette))))
        }

        /// Number of blocks in the palette.
        pub fn len(&self) -> usize {
            self.0.len()
        }

        /// The palette's block ids as a JSON array string.
        pub fn block_ids_json(&self, out: &mut DiplomatWrite) {
            let ids: Vec<&str> = self.0.block_ids().collect();
            let _ = write!(out, "{}", serde_json::to_string(&ids).unwrap_or_default());
        }

        /// The palette block whose color is closest (Oklab distance) to the
        /// given RGB. Errors with `NotFound` on an empty palette.
        pub fn closest_block(
            &self,
            r: u8,
            g: u8,
            b: u8,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            let target = crate::blockpedia::ExtendedColorData::from_rgb(r, g, b);
            let id = self.0.find_closest(&target).ok_or(NucleationError::NotFound)?;
            let _ = write!(out, "{}", id);
            Ok(())
        }
    }

    /// Filter-driven palette construction (wraps
    /// [`crate::building::PaletteBuilder`], which fronts blockpedia's
    /// `BlockFilter`). Call flag methods, then `build` — the builder is
    /// consumed; further calls error with `AlreadyConsumed`.
    #[diplomat::opaque_mut]
    pub struct PaletteBuilder(pub(crate) Option<crate::building::PaletteBuilder>);

    impl PaletteBuilder {
        /// A builder matching every colored block (no filters yet).
        pub fn create() -> Box<PaletteBuilder> {
            Box::new(PaletteBuilder(Some(crate::building::PaletteBuilder::new())))
        }

        /// Exclude gravity-affected blocks (sand, gravel, ...).
        pub fn exclude_falling(&mut self) -> Result<(), NucleationError> {
            let b = self.0.take().ok_or(NucleationError::AlreadyConsumed)?;
            self.0 = Some(b.exclude_falling());
            Ok(())
        }

        /// Exclude blocks with block entities (chests, furnaces, ...).
        pub fn exclude_tile_entities(&mut self) -> Result<(), NucleationError> {
            let b = self.0.take().ok_or(NucleationError::AlreadyConsumed)?;
            self.0 = Some(b.exclude_tile_entities());
            Ok(())
        }

        /// Keep only full cube blocks (no stairs, slabs, fences, ...).
        /// Metadata-driven: uses the official model geometry extracted from
        /// the vanilla jars, not block-name guessing.
        pub fn full_blocks_only(&mut self) -> Result<(), NucleationError> {
            let b = self.0.take().ok_or(NucleationError::AlreadyConsumed)?;
            self.0 = Some(b.full_blocks_only());
            Ok(())
        }

        /// Exclude blocks that need supporting blocks (torches, rails, ...).
        pub fn exclude_needs_support(&mut self) -> Result<(), NucleationError> {
            let b = self.0.take().ok_or(NucleationError::AlreadyConsumed)?;
            self.0 = Some(b.exclude_needs_support());
            Ok(())
        }

        /// Exclude transparent/translucent blocks (glass, leaves, ...).
        /// Metadata-driven: uses the per-block transparency flag from the
        /// block-data pipeline, not block-name guessing.
        pub fn exclude_transparent(&mut self) -> Result<(), NucleationError> {
            let b = self.0.take().ok_or(NucleationError::AlreadyConsumed)?;
            self.0 = Some(b.exclude_transparent());
            Ok(())
        }

        /// Exclude light-emitting blocks (glowstone, lanterns, ...).
        pub fn exclude_light_sources(&mut self) -> Result<(), NucleationError> {
            let b = self.0.take().ok_or(NucleationError::AlreadyConsumed)?;
            self.0 = Some(b.exclude_light_sources());
            Ok(())
        }

        /// Keep only blocks obtainable in survival.
        pub fn survival_only(&mut self) -> Result<(), NucleationError> {
            let b = self.0.take().ok_or(NucleationError::AlreadyConsumed)?;
            self.0 = Some(b.survival_obtainable_only());
            Ok(())
        }

        /// Exclude blocks whose id contains `keyword`.
        pub fn exclude_keyword(&mut self, keyword: &DiplomatStr) -> Result<(), NucleationError> {
            let kw = std::str::from_utf8(keyword).map_err(|_| NucleationError::InvalidArgument)?;
            let b = self.0.take().ok_or(NucleationError::AlreadyConsumed)?;
            self.0 = Some(b.exclude_keyword(kw));
            Ok(())
        }

        /// Keep only blocks whose id contains `keyword` (repeatable; matches
        /// any of the included keywords).
        pub fn include_keyword(&mut self, keyword: &DiplomatStr) -> Result<(), NucleationError> {
            let kw = std::str::from_utf8(keyword).map_err(|_| NucleationError::InvalidArgument)?;
            let b = self.0.take().ok_or(NucleationError::AlreadyConsumed)?;
            self.0 = Some(b.include_keyword(kw));
            Ok(())
        }

        /// Require the vanilla block tag `t` (`minecraft:wool` or short
        /// `wool`, nested paths like `mineable/pickaxe` too). Repeatable —
        /// a block must carry ALL required tags (AND semantics).
        pub fn tag(&mut self, t: &DiplomatStr) -> Result<(), NucleationError> {
            let tag = std::str::from_utf8(t).map_err(|_| NucleationError::InvalidArgument)?;
            let b = self.0.take().ok_or(NucleationError::AlreadyConsumed)?;
            self.0 = Some(b.tag(tag));
            Ok(())
        }

        /// Exclude blocks carrying the vanilla block tag `t` (any listed
        /// tag disqualifies). Repeatable.
        pub fn exclude_tag(&mut self, t: &DiplomatStr) -> Result<(), NucleationError> {
            let tag = std::str::from_utf8(t).map_err(|_| NucleationError::InvalidArgument)?;
            let b = self.0.take().ok_or(NucleationError::AlreadyConsumed)?;
            self.0 = Some(b.exclude_tag(tag));
            Ok(())
        }

        /// Keep only blocks of the official definition kind `k`
        /// (`minecraft:stair` or short `stair`; plain full blocks are
        /// `minecraft:block`). Repeatable — a block matching ANY listed
        /// kind passes (OR semantics).
        pub fn kind(&mut self, k: &DiplomatStr) -> Result<(), NucleationError> {
            let kind = std::str::from_utf8(k).map_err(|_| NucleationError::InvalidArgument)?;
            let b = self.0.take().ok_or(NucleationError::AlreadyConsumed)?;
            self.0 = Some(b.kind(kind));
            Ok(())
        }

        /// Keep only blocks whose measured Oklab lightness L is within
        /// `[min, max]` (0.0 = black, 1.0 = white).
        pub fn lightness_between(&mut self, min: f32, max: f32) -> Result<(), NucleationError> {
            let b = self.0.take().ok_or(NucleationError::AlreadyConsumed)?;
            self.0 = Some(b.lightness_between(min, max));
            Ok(())
        }

        /// Keep only near-neutral blocks: measured Oklab chroma at most
        /// `max` (the grayscale preset uses 0.022).
        pub fn chroma_below(&mut self, max: f32) -> Result<(), NucleationError> {
            let b = self.0.take().ok_or(NucleationError::AlreadyConsumed)?;
            self.0 = Some(b.chroma_below(max));
            Ok(())
        }

        /// Keep only blocks whose measured color is within `max_distance`
        /// (Oklab; ~0.05 = same family, ~0.15 = generous) of the RGB color.
        pub fn color_near(
            &mut self,
            r: u8,
            g: u8,
            b: u8,
            max_distance: f32,
        ) -> Result<(), NucleationError> {
            let builder = self.0.take().ok_or(NucleationError::AlreadyConsumed)?;
            self.0 = Some(builder.color_near(r, g, b, max_distance));
            Ok(())
        }

        /// Build the palette; consumes the builder.
        pub fn build(&mut self) -> Result<Box<Palette>, NucleationError> {
            let b = self.0.take().ok_or(NucleationError::AlreadyConsumed)?;
            Ok(Box::new(Palette(std::sync::Arc::new(b.build()))))
        }
    }

    /// Decides which block goes at each point of a filled shape. Wraps `BrushEnum`.
    #[diplomat::opaque_mut]
    pub struct Brush(pub(crate) crate::building::BrushEnum);

    impl Brush {
        /// Every point becomes `block_name` (a block-state string).
        pub fn solid(block_name: &DiplomatStr) -> Result<Box<Brush>, NucleationError> {
            let name =
                std::str::from_utf8(block_name).map_err(|_| NucleationError::InvalidArgument)?;
            Ok(Box::new(Brush(crate::building::BrushEnum::Solid(
                crate::building::SolidBrush::new(crate::BlockState::new(name.to_owned())),
            ))))
        }

        /// Nearest-block-to-RGB-color brush.
        pub fn color(r: u8, g: u8, b: u8) -> Box<Brush> {
            Box::new(Brush(crate::building::BrushEnum::Color(
                crate::building::ColorBrush::new(r, g, b),
            )))
        }

        /// Linear color gradient between two anchored points.
        #[allow(clippy::too_many_arguments)]
        pub fn linear_gradient(
            x1: i32,
            y1: i32,
            z1: i32,
            r1: u8,
            g1: u8,
            b1: u8,
            x2: i32,
            y2: i32,
            z2: i32,
            r2: u8,
            g2: u8,
            b2: u8,
            space: InterpolationSpace,
        ) -> Box<Brush> {
            let brush = crate::building::LinearGradientBrush::new(
                (x1, y1, z1),
                (r1, g1, b1),
                (x2, y2, z2),
                (r2, g2, b2),
            )
            .with_space(space.to_core());
            Box::new(Brush(crate::building::BrushEnum::Linear(brush)))
        }

        /// Base color shaded by surface normal against light direction
        /// (`lx`, `ly`, `lz`).
        pub fn shaded(r: u8, g: u8, b: u8, lx: f32, ly: f32, lz: f32) -> Box<Brush> {
            Box::new(Brush(crate::building::BrushEnum::Shaded(
                crate::building::ShadedBrush::new((r, g, b), (lx as f64, ly as f64, lz as f64)),
            )))
        }

        /// Bilinear gradient over the patch `origin + s*u + t*v` with corner colors
        /// c00/c10/c01/c11.
        #[allow(clippy::too_many_arguments)]
        pub fn bilinear_gradient(
            ox: i32,
            oy: i32,
            oz: i32,
            ux: i32,
            uy: i32,
            uz: i32,
            vx: i32,
            vy: i32,
            vz: i32,
            r00: u8,
            g00: u8,
            b00: u8,
            r10: u8,
            g10: u8,
            b10: u8,
            r01: u8,
            g01: u8,
            b01: u8,
            r11: u8,
            g11: u8,
            b11: u8,
            space: InterpolationSpace,
        ) -> Box<Brush> {
            let brush = crate::building::BilinearGradientBrush::new(
                (ox, oy, oz),
                (ux, uy, uz),
                (vx, vy, vz),
                (r00, g00, b00),
                (r10, g10, b10),
                (r01, g01, b01),
                (r11, g11, b11),
            )
            .with_space(space.to_core());
            Box::new(Brush(crate::building::BrushEnum::Bilinear(brush)))
        }

        /// Inverse-distance-weighted gradient between colored anchor points.
        /// `positions` is flat `[x0, y0, z0, x1, …]` and `colors` is flat
        /// `[r0, g0, b0, r1, …]`; both must describe the same non-zero number of
        /// points (`positions.len() == colors.len()`, a multiple of 3).
        pub fn point_gradient(
            positions: &[i32],
            colors: &[u8],
            falloff: f32,
            space: InterpolationSpace,
        ) -> Result<Box<Brush>, NucleationError> {
            if positions.is_empty()
                || !positions.len().is_multiple_of(3)
                || colors.len() != positions.len()
            {
                return Err(NucleationError::InvalidArgument);
            }
            #[allow(clippy::type_complexity)]
            let points: Vec<((i32, i32, i32), (u8, u8, u8))> = positions
                .chunks_exact(3)
                .zip(colors.chunks_exact(3))
                .map(|(p, c)| ((p[0], p[1], p[2]), (c[0], c[1], c[2])))
                .collect();
            let brush = crate::building::PointGradientBrush::new(points)
                .with_space(space.to_core())
                .with_falloff(falloff as f64);
            Ok(Box::new(Brush(crate::building::BrushEnum::Point(brush))))
        }

        /// Spotlight-lit base color (`r`, `g`, `b`): Lambert shading toward a
        /// cone light at (`px`, `py`, `pz`) aimed along (`dx`, `dy`, `dz`).
        /// Full intensity inside 0.7 × `cone_angle_deg`, smoothstep falloff
        /// to zero at the cone edge; surfaces facing away or outside the cone
        /// drop to a 4% ambient floor.
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
        ) -> Box<Brush> {
            Box::new(Brush(crate::building::BrushEnum::Spotlight(
                crate::building::SpotlightBrush::new(
                    (px as f64, py as f64, pz as f64),
                    (dx as f64, dy as f64, dz as f64),
                    cone_angle_deg as f64,
                    (r, g, b),
                ),
            )))
        }

        /// Use `palette` for this brush's color→block snapping instead of the
        /// default all-blocks palette. No-op for `solid` brushes, which place
        /// a fixed block state. Set it before filling; the palette is shared,
        /// not copied.
        pub fn set_palette(&mut self, palette: &Palette) {
            self.0.set_palette(palette.0.clone());
        }

        /// Gradient along a parametric curve: `stops` holds the curve parameters in
        /// `[0, 1]` and `colors` the matching flat RGB triples
        /// (`colors.len() == stops.len() * 3`, `stops` non-empty).
        pub fn curve_gradient(
            stops: &[f32],
            colors: &[u8],
            space: InterpolationSpace,
        ) -> Result<Box<Brush>, NucleationError> {
            if stops.is_empty() || colors.len() != stops.len() * 3 {
                return Err(NucleationError::InvalidArgument);
            }
            let stops: Vec<(f64, (u8, u8, u8))> = stops
                .iter()
                .zip(colors.chunks_exact(3))
                .map(|(t, c)| (*t as f64, (c[0], c[1], c[2])))
                .collect();
            let brush = crate::building::CurveGradientBrush::new(stops).with_space(space.to_core());
            Ok(Box::new(Brush(crate::building::BrushEnum::CurveGradient(
                brush,
            ))))
        }
    }

    /// Namespace for the fill operations that combine a schematic, a shape and a
    /// brush (the old `buildingtool_*` free functions).
    #[diplomat::opaque]
    pub struct BuildingTool;

    impl BuildingTool {
        /// Fill `shape` into `schematic` using `brush`.
        pub fn fill(schematic: &mut Schematic, shape: &Shape, brush: &Brush) {
            let mut tool = crate::building::BuildingTool::new(&mut schematic.0);
            tool.fill_enum(&shape.0, &brush.0);
        }

        /// Fill `count` copies of `shape`, each offset by `offset * i`.
        pub fn rstack(
            schematic: &mut Schematic,
            shape: &Shape,
            brush: &Brush,
            count: usize,
            offset_x: i32,
            offset_y: i32,
            offset_z: i32,
        ) {
            let mut tool = crate::building::BuildingTool::new(&mut schematic.0);
            tool.rstack(&shape.0, &brush.0, count, (offset_x, offset_y, offset_z));
        }

        /// Masked fill that preserves everything already placed: `brush` is
        /// only written where `schematic` currently has air (or nothing at
        /// all), so existing structures inside `shape` survive untouched.
        pub fn fill_only_air(schematic: &mut Schematic, shape: &Shape, brush: &Brush) {
            let mut tool = crate::building::BuildingTool::new(&mut schematic.0);
            tool.fill_enum_masked(&shape.0, &brush.0, &crate::building::FillMode::KeepExisting);
        }

        /// Masked fill that only overwrites the listed blocks: `targets_json`
        /// is a JSON array of block ids (e.g. `["minecraft:stone"]`, state
        /// properties ignored) and every cell of `shape` whose current block
        /// id is in the list is replaced by `brush` — everything else,
        /// including air, is left alone.
        pub fn fill_replacing(
            schematic: &mut Schematic,
            shape: &Shape,
            brush: &Brush,
            targets_json: &DiplomatStr,
        ) -> Result<(), NucleationError> {
            let json =
                std::str::from_utf8(targets_json).map_err(|_| NucleationError::InvalidArgument)?;
            let targets: Vec<String> =
                serde_json::from_str(json).map_err(|_| NucleationError::Parse)?;
            let mut tool = crate::building::BuildingTool::new(&mut schematic.0);
            tool.fill_enum_masked(
                &shape.0,
                &brush.0,
                &crate::building::FillMode::ReplaceOnly(targets),
            );
            Ok(())
        }
    }
}
