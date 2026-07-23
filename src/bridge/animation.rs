//! Construction-shaped animation API exposed through every generated binding.

use crate::bridge::shared::ffi::NucleationError;

fn utf8(bytes: &[u8]) -> Result<&str, NucleationError> {
    std::str::from_utf8(bytes).map_err(|_| NucleationError::InvalidArgument)
}

fn exclusions_json(bytes: &[u8]) -> Result<Vec<String>, NucleationError> {
    serde_json::from_str(utf8(bytes)?).map_err(|_| NucleationError::InvalidArgument)
}

fn property(name: &str) -> Result<crate::animation::Property, NucleationError> {
    use crate::animation::Property::*;
    match name {
        "x" => Ok(X),
        "y" => Ok(Y),
        "z" => Ok(Z),
        "rotateX" | "rotate_x" => Ok(RotX),
        "rotateY" | "rotate_y" => Ok(RotY),
        "rotateZ" | "rotate_z" => Ok(RotZ),
        "scale" => Ok(ScaleUniform),
        "scaleX" | "scale_x" => Ok(ScaleX),
        "scaleY" | "scale_y" => Ok(ScaleY),
        "scaleZ" | "scale_z" => Ok(ScaleZ),
        "opacity" => Ok(Opacity),
        "tintR" | "tint_r" => Ok(TintR),
        "tintG" | "tint_g" => Ok(TintG),
        "tintB" | "tint_b" => Ok(TintB),
        "tintA" | "tint_a" => Ok(TintA),
        "emissiveR" | "emissive_r" => Ok(EmissiveR),
        "emissiveG" | "emissive_g" => Ok(EmissiveG),
        "emissiveB" | "emissive_b" => Ok(EmissiveB),
        _ => Err(NucleationError::InvalidArgument),
    }
}

fn easing(name: &str) -> Result<crate::animation::Easing, NucleationError> {
    use crate::animation::{Easing, Power};
    match name {
        "linear" => Ok(Easing::Linear),
        "inQuad" | "in_quad" => Ok(Easing::In(Power::Quad)),
        "outQuad" | "out_quad" => Ok(Easing::Out(Power::Quad)),
        "inOutQuad" | "in_out_quad" => Ok(Easing::InOut(Power::Quad)),
        "inCubic" | "in_cubic" => Ok(Easing::In(Power::Cubic)),
        "outCubic" | "out_cubic" => Ok(Easing::Out(Power::Cubic)),
        "inOutCubic" | "in_out_cubic" => Ok(Easing::InOut(Power::Cubic)),
        "inOutSine" | "in_out_sine" => Ok(Easing::InOut(Power::Sine)),
        "outBack" | "out_back" => Ok(Easing::out_back()),
        _ => Err(NucleationError::InvalidArgument),
    }
}

#[diplomat::bridge]
pub mod ffi {
    use super::super::building::ffi::{Brush, Shape};
    #[cfg(all(feature = "rendering", not(target_arch = "wasm32")))]
    use super::super::rendering::ffi::RenderConfig;
    use super::super::schematic::ffi::Schematic;
    use super::super::shared::ffi::NucleationError;
    use super::{easing, exclusions_json, property, utf8};
    use diplomat_runtime::DiplomatWrite;
    use std::fmt::Write;

    /// A reusable set of property tracks, modelled after Anime.js object animations.
    #[diplomat::opaque_mut]
    pub struct AnimationEffect(pub(crate) crate::animation::AnimationEffect);

    impl AnimationEffect {
        pub fn create(duration_ms: f32) -> Box<Self> {
            Box::new(Self(crate::animation::AnimationEffect::new(duration_ms)))
        }

        pub fn instant() -> Box<Self> {
            Self::create(0.0)
        }

        pub fn pop_in(duration_ms: f32) -> Box<Self> {
            Box::new(Self(crate::animation::presets::pop_in(duration_ms).into()))
        }

        pub fn drop_in(duration_ms: f32, height: f32) -> Box<Self> {
            Box::new(Self(
                crate::animation::presets::drop_in(duration_ms, height).into(),
            ))
        }

        pub fn drop_and_pop(duration_ms: f32, height: f32) -> Box<Self> {
            Box::new(Self(
                crate::animation::presets::drop_and_pop(duration_ms, height).into(),
            ))
        }

        pub fn spin_in(duration_ms: f32, turns: f32) -> Box<Self> {
            Box::new(Self(
                crate::animation::presets::spin_in(duration_ms, turns).into(),
            ))
        }

        pub fn turntable(duration_ms: f32) -> Box<Self> {
            Box::new(Self(
                crate::animation::presets::turntable(duration_ms).into(),
            ))
        }

        /// Add a two-key property tween. Property names follow Anime.js/Three.js:
        /// `x`, `y`, `z`, `rotateX`, `rotateY`, `rotateZ`, `scale`, `opacity`,
        /// `tintR/G/B/A`, and `emissiveR/G/B`.
        pub fn add_tween(
            &mut self,
            property_name: &DiplomatStr,
            from: f32,
            to: f32,
            easing_name: &DiplomatStr,
        ) -> Result<(), NucleationError> {
            let p = property(utf8(property_name)?)?;
            let e = easing(utf8(easing_name)?)?;
            self.0 = self.0.clone().tween(p, from, to, e);
            Ok(())
        }

        /// Add a normalised keyframe (`at` in `0..=1`) to a property track.
        pub fn add_keyframe(
            &mut self,
            property_name: &DiplomatStr,
            at: f32,
            value: f32,
            easing_name: &DiplomatStr,
        ) -> Result<(), NucleationError> {
            let p = property(utf8(property_name)?)?;
            let e = easing(utf8(easing_name)?)?;
            self.0 = self.0.clone().keyframe(p, at, value, e);
            Ok(())
        }

        pub fn set_repeat_forever(&mut self) {
            self.0 = self.0.clone().repeat_forever();
        }
    }

    /// A schematic wrapper that records construction calls as animation targets.
    #[diplomat::opaque_mut]
    pub struct BuildAnimation(pub(crate) crate::animation::BuildAnimation);

    impl BuildAnimation {
        pub fn create(name: &DiplomatStr) -> Box<Self> {
            let name = std::str::from_utf8(name).unwrap_or("animation");
            Box::new(Self(crate::animation::BuildAnimation::new(name)))
        }

        pub fn set_default_effect(&mut self, effect: &AnimationEffect) {
            self.0.set_default_effect(effect.0.clone());
        }

        /// Apply an effect to exactly the next recorded operation or explicit group.
        /// The returned borrowed builder enables fluent calls in every generated binding.
        pub fn with_effect<'a>(&'a mut self, effect: &AnimationEffect) -> &'a mut BuildAnimation {
            self.0.with_effect(effect.0.clone());
            self
        }

        pub fn set_step_ms(&mut self, step_ms: f32) {
            self.0.set_step_ms(step_ms);
        }

        pub fn set_stagger_total_ms(&mut self, total_ms: f32) {
            self.0.set_stagger_total_ms(Some(total_ms));
        }

        pub fn clear_stagger(&mut self) {
            self.0.set_stagger_total_ms(None);
        }

        /// Shift every construction group's start time. Negative offsets let a
        /// repeating staggered effect cross the beginning of a loop capture.
        pub fn set_stagger_offset_ms(&mut self, offset_ms: f32) {
            self.0.set_stagger_offset_ms(offset_ms);
        }

        /// Capture exactly one loop period, excluding the duplicate endpoint.
        /// The rounded frame count evenly partitions the complete period.
        pub fn set_loop_period_ms(&mut self, period_ms: f32) -> Result<(), NucleationError> {
            if !period_ms.is_finite() || period_ms <= 0.0 {
                return Err(NucleationError::InvalidArgument);
            }
            self.0.set_loop_period_ms(Some(period_ms));
            Ok(())
        }

        pub fn clear_loop_period(&mut self) {
            self.0.set_loop_period_ms(None);
        }

        pub fn begin_group(&mut self) -> Result<(), NucleationError> {
            self.0
                .begin_group(None)
                .map_err(|_| NucleationError::InvalidArgument)
        }

        pub fn begin_keyed_group(&mut self, key: f32) -> Result<(), NucleationError> {
            self.0
                .begin_group_with_key(None, key)
                .map_err(|_| NucleationError::InvalidArgument)
        }

        pub fn end_group(&mut self) -> Result<u32, NucleationError> {
            self.0
                .end_group()
                .map_err(|_| NucleationError::InvalidArgument)
        }

        pub fn set_block(
            &mut self,
            x: i32,
            y: i32,
            z: i32,
            block: &DiplomatStr,
        ) -> Result<u32, NucleationError> {
            self.0
                .set_block(x, y, z, utf8(block)?)
                .map_err(|_| NucleationError::InvalidArgument)
        }

        pub fn create_region(
            &mut self,
            name: &DiplomatStr,
            min_x: i32,
            min_y: i32,
            min_z: i32,
            max_x: i32,
            max_y: i32,
            max_z: i32,
        ) -> Result<(), NucleationError> {
            self.0.schematic_mut().create_region(
                utf8(name)?.to_string(),
                (min_x, min_y, min_z),
                (max_x, max_y, max_z),
            );
            Ok(())
        }

        pub fn set_block_in_region(
            &mut self,
            region: &DiplomatStr,
            x: i32,
            y: i32,
            z: i32,
            block: &DiplomatStr,
        ) -> Result<u32, NucleationError> {
            self.0
                .set_block_in_region(utf8(region)?, x, y, z, utf8(block)?)
                .map_err(|_| NucleationError::InvalidArgument)
        }

        pub fn translate(
            &mut self,
            x: i32,
            y: i32,
            z: i32,
            duration_ms: f32,
        ) -> Result<(), NucleationError> {
            self.0
                .translate(x, y, z, duration_ms)
                .map_err(|_| NucleationError::InvalidArgument)
        }

        pub fn translate_region(
            &mut self,
            region: &DiplomatStr,
            x: i32,
            y: i32,
            z: i32,
            duration_ms: f32,
        ) -> Result<(), NucleationError> {
            self.0
                .translate_region(utf8(region)?, x, y, z, duration_ms)
                .map_err(|_| NucleationError::InvalidArgument)
        }

        pub fn translate_all(
            &mut self,
            x: i32,
            y: i32,
            z: i32,
            duration_ms: f32,
        ) -> Result<(), NucleationError> {
            self.0
                .translate_all(x, y, z, duration_ms)
                .map_err(|_| NucleationError::InvalidArgument)
        }

        pub fn rotate_x(&mut self, degrees: i32, duration_ms: f32) -> Result<(), NucleationError> {
            self.0
                .rotate_x(degrees, duration_ms)
                .map_err(|_| NucleationError::InvalidArgument)
        }
        pub fn rotate_y(&mut self, degrees: i32, duration_ms: f32) -> Result<(), NucleationError> {
            self.0
                .rotate_y(degrees, duration_ms)
                .map_err(|_| NucleationError::InvalidArgument)
        }
        pub fn rotate_z(&mut self, degrees: i32, duration_ms: f32) -> Result<(), NucleationError> {
            self.0
                .rotate_z(degrees, duration_ms)
                .map_err(|_| NucleationError::InvalidArgument)
        }
        pub fn rotate_region_x(
            &mut self,
            region: &DiplomatStr,
            degrees: i32,
            duration_ms: f32,
        ) -> Result<(), NucleationError> {
            self.0
                .rotate_region_x(utf8(region)?, degrees, duration_ms)
                .map_err(|_| NucleationError::InvalidArgument)
        }
        pub fn rotate_region_y(
            &mut self,
            region: &DiplomatStr,
            degrees: i32,
            duration_ms: f32,
        ) -> Result<(), NucleationError> {
            self.0
                .rotate_region_y(utf8(region)?, degrees, duration_ms)
                .map_err(|_| NucleationError::InvalidArgument)
        }
        pub fn rotate_region_z(
            &mut self,
            region: &DiplomatStr,
            degrees: i32,
            duration_ms: f32,
        ) -> Result<(), NucleationError> {
            self.0
                .rotate_region_z(utf8(region)?, degrees, duration_ms)
                .map_err(|_| NucleationError::InvalidArgument)
        }
        pub fn rotate_all_x(
            &mut self,
            degrees: i32,
            duration_ms: f32,
        ) -> Result<(), NucleationError> {
            self.0
                .rotate_all_x(degrees, duration_ms)
                .map_err(|_| NucleationError::InvalidArgument)
        }
        pub fn rotate_all_y(
            &mut self,
            degrees: i32,
            duration_ms: f32,
        ) -> Result<(), NucleationError> {
            self.0
                .rotate_all_y(degrees, duration_ms)
                .map_err(|_| NucleationError::InvalidArgument)
        }
        pub fn rotate_all_z(
            &mut self,
            degrees: i32,
            duration_ms: f32,
        ) -> Result<(), NucleationError> {
            self.0
                .rotate_all_z(degrees, duration_ms)
                .map_err(|_| NucleationError::InvalidArgument)
        }

        pub fn flip_x(&mut self, duration_ms: f32) -> Result<(), NucleationError> {
            self.0
                .flip_x(duration_ms)
                .map_err(|_| NucleationError::InvalidArgument)
        }
        pub fn flip_y(&mut self, duration_ms: f32) -> Result<(), NucleationError> {
            self.0
                .flip_y(duration_ms)
                .map_err(|_| NucleationError::InvalidArgument)
        }
        pub fn flip_z(&mut self, duration_ms: f32) -> Result<(), NucleationError> {
            self.0
                .flip_z(duration_ms)
                .map_err(|_| NucleationError::InvalidArgument)
        }
        pub fn flip_region_x(
            &mut self,
            region: &DiplomatStr,
            duration_ms: f32,
        ) -> Result<(), NucleationError> {
            self.0
                .flip_region_x(utf8(region)?, duration_ms)
                .map_err(|_| NucleationError::InvalidArgument)
        }
        pub fn flip_region_y(
            &mut self,
            region: &DiplomatStr,
            duration_ms: f32,
        ) -> Result<(), NucleationError> {
            self.0
                .flip_region_y(utf8(region)?, duration_ms)
                .map_err(|_| NucleationError::InvalidArgument)
        }
        pub fn flip_region_z(
            &mut self,
            region: &DiplomatStr,
            duration_ms: f32,
        ) -> Result<(), NucleationError> {
            self.0
                .flip_region_z(utf8(region)?, duration_ms)
                .map_err(|_| NucleationError::InvalidArgument)
        }
        pub fn flip_all_x(&mut self, duration_ms: f32) -> Result<(), NucleationError> {
            self.0
                .flip_all_x(duration_ms)
                .map_err(|_| NucleationError::InvalidArgument)
        }
        pub fn flip_all_y(&mut self, duration_ms: f32) -> Result<(), NucleationError> {
            self.0
                .flip_all_y(duration_ms)
                .map_err(|_| NucleationError::InvalidArgument)
        }
        pub fn flip_all_z(&mut self, duration_ms: f32) -> Result<(), NucleationError> {
            self.0
                .flip_all_z(duration_ms)
                .map_err(|_| NucleationError::InvalidArgument)
        }

        pub fn stamp_region(
            &mut self,
            source: &Schematic,
            region: &DiplomatStr,
            x: i32,
            y: i32,
            z: i32,
            exclusions: &DiplomatStr,
            duration_ms: f32,
        ) -> Result<(), NucleationError> {
            let excluded = exclusions_json(exclusions)?;
            self.0
                .stamp_region(&source.0, utf8(region)?, (x, y, z), &excluded, duration_ms)
                .map_err(|_| NucleationError::InvalidArgument)
        }

        pub fn stamp_box(
            &mut self,
            source: &Schematic,
            min_x: i32,
            min_y: i32,
            min_z: i32,
            max_x: i32,
            max_y: i32,
            max_z: i32,
            x: i32,
            y: i32,
            z: i32,
            exclusions: &DiplomatStr,
            duration_ms: f32,
        ) -> Result<(), NucleationError> {
            let excluded = exclusions_json(exclusions)?;
            self.0
                .stamp_box(
                    &source.0,
                    crate::BoundingBox::new((min_x, min_y, min_z), (max_x, max_y, max_z)),
                    (x, y, z),
                    &excluded,
                    duration_ms,
                )
                .map_err(|_| NucleationError::InvalidArgument)
        }

        pub fn set_operation_gizmos(&mut self, enabled: bool) {
            self.0.set_operation_gizmos(enabled);
        }

        pub fn operations_json(&self, out: &mut DiplomatWrite) -> Result<(), NucleationError> {
            let json = self
                .0
                .operations_json()
                .map_err(|_| NucleationError::Serialize)?;
            write!(out, "{}", json).map_err(|_| NucleationError::Serialize)
        }

        pub fn frame_json(
            &self,
            time_ms: f32,
            out: &mut DiplomatWrite,
        ) -> Result<(), NucleationError> {
            let json = serde_json::to_string(&self.0.frame_at(time_ms))
                .map_err(|_| NucleationError::Serialize)?;
            write!(out, "{}", json).map_err(|_| NucleationError::Serialize)
        }

        /// Fill a parametric shape and record its voxels as ordered groups in
        /// the same transactional construction operation.
        pub fn fill_along_parameter(
            &mut self,
            shape: &Shape,
            brush: &Brush,
            group_count: u32,
        ) -> Result<u32, NucleationError> {
            self.0
                .fill_along_parameter(&shape.0, &brush.0, group_count as usize)
                .map(|groups| groups.len() as u32)
                .map_err(|_| NucleationError::InvalidArgument)
        }

        pub fn add_armor_stand(
            &mut self,
            x: f64,
            y: f64,
            z: f64,
            yaw: f32,
            armor_material: &DiplomatStr,
        ) -> Result<u32, NucleationError> {
            let material = utf8(armor_material)?;
            let equipment = if material.is_empty() {
                crate::entity::ArmorStandEquipment::default()
            } else {
                crate::entity::ArmorStandEquipment::full_set(material)
            };
            self.0
                .schematic_mut()
                .add_entity(crate::entity::Entity::armor_stand(
                    (x, y, z),
                    yaw,
                    equipment,
                ));
            self.0
                .record_entity_position(x, y, z)
                .map_err(|_| NucleationError::InvalidArgument)
        }

        pub fn animate_camera(&mut self, effect: &AnimationEffect, offset_ms: f32) {
            self.0.animate_camera(effect.0.clip().clone(), offset_ms);
        }

        pub fn frame_count(&self, fps: f64, hold_ms: f32) -> u32 {
            self.0.frames(fps, hold_ms).len() as u32
        }

        /// Render directly to a looping GIF. The renderer, meshes, timeline and
        /// GIF encoder all live in the Rust core; no ffmpeg subprocess is needed.
        #[diplomat::attr(js, disable)]
        #[cfg(all(feature = "rendering", not(target_arch = "wasm32")))]
        pub fn render_gif(
            &self,
            pack_zip: &[u8],
            config: &RenderConfig,
            path: &DiplomatStr,
            fps: f64,
            hold_ms: f32,
        ) -> Result<u32, NucleationError> {
            let path = utf8(path)?;
            let pack = crate::meshing::ResourcePackSource::from_bytes(pack_zip)
                .map_err(|_| NucleationError::Parse)?;
            let meshes = self
                .0
                .mesh_outputs(&pack, &crate::meshing::MeshConfig::default())
                .map_err(|_| NucleationError::Mesh)?;
            let frames = self.0.frames(fps, hold_ms);
            let pixels = crate::rendering::render_animation(&meshes, &frames, &config.0, None)
                .map_err(|_| NucleationError::Render)?;
            crate::rendering::write_animation_gif(
                &pixels,
                config.0.width,
                config.0.height,
                fps,
                path,
            )
            .map_err(|_| NucleationError::Io)?;
            Ok(frames.len() as u32)
        }

        /// Render numbered PNG frames (`prefix0000.png`, ...) for an external
        /// compositor while using the exact same public timeline API.
        #[diplomat::attr(js, disable)]
        #[cfg(all(feature = "rendering", not(target_arch = "wasm32")))]
        pub fn render_frames(
            &self,
            pack_zip: &[u8],
            config: &RenderConfig,
            prefix: &DiplomatStr,
            fps: f64,
            hold_ms: f32,
        ) -> Result<u32, NucleationError> {
            let prefix = utf8(prefix)?;
            let pack = crate::meshing::ResourcePackSource::from_bytes(pack_zip)
                .map_err(|_| NucleationError::Parse)?;
            let meshes = self
                .0
                .mesh_outputs(&pack, &crate::meshing::MeshConfig::default())
                .map_err(|_| NucleationError::Mesh)?;
            let frames = self.0.frames(fps, hold_ms);
            crate::rendering::render_animation_to_files(&meshes, &frames, &config.0, None, prefix)
                .map_err(|_| NucleationError::Render)?;
            Ok(frames.len() as u32)
        }

        #[diplomat::attr(js, disable)]
        pub fn save_to_file(&self, path: &DiplomatStr) -> Result<(), NucleationError> {
            use crate::formats::manager::get_manager;
            let path = utf8(path)?;
            let manager = get_manager();
            let manager = manager.lock().map_err(|_| NucleationError::Lock)?;
            let bytes = manager
                .write_auto_with_settings(path, self.0.schematic(), None, None)
                .map_err(|_| NucleationError::Serialize)?;
            std::fs::write(path, bytes).map_err(|_| NucleationError::Io)
        }

        pub fn group_count(&self) -> u32 {
            self.0.groups().len() as u32
        }

        pub fn duration_ms(&self) -> f32 {
            self.0.duration_ms()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ffi::{AnimationEffect, BuildAnimation};
    use crate::bridge::building::ffi::{Brush, Curve3D, Shape};

    #[test]
    fn bridge_builds_and_groups_a_sampled_curve_in_one_bulk_operation() {
        let curve = Curve3D::from_points(&[0.0, 0.0, 0.0, 9.0, 0.0, 0.0], false).unwrap();
        let shape = Shape::tube_along(&curve, 0.6).unwrap();
        let brush = Brush::solid(b"minecraft:stone").unwrap();
        let mut animation = BuildAnimation::create(b"curve");

        assert_eq!(
            animation.fill_along_parameter(&shape, &brush, 4).unwrap(),
            4
        );
        animation.set_stagger_offset_ms(-1_000.0);
        animation.set_loop_period_ms(1_000.0).unwrap();
        assert_eq!(animation.frame_count(20.0, 5_000.0), 20);
    }

    #[test]
    fn bridge_exposes_transactional_region_operations_and_receipts() {
        let mut animation = BuildAnimation::create(b"operations");
        animation
            .create_region(b"wing", 10, 0, 10, 11, 0, 10)
            .unwrap();
        animation
            .set_block_in_region(b"wing", 10, 0, 10, b"minecraft:oak_stairs[facing=east]")
            .unwrap();
        animation.rotate_region_y(b"wing", -270, 250.0).unwrap();
        assert_eq!(animation.0.operation_count(), 1);
        assert_eq!(animation.0.operation_receipts()[0].duration_ms, 250.0);
        assert!(animation.rotate_region_y(b"wing", 45, 250.0).is_err());
        assert_eq!(animation.0.operation_count(), 1);
        let receipt = &animation.0.operation_receipts()[0];
        let frame = animation
            .0
            .frame_at(receipt.start_ms + receipt.duration_ms * 0.5);
        assert!(!frame.gizmos.is_empty());
    }

    #[test]
    fn bridge_builder_exposes_group_default_override_and_camera_controls() {
        let mut animation = BuildAnimation::create(b"bridge");
        let default = AnimationEffect::drop_and_pop(480.0, 4.5);
        animation.set_default_effect(&default);
        animation.begin_group().unwrap();
        animation.set_block(0, 0, 0, b"minecraft:stone").unwrap();
        animation.set_block(1, 0, 0, b"minecraft:stone").unwrap();
        assert_eq!(animation.end_group().unwrap(), 0);
        let spin = AnimationEffect::spin_in(600.0, 1.0);
        assert_eq!(
            animation
                .with_effect(&spin)
                .set_block(0, 1, 0, b"minecraft:furnace")
                .unwrap(),
            1
        );
        let camera = AnimationEffect::turntable(2_000.0);
        animation.animate_camera(&camera, 0.0);
        assert_eq!(animation.group_count(), 2);
        assert!(animation.duration_ms() >= 2_000.0);
        assert_eq!(animation.frame_count(20.0, 1_000.0), 61);
    }
}
