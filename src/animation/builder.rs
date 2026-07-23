//! Construction-shaped animation builder shared by Rust and generated bindings.
//!
//! Mutations are recorded as animation groups while being applied immediately to
//! the underlying schematic. A normal mutation creates one group; `begin_group`
//! / `end_group` batches many mutations into one target.

use super::{
    BlockEntityDelta, CellDelta, Clip, Easing, EntityDelta, Group, GroupId, Keyframe,
    LatticeAffine, OperationBounds, OperationKind, OperationReceipt, OperationScope,
    OperationTransform, Property, Repeat, Target, Timeline, Track, TransformAxis,
};
use crate::universal_schematic::UniversalSchematic;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq)]
pub struct AnimationEffect {
    clip: Clip,
}

impl AnimationEffect {
    pub fn new(duration_ms: f32) -> Self {
        Self {
            clip: Clip::new(duration_ms),
        }
    }

    pub fn from_clip(clip: Clip) -> Self {
        Self { clip }
    }

    pub fn tween(mut self, property: Property, from: f32, to: f32, easing: Easing) -> Self {
        self.clip
            .tracks
            .push(Track::tween(property, from, to, easing));
        self
    }

    pub fn keyframe(mut self, property: Property, at: f32, value: f32, easing: Easing) -> Self {
        if let Some(track) = self.clip.tracks.iter_mut().find(|t| t.property == property) {
            track.keys.push(Keyframe::eased(at, value, easing));
            track.keys.sort_by(|a, b| {
                a.at.partial_cmp(&b.at)
                    .unwrap_or(core::cmp::Ordering::Equal)
            });
        } else {
            self.clip.tracks.push(Track::new(
                property,
                vec![Keyframe::eased(at, value, easing)],
            ));
        }
        self
    }

    pub fn repeat_forever(mut self) -> Self {
        self.clip.repeat = Repeat::Forever;
        self
    }

    pub fn clip(&self) -> &Clip {
        &self.clip
    }
}

impl From<Clip> for AnimationEffect {
    fn from(value: Clip) -> Self {
        Self::from_clip(value)
    }
}

#[derive(Debug, Clone)]
struct RecordedStep {
    blocks: Vec<(i32, i32, i32)>,
    region: String,
    mesh_region: Option<String>,
    effect: Option<Clip>,
    key: f32,
    mesh_source: UniversalSchematic,
    visible_from_ms: f32,
    visible_until_ms: Option<f32>,
}

#[derive(Debug, Clone)]
struct OpenGroup {
    blocks: Vec<(i32, i32, i32)>,
    region: Option<String>,
    effect: Option<Clip>,
    key: f32,
}

#[derive(Debug, Clone)]
struct RecordedOperation {
    targets: Vec<GroupId>,
    transform: OperationTransform,
    start_ms: f32,
    duration_ms: f32,
    appears: bool,
    receipt: OperationReceipt,
}

/// A schematic plus its recorded construction timeline.
#[derive(Debug, Clone)]
pub struct BuildAnimation {
    schematic: UniversalSchematic,
    steps: Vec<RecordedStep>,
    open_group: Option<OpenGroup>,
    default_effect: Clip,
    pending_effect: Option<Clip>,
    step_ms: f32,
    stagger_total_ms: Option<f32>,
    stagger_offset_ms: f32,
    loop_period_ms: Option<f32>,
    camera: Vec<(Clip, f32)>,
    operations: Vec<RecordedOperation>,
    operation_gizmos: bool,
}

impl BuildAnimation {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            schematic: UniversalSchematic::new(name.into()),
            steps: Vec::new(),
            open_group: None,
            default_effect: super::presets::drop_and_pop(480.0, 4.5),
            pending_effect: None,
            step_ms: 600.0,
            stagger_total_ms: None,
            stagger_offset_ms: 0.0,
            loop_period_ms: None,
            camera: Vec::new(),
            operations: Vec::new(),
            operation_gizmos: true,
        }
    }

    pub fn schematic(&self) -> &UniversalSchematic {
        &self.schematic
    }

    pub fn schematic_mut(&mut self) -> &mut UniversalSchematic {
        &mut self.schematic
    }

    pub fn set_default_effect(&mut self, effect: impl Into<AnimationEffect>) {
        self.default_effect = effect.into().clip;
    }

    /// Apply an effect to exactly the next recorded operation or group.
    pub fn with_effect(&mut self, effect: impl Into<AnimationEffect>) -> &mut Self {
        self.pending_effect = Some(effect.into().clip);
        self
    }

    pub fn set_step_ms(&mut self, step_ms: f32) {
        self.step_ms = step_ms.max(0.0);
    }

    pub fn step_ms(&self) -> f32 {
        self.step_ms
    }

    pub fn set_stagger_total_ms(&mut self, stagger_total_ms: Option<f32>) {
        self.stagger_total_ms = stagger_total_ms.map(|v| v.max(0.0));
    }

    pub fn set_stagger_offset_ms(&mut self, offset_ms: f32) {
        self.stagger_offset_ms = if offset_ms.is_finite() {
            offset_ms
        } else {
            0.0
        };
    }

    pub fn set_loop_period_ms(&mut self, period_ms: Option<f32>) {
        self.loop_period_ms = period_ms.filter(|period| period.is_finite() && *period > 0.0);
    }

    pub fn begin_group(&mut self, effect: Option<Clip>) -> Result<(), String> {
        let key = self.steps.len() as f32;
        self.begin_group_with_key(effect, key)
    }

    pub fn begin_group_with_key(&mut self, effect: Option<Clip>, key: f32) -> Result<(), String> {
        let pending_effect = self.pending_effect.take();
        let effect = effect.or(pending_effect);
        if self.open_group.is_some() {
            return Err("animation groups cannot be nested".into());
        }
        self.open_group = Some(OpenGroup {
            blocks: Vec::new(),
            region: None,
            effect,
            key,
        });
        Ok(())
    }

    pub fn end_group(&mut self) -> Result<GroupId, String> {
        let mut group = self
            .open_group
            .take()
            .ok_or_else(|| "no animation group is open".to_string())?;
        group.blocks.sort_unstable();
        group.blocks.dedup();
        if group.blocks.is_empty() {
            return Err("an animation group cannot be empty".into());
        }
        let id = self.steps.len() as GroupId;
        self.steps.push(RecordedStep {
            blocks: group.blocks,
            region: group.region.clone().unwrap_or_else(|| "Main".to_string()),
            mesh_region: Some(group.region.unwrap_or_else(|| "Main".to_string())),
            effect: group.effect,
            key: group.key,
            mesh_source: self.schematic.clone(),
            visible_from_ms: 0.0,
            visible_until_ms: None,
        });
        Ok(id)
    }

    pub fn set_block(&mut self, x: i32, y: i32, z: i32, block: &str) -> Result<GroupId, String> {
        let effect = self.pending_effect.take();
        self.set_block_inner("Main", x, y, z, block, effect)
    }

    pub fn set_block_in_region(
        &mut self,
        region: &str,
        x: i32,
        y: i32,
        z: i32,
        block: &str,
    ) -> Result<GroupId, String> {
        let effect = self.pending_effect.take();
        self.set_block_inner(region, x, y, z, block, effect)
    }

    /// Fill a parametric shape and partition its voxels into ordered animation
    /// groups using the shape's normalised `parameter_at` value.
    pub fn fill_along_parameter(
        &mut self,
        shape: &crate::building::ShapeEnum,
        brush: &crate::building::BrushEnum,
        group_count: usize,
    ) -> Result<Vec<GroupId>, String> {
        use crate::building::Shape;
        use std::collections::BTreeMap;

        let effect = self.pending_effect.take();
        if self.open_group.is_some() {
            return Err("finish the open animation group before filling a shape".into());
        }
        if group_count == 0 {
            return Err("parametric fill requires at least one animation group".into());
        }

        let mut placements = BTreeMap::new();
        shape.for_each_point(|x, y, z| {
            let Some(parameter) = shape.parameter_at(x, y, z) else {
                return;
            };
            let normal = shape.normal_at(x, y, z);
            if let Some(block) = brush.get_block_with_parameter(x, y, z, normal, Some(parameter)) {
                let group = ((parameter.clamp(0.0, 1.0) * group_count as f64).floor() as usize)
                    .min(group_count - 1);
                placements.insert((x, y, z), (group, block));
            }
        });
        if placements.is_empty() {
            return Err("parametric shape produced no blocks".into());
        }
        let (min_x, min_y, min_z, max_x, max_y, max_z) = shape.bounds();
        self.schematic
            .ensure_bounds((min_x, min_y, min_z), (max_x, max_y, max_z));
        let replaced: std::collections::HashSet<_> = placements.keys().copied().collect();
        for step in &mut self.steps {
            step.blocks.retain(|position| !replaced.contains(position));
        }

        let mut buckets = vec![Vec::new(); group_count];
        for (position, (group, block)) in placements {
            self.schematic
                .set_block(position.0, position.1, position.2, &block);
            buckets[group].push(position);
        }

        let mut ids = Vec::new();
        for (index, blocks) in buckets.into_iter().enumerate() {
            if blocks.is_empty() {
                continue;
            }
            let id = self.steps.len() as GroupId;
            self.steps.push(RecordedStep {
                blocks,
                region: "Main".to_string(),
                mesh_region: Some("Main".to_string()),
                effect: effect.clone(),
                key: index as f32,
                mesh_source: self.schematic.clone(),
                visible_from_ms: 0.0,
                visible_until_ms: None,
            });
            ids.push(id);
        }
        Ok(ids)
    }

    fn set_block_inner(
        &mut self,
        region: &str,
        x: i32,
        y: i32,
        z: i32,
        block: &str,
        effect: Option<Clip>,
    ) -> Result<GroupId, String> {
        if self.open_group.is_some() && effect.is_some() {
            return Err("set the effect on the open group, not a mutation inside it".into());
        }
        if !self
            .schematic
            .try_set_block_in_region_str(region, x, y, z, block)?
        {
            return Err(format!("Region '{region}' not found"));
        }
        let pos = (x, y, z);
        for step in &mut self.steps {
            if step.region == region {
                step.blocks.retain(|&recorded| recorded != pos);
            }
        }
        if let Some(group) = self.open_group.as_mut() {
            match group.region.as_deref() {
                Some(existing) if existing != region => {
                    return Err("an animation group cannot mix schematic regions".into())
                }
                None => group.region = Some(region.to_string()),
                _ => {}
            }
            group.blocks.retain(|&recorded| recorded != pos);
            group.blocks.push(pos);
            return Ok(self.steps.len() as GroupId);
        }
        let id = self.steps.len() as GroupId;
        self.steps.push(RecordedStep {
            blocks: vec![pos],
            region: region.to_string(),
            mesh_region: Some(region.to_string()),
            effect,
            key: id as f32,
            mesh_source: self.schematic.clone(),
            visible_from_ms: 0.0,
            visible_until_ms: None,
        });
        Ok(id)
    }

    /// Record an entity's floored block position as an animation target after the
    /// caller adds the entity to `schematic_mut()`.
    pub fn record_entity_position(&mut self, x: f64, y: f64, z: f64) -> Result<GroupId, String> {
        let effect = self.pending_effect.take();
        let pos = (x.floor() as i32, y.floor() as i32, z.floor() as i32);
        if let Some(group) = self.open_group.as_mut() {
            if effect.is_some() {
                return Err("set the effect on the open group, not an entity inside it".into());
            }
            group.blocks.push(pos);
            return Ok(self.steps.len() as GroupId);
        }
        let id = self.steps.len() as GroupId;
        self.steps.push(RecordedStep {
            blocks: vec![pos],
            region: "Main".to_string(),
            mesh_region: Some("Main".to_string()),
            effect,
            key: id as f32,
            mesh_source: self.schematic.clone(),
            visible_from_ms: 0.0,
            visible_until_ms: None,
        });
        Ok(id)
    }

    pub fn operation_count(&self) -> usize {
        self.operations.len()
    }

    pub fn operation_receipts(&self) -> Vec<OperationReceipt> {
        self.operations
            .iter()
            .map(|operation| operation.receipt.clone())
            .collect()
    }

    pub fn operations_json(&self) -> Result<String, String> {
        serde_json::to_string(&self.operation_receipts()).map_err(|error| error.to_string())
    }

    fn next_operation_start_ms(&self) -> f32 {
        let construction = self.timeline().duration_ms();
        self.operations
            .last()
            .map(|op| op.start_ms + op.duration_ms)
            .unwrap_or(construction)
            .max(construction)
    }

    fn validate_operation(&self, duration_ms: f32) -> Result<(), String> {
        if self.open_group.is_some() {
            return Err("finish the open animation group before recording an operation".into());
        }
        if !duration_ms.is_finite() || duration_ms < 0.0 {
            return Err("operation duration must be finite and non-negative".into());
        }
        Ok(())
    }

    fn rotate_position(
        axis: TransformAxis,
        p: (i32, i32, i32),
        bounds: &crate::bounding_box::BoundingBox,
        degrees: i32,
    ) -> Result<(i32, i32, i32), String> {
        let min = [
            i64::from(bounds.min.0),
            i64::from(bounds.min.1),
            i64::from(bounds.min.2),
        ];
        let max = [
            i64::from(bounds.max.0),
            i64::from(bounds.max.1),
            i64::from(bounds.max.2),
        ];
        let mut size = [0, 1, 2].map(|i| max[i] - min[i] + 1);
        let (a, b) = match axis {
            TransformAxis::X => (1, 2),
            TransformAxis::Y => (0, 2),
            TransformAxis::Z => (0, 1),
        };
        let mut values = [i64::from(p.0), i64::from(p.1), i64::from(p.2)];
        for _ in 0..degrees.rem_euclid(360) / 90 {
            let local_a = values[a] - min[a];
            let local_b = values[b] - min[b];
            values[a] = min[a] + size[b] - 1 - local_b;
            values[b] = min[b] + local_a;
            size.swap(a, b);
        }
        Ok((
            i32::try_from(values[0]).map_err(|_| "Transform exceeds the i32 coordinate range")?,
            i32::try_from(values[1]).map_err(|_| "Transform exceeds the i32 coordinate range")?,
            i32::try_from(values[2]).map_err(|_| "Transform exceeds the i32 coordinate range")?,
        ))
    }

    fn bounds_pivot(bounds: &crate::bounding_box::BoundingBox) -> [f32; 3] {
        [
            (bounds.min.0 as f64 + bounds.max.0 as f64) as f32 * 0.5,
            (bounds.min.1 as f64 + bounds.max.1 as f64) as f32 * 0.5,
            (bounds.min.2 as f64 + bounds.max.2 as f64) as f32 * 0.5,
        ]
    }

    fn inverse_rotation_degrees(axis: TransformAxis, degrees: i32) -> f32 {
        match axis {
            TransformAxis::Y => degrees as f32,
            TransformAxis::X | TransformAxis::Z => -(degrees as f32),
        }
    }

    fn rotate_region_axis(
        &mut self,
        region: &str,
        axis: TransformAxis,
        degrees: i32,
        duration_ms: f32,
    ) -> Result<(), String> {
        self.validate_operation(duration_ms)?;
        if degrees % 90 != 0 {
            return Err(format!(
                "Rotation must be a multiple of 90 degrees, got {degrees}"
            ));
        }
        let normalized = degrees.rem_euclid(360);
        let bounds = self
            .schematic
            .get_region_bounding_box(region)
            .ok_or_else(|| format!("Region '{region}' not found"))?;
        let mut transformed = self.schematic.clone();
        match axis {
            TransformAxis::X => transformed.rotate_region_x(region, normalized)?,
            TransformAxis::Y => transformed.rotate_region_y(region, normalized)?,
            TransformAxis::Z => transformed.rotate_region_z(region, normalized)?,
        }
        let final_bounds = transformed
            .get_region_bounding_box(region)
            .ok_or_else(|| format!("Region '{region}' not found after rotation"))?;
        let targets = self.target_ids(Some(region));
        let before_positions = if normalized != 0 {
            self.validate_transform_coverage(&targets, Some(region))?
        } else {
            BTreeMap::new()
        };
        let operation_scope = if region == "Main" {
            OperationScope::DefaultRegion
        } else {
            OperationScope::Region(region.to_string())
        };
        let operation_kind = OperationKind::Rotate {
            axis,
            quarter_turns: (normalized / 90) as u8,
        };
        let mut next_steps = self.steps.clone();
        for step in next_steps
            .iter_mut()
            .filter(|step| step.region == region && step.visible_until_ms.is_none())
        {
            for p in &mut step.blocks {
                *p = Self::rotate_position(axis, *p, &bounds, normalized)?;
            }
            step.blocks.sort_unstable();
            step.blocks.dedup();
        }
        let before_schematic = self.schematic.clone();
        self.steps = next_steps;
        self.schematic = transformed;
        if normalized != 0 {
            self.push_operation(
                targets,
                operation_scope,
                operation_kind,
                &before_schematic,
                &before_positions,
                Some(bounds.clone()),
                Some(final_bounds.clone()),
                OperationTransform::Rotate {
                    axis,
                    inverse_degrees: Self::inverse_rotation_degrees(axis, normalized),
                    pivot: Self::bounds_pivot(&bounds),
                    final_pivot: Self::bounds_pivot(&final_bounds),
                },
                duration_ms,
            )?;
        }
        Ok(())
    }

    pub fn rotate_region_x(
        &mut self,
        region: &str,
        degrees: i32,
        duration_ms: f32,
    ) -> Result<(), String> {
        self.rotate_region_axis(region, TransformAxis::X, degrees, duration_ms)
    }

    pub fn rotate_region_y(
        &mut self,
        region: &str,
        degrees: i32,
        duration_ms: f32,
    ) -> Result<(), String> {
        self.rotate_region_axis(region, TransformAxis::Y, degrees, duration_ms)
    }

    pub fn rotate_region_z(
        &mut self,
        region: &str,
        degrees: i32,
        duration_ms: f32,
    ) -> Result<(), String> {
        self.rotate_region_axis(region, TransformAxis::Z, degrees, duration_ms)
    }

    pub fn rotate_x(&mut self, degrees: i32, duration_ms: f32) -> Result<(), String> {
        self.rotate_region_x("Main", degrees, duration_ms)
    }

    pub fn rotate_y(&mut self, degrees: i32, duration_ms: f32) -> Result<(), String> {
        self.rotate_region_y("Main", degrees, duration_ms)
    }

    pub fn rotate_z(&mut self, degrees: i32, duration_ms: f32) -> Result<(), String> {
        self.rotate_region_z("Main", degrees, duration_ms)
    }

    fn scoped_entities(
        schematic: &UniversalSchematic,
        scope: &OperationScope,
    ) -> Vec<crate::entity::Entity> {
        match scope {
            OperationScope::DefaultRegion => schematic
                .get_region("Main")
                .map(|region| region.entities.clone())
                .unwrap_or_default(),
            OperationScope::Region(name) => schematic
                .get_region(name)
                .map(|region| region.entities.clone())
                .unwrap_or_default(),
            OperationScope::Schematic => schematic.get_entities_as_list(),
            OperationScope::StampRegion(_) | OperationScope::StampBox => Vec::new(),
        }
    }

    fn exact_final_position(
        kind: OperationKind,
        bounds: Option<&crate::bounding_box::BoundingBox>,
        position: (i32, i32, i32),
    ) -> Result<(i32, i32, i32), String> {
        match kind {
            OperationKind::Translate { delta } => Ok((
                position
                    .0
                    .checked_add(delta[0])
                    .ok_or("operation X coordinate overflow")?,
                position
                    .1
                    .checked_add(delta[1])
                    .ok_or("operation Y coordinate overflow")?,
                position
                    .2
                    .checked_add(delta[2])
                    .ok_or("operation Z coordinate overflow")?,
            )),
            OperationKind::Rotate {
                axis,
                quarter_turns,
            } => Self::rotate_position(
                axis,
                position,
                bounds.ok_or("rotation requires source bounds")?,
                i32::from(quarter_turns) * 90,
            ),
            OperationKind::Flip { axis } => {
                let bounds = bounds.ok_or("flip requires source bounds")?;
                let min = [bounds.min.0, bounds.min.1, bounds.min.2];
                let max = [bounds.max.0, bounds.max.1, bounds.max.2];
                let mut out = [position.0, position.1, position.2];
                let index = match axis {
                    TransformAxis::X => 0,
                    TransformAxis::Y => 1,
                    TransformAxis::Z => 2,
                };
                out[index] = i32::try_from(
                    i64::from(min[index]) + i64::from(max[index]) - i64::from(out[index]),
                )
                .map_err(|_| "flip coordinate overflow")?;
                Ok((out[0], out[1], out[2]))
            }
            OperationKind::Stamp => Err("stamp does not use transform snapshots".into()),
        }
    }

    fn push_operation(
        &mut self,
        targets: Vec<GroupId>,
        scope: OperationScope,
        kind: OperationKind,
        before_schematic: &UniversalSchematic,
        before_positions: &BTreeMap<GroupId, Vec<(i32, i32, i32)>>,
        before_bounds: Option<crate::bounding_box::BoundingBox>,
        final_bounds: Option<crate::bounding_box::BoundingBox>,
        transform: OperationTransform,
        duration_ms: f32,
    ) -> Result<(), String> {
        let start_ms = self.next_operation_start_ms();
        let end_ms = start_ms + duration_ms;
        let mut final_steps = Vec::with_capacity(targets.len());
        let mut cells = Vec::new();
        for id in &targets {
            let Some(current) = self.steps.get(*id as usize).cloned() else {
                continue;
            };
            let final_blocks = current.blocks.clone();
            let mut before = current;
            before.blocks = before_positions
                .get(id)
                .cloned()
                .ok_or_else(|| format!("missing exact source snapshot for group {id}"))?;
            before.visible_until_ms = Some(end_ms);
            self.steps[*id as usize] = before.clone();

            for &before_position in &before.blocks {
                let final_position =
                    Self::exact_final_position(kind, before_bounds.as_ref(), before_position)?;
                if !final_blocks.contains(&final_position) {
                    return Err(format!(
                        "exact transform result {final_position:?} missing from group {id}"
                    ));
                }
                cells.push(CellDelta {
                    region: before.region.clone(),
                    before_position: [before_position.0, before_position.1, before_position.2],
                    final_position: [final_position.0, final_position.1, final_position.2],
                    before_block: before
                        .mesh_source
                        .get_block_string_in_region(
                            &before.region,
                            before_position.0,
                            before_position.1,
                            before_position.2,
                        )
                        .unwrap_or_else(|| "minecraft:air".to_string()),
                    final_block: self
                        .schematic
                        .get_block_string_in_region(
                            &before.region,
                            final_position.0,
                            final_position.1,
                            final_position.2,
                        )
                        .unwrap_or_else(|| "minecraft:air".to_string()),
                });
            }

            final_steps.push(RecordedStep {
                blocks: final_blocks,
                mesh_region: Some(before.region.clone()),
                region: before.region,
                effect: Some(Clip::new(0.0)),
                key: self.steps.len() as f32 + final_steps.len() as f32,
                mesh_source: self.schematic.clone(),
                visible_from_ms: end_ms,
                visible_until_ms: None,
            });
        }
        self.steps.extend(final_steps);
        let linear_matrix = transform.matrix_at(1.0);
        let origin = super::operation::transform_point(linear_matrix, [0.0; 3]);
        let mut linear = [[0i8; 3]; 3];
        for column in 0..3 {
            let mut basis = [0.0; 3];
            basis[column] = 1.0;
            let mapped = super::operation::transform_point(linear_matrix, basis);
            for row in 0..3 {
                linear[row][column] = (mapped[row] - origin[row]).round() as i8;
            }
        }
        let offset = cells.first().map_or_else(
            || origin.map(|value| value.round() as i64),
            |cell| {
                let before = cell.before_position.map(i64::from);
                let final_position = cell.final_position.map(i64::from);
                [0, 1, 2].map(|row| {
                    final_position[row]
                        - (0..3)
                            .map(|column| linear[row][column] as i64 * before[column])
                            .sum::<i64>()
                })
            },
        );
        let before_bounds = before_bounds.map(OperationBounds::from);
        let final_bounds = final_bounds.map(OperationBounds::from);
        let block_entities = cells
            .iter()
            .filter_map(|cell| {
                let source = before_schematic.get_block_entity_in_region(
                    &cell.region,
                    cell.before_position[0],
                    cell.before_position[1],
                    cell.before_position[2],
                );
                let replaced_destination = before_schematic.get_block_entity_in_region(
                    &cell.region,
                    cell.final_position[0],
                    cell.final_position[1],
                    cell.final_position[2],
                );
                let final_state = self.schematic.get_block_entity_in_region(
                    &cell.region,
                    cell.final_position[0],
                    cell.final_position[1],
                    cell.final_position[2],
                );
                (source.is_some() || replaced_destination.is_some() || final_state.is_some())
                    .then_some(BlockEntityDelta {
                        source_position: cell.before_position,
                        final_position: cell.final_position,
                        source,
                        replaced_destination,
                        final_state,
                    })
            })
            .collect();
        let entities = Self::scoped_entities(before_schematic, &scope)
            .into_iter()
            .zip(Self::scoped_entities(&self.schematic, &scope))
            .filter_map(|(before, final_state)| {
                (before != final_state).then_some(EntityDelta {
                    before,
                    final_state,
                })
            })
            .collect();
        let receipt = OperationReceipt {
            start_ms,
            duration_ms,
            id: self.operations.len() as u32,
            scope,
            kind,
            before_bounds,
            final_bounds,
            pivot2: before_bounds.map(OperationBounds::pivot2),
            final_pivot2: final_bounds.map(OperationBounds::pivot2),
            affine: Some(LatticeAffine { linear, offset }),
            cells,
            excluded_cells: Vec::new(),
            block_entities,
            entities,
        };
        self.operations.push(RecordedOperation {
            targets,
            transform,
            start_ms,
            duration_ms,
            appears: false,
            receipt,
        });
        Ok(())
    }

    fn target_ids(&self, region: Option<&str>) -> Vec<GroupId> {
        self.steps
            .iter()
            .enumerate()
            .filter_map(|(id, step)| {
                (step.visible_until_ms.is_none() && region.is_none_or(|name| step.region == name))
                    .then_some(id as GroupId)
            })
            .collect()
    }

    fn validate_transform_coverage(
        &self,
        targets: &[GroupId],
        region: Option<&str>,
    ) -> Result<BTreeMap<GroupId, Vec<(i32, i32, i32)>>, String> {
        let snapshots: BTreeMap<_, _> = targets
            .iter()
            .filter_map(|id| {
                self.steps
                    .get(*id as usize)
                    .map(|step| (*id, step.blocks.clone()))
            })
            .collect();
        let tracked: BTreeSet<_> = targets
            .iter()
            .filter_map(|id| self.steps.get(*id as usize))
            .flat_map(|step| {
                step.blocks
                    .iter()
                    .copied()
                    .map(move |position| (step.region.clone(), position))
            })
            .collect();
        let region_names = region.map_or_else(
            || self.schematic.get_region_names(),
            |name| vec![name.to_string()],
        );
        let mut actual = BTreeSet::new();
        for name in region_names {
            let Some(source_region) = self.schematic.get_region(&name) else {
                continue;
            };
            for index in 0..source_region.blocks.len() {
                let position = source_region.index_to_coords(index);
                if source_region
                    .get_block(position.0, position.1, position.2)
                    .is_some_and(|block| !crate::fingerprint::is_air(block.get_name()))
                {
                    actual.insert((name.clone(), position));
                }
            }
        }
        if tracked != actual {
            return Err(format!(
                "operation requires exact active-group coverage (tracked {}, schematic {})",
                tracked.len(),
                actual.len()
            ));
        }
        Ok(snapshots)
    }

    pub fn translate(&mut self, dx: i32, dy: i32, dz: i32, duration_ms: f32) -> Result<(), String> {
        self.translate_region("Main", dx, dy, dz, duration_ms)
    }

    fn install_stamp_draws(
        &mut self,
        source: &UniversalSchematic,
        source_region: Option<&str>,
        source_positions: Vec<(i32, i32, i32)>,
        final_positions: Vec<(i32, i32, i32)>,
        start_ms: f32,
        duration_ms: f32,
    ) -> GroupId {
        let end_ms = start_ms + duration_ms;
        let mut retired = Vec::new();
        for step in self
            .steps
            .iter_mut()
            .filter(|step| step.region == "Main" && step.visible_until_ms.is_none())
        {
            let mut old_draw = step.clone();
            old_draw.blocks = step
                .blocks
                .iter()
                .copied()
                .filter(|position| final_positions.contains(position))
                .collect();
            step.blocks
                .retain(|position| !final_positions.contains(position));
            if !old_draw.blocks.is_empty() {
                old_draw.effect = Some(Clip::new(0.0));
                old_draw.visible_until_ms = Some(end_ms);
                retired.push(old_draw);
            }
        }
        self.steps.extend(retired);

        let moving_id = self.steps.len() as GroupId;
        self.steps.push(RecordedStep {
            blocks: source_positions,
            region: "Main".to_string(),
            mesh_region: source_region.map(str::to_string),
            effect: Some(Clip::new(0.0)),
            key: moving_id as f32,
            mesh_source: source.clone(),
            visible_from_ms: start_ms,
            visible_until_ms: Some(end_ms),
        });
        let final_id = self.steps.len() as GroupId;
        self.steps.push(RecordedStep {
            blocks: final_positions,
            region: "Main".to_string(),
            mesh_region: Some("Main".to_string()),
            effect: Some(Clip::new(0.0)),
            key: final_id as f32,
            mesh_source: self.schematic.clone(),
            visible_from_ms: end_ms,
            visible_until_ms: None,
        });
        moving_id
    }

    pub fn translate_region(
        &mut self,
        region: &str,
        dx: i32,
        dy: i32,
        dz: i32,
        duration_ms: f32,
    ) -> Result<(), String> {
        self.validate_operation(duration_ms)?;
        let bounds = self
            .schematic
            .get_region_bounding_box(region)
            .ok_or_else(|| format!("Region '{region}' not found"))?;
        let mut transformed = self.schematic.clone();
        transformed.translate_region(region, dx, dy, dz)?;
        let final_bounds = transformed
            .get_region_bounding_box(region)
            .ok_or_else(|| format!("Region '{region}' not found after translation"))?;
        let targets = self.target_ids(Some(region));
        let before_positions = if (dx, dy, dz) != (0, 0, 0) {
            self.validate_transform_coverage(&targets, Some(region))?
        } else {
            BTreeMap::new()
        };
        let operation_scope = if region == "Main" {
            OperationScope::DefaultRegion
        } else {
            OperationScope::Region(region.to_string())
        };
        let operation_kind = OperationKind::Translate {
            delta: [dx, dy, dz],
        };
        let mut next_steps = self.steps.clone();
        for step in next_steps
            .iter_mut()
            .filter(|step| step.region == region && step.visible_until_ms.is_none())
        {
            for p in &mut step.blocks {
                p.0 =
                    p.0.checked_add(dx)
                        .ok_or("Transform exceeds the i32 coordinate range")?;
                p.1 =
                    p.1.checked_add(dy)
                        .ok_or("Transform exceeds the i32 coordinate range")?;
                p.2 =
                    p.2.checked_add(dz)
                        .ok_or("Transform exceeds the i32 coordinate range")?;
            }
        }
        let before_schematic = self.schematic.clone();
        self.steps = next_steps;
        self.schematic = transformed;
        if (dx, dy, dz) != (0, 0, 0) {
            self.push_operation(
                targets,
                operation_scope,
                operation_kind,
                &before_schematic,
                &before_positions,
                Some(bounds.clone()),
                Some(final_bounds.clone()),
                OperationTransform::Translate {
                    inverse_delta: [-(dx as f32), -(dy as f32), -(dz as f32)],
                },
                duration_ms,
            )?;
        }
        Ok(())
    }

    pub fn translate_all(
        &mut self,
        dx: i32,
        dy: i32,
        dz: i32,
        duration_ms: f32,
    ) -> Result<(), String> {
        self.validate_operation(duration_ms)?;
        let bounds = self
            .schematic
            .get_schematic_bounding_box()
            .ok_or("Schematic has no bounds")?;
        let mut transformed = self.schematic.clone();
        transformed.translate_schematic(dx, dy, dz)?;
        let final_bounds = transformed
            .get_schematic_bounding_box()
            .ok_or("Schematic has no bounds after translation")?;
        let targets = self.target_ids(None);
        let before_positions = if (dx, dy, dz) != (0, 0, 0) {
            self.validate_transform_coverage(&targets, None)?
        } else {
            BTreeMap::new()
        };
        let operation_scope = OperationScope::Schematic;
        let operation_kind = OperationKind::Translate {
            delta: [dx, dy, dz],
        };
        let mut next_steps = self.steps.clone();
        for step in next_steps
            .iter_mut()
            .filter(|step| step.visible_until_ms.is_none())
        {
            for p in &mut step.blocks {
                p.0 =
                    p.0.checked_add(dx)
                        .ok_or("Transform exceeds the i32 coordinate range")?;
                p.1 =
                    p.1.checked_add(dy)
                        .ok_or("Transform exceeds the i32 coordinate range")?;
                p.2 =
                    p.2.checked_add(dz)
                        .ok_or("Transform exceeds the i32 coordinate range")?;
            }
        }
        let before_schematic = self.schematic.clone();
        self.steps = next_steps;
        self.schematic = transformed;
        if (dx, dy, dz) != (0, 0, 0) {
            self.push_operation(
                targets,
                operation_scope,
                operation_kind,
                &before_schematic,
                &before_positions,
                Some(bounds.clone()),
                Some(final_bounds.clone()),
                OperationTransform::Translate {
                    inverse_delta: [-(dx as f32), -(dy as f32), -(dz as f32)],
                },
                duration_ms,
            )?;
        }
        Ok(())
    }

    fn rotate_all_axis(
        &mut self,
        axis: TransformAxis,
        degrees: i32,
        duration_ms: f32,
    ) -> Result<(), String> {
        self.validate_operation(duration_ms)?;
        if degrees % 90 != 0 {
            return Err(format!(
                "Rotation must be a multiple of 90 degrees, got {degrees}"
            ));
        }
        let normalized = degrees.rem_euclid(360);
        let bounds = self
            .schematic
            .get_schematic_bounding_box()
            .ok_or("Schematic has no bounds")?;
        let mut transformed = self.schematic.clone();
        match axis {
            TransformAxis::X => transformed.rotate_schematic_x(normalized)?,
            TransformAxis::Y => transformed.rotate_schematic_y(normalized)?,
            TransformAxis::Z => transformed.rotate_schematic_z(normalized)?,
        }
        let final_bounds = transformed
            .get_schematic_bounding_box()
            .ok_or("Schematic has no bounds after rotation")?;
        let targets = self.target_ids(None);
        let before_positions = if normalized != 0 {
            self.validate_transform_coverage(&targets, None)?
        } else {
            BTreeMap::new()
        };
        let operation_scope = OperationScope::Schematic;
        let operation_kind = OperationKind::Rotate {
            axis,
            quarter_turns: (normalized / 90) as u8,
        };
        let mut next_steps = self.steps.clone();
        for step in next_steps
            .iter_mut()
            .filter(|step| step.visible_until_ms.is_none())
        {
            for p in &mut step.blocks {
                *p = Self::rotate_position(axis, *p, &bounds, normalized)?;
            }
            step.blocks.sort_unstable();
        }
        let before_schematic = self.schematic.clone();
        self.steps = next_steps;
        self.schematic = transformed;
        if normalized != 0 {
            self.push_operation(
                targets,
                operation_scope,
                operation_kind,
                &before_schematic,
                &before_positions,
                Some(bounds.clone()),
                Some(final_bounds.clone()),
                OperationTransform::Rotate {
                    axis,
                    inverse_degrees: Self::inverse_rotation_degrees(axis, normalized),
                    pivot: Self::bounds_pivot(&bounds),
                    final_pivot: Self::bounds_pivot(&final_bounds),
                },
                duration_ms,
            )?;
        }
        Ok(())
    }

    pub fn rotate_all_x(&mut self, degrees: i32, duration_ms: f32) -> Result<(), String> {
        self.rotate_all_axis(TransformAxis::X, degrees, duration_ms)
    }
    pub fn rotate_all_y(&mut self, degrees: i32, duration_ms: f32) -> Result<(), String> {
        self.rotate_all_axis(TransformAxis::Y, degrees, duration_ms)
    }
    pub fn rotate_all_z(&mut self, degrees: i32, duration_ms: f32) -> Result<(), String> {
        self.rotate_all_axis(TransformAxis::Z, degrees, duration_ms)
    }

    fn flip_axis(
        &mut self,
        region: Option<&str>,
        axis: TransformAxis,
        duration_ms: f32,
    ) -> Result<(), String> {
        self.validate_operation(duration_ms)?;
        let bounds = match region {
            Some(name) => self
                .schematic
                .get_region_bounding_box(name)
                .ok_or_else(|| format!("Region '{name}' not found"))?,
            None => self
                .schematic
                .get_schematic_bounding_box()
                .ok_or("Schematic has no bounds")?,
        };
        let mut transformed = self.schematic.clone();
        match (region, axis) {
            (Some(name), TransformAxis::X) => transformed.flip_region_x(name)?,
            (Some(name), TransformAxis::Y) => transformed.flip_region_y(name)?,
            (Some(name), TransformAxis::Z) => transformed.flip_region_z(name)?,
            (None, TransformAxis::X) => transformed.flip_schematic_x()?,
            (None, TransformAxis::Y) => transformed.flip_schematic_y()?,
            (None, TransformAxis::Z) => transformed.flip_schematic_z()?,
        }
        let final_bounds = match region {
            Some(name) => transformed
                .get_region_bounding_box(name)
                .ok_or_else(|| format!("Region '{name}' not found after flip"))?,
            None => transformed
                .get_schematic_bounding_box()
                .ok_or("Schematic has no bounds after flip")?,
        };
        let operation_scope = match region {
            Some("Main") => OperationScope::DefaultRegion,
            Some(name) => OperationScope::Region(name.to_string()),
            None => OperationScope::Schematic,
        };
        let operation_kind = OperationKind::Flip { axis };
        let targets = self.target_ids(region);
        let before_positions = self.validate_transform_coverage(&targets, region)?;
        let min = [bounds.min.0, bounds.min.1, bounds.min.2];
        let max = [bounds.max.0, bounds.max.1, bounds.max.2];
        let index = match axis {
            TransformAxis::X => 0,
            TransformAxis::Y => 1,
            TransformAxis::Z => 2,
        };
        let mut next_steps = self.steps.clone();
        for step in next_steps.iter_mut().filter(|step| {
            step.visible_until_ms.is_none() && region.is_none_or(|name| step.region == name)
        }) {
            for p in &mut step.blocks {
                let mut v = [p.0, p.1, p.2];
                let mirrored = i64::from(min[index]) + i64::from(max[index]) - i64::from(v[index]);
                v[index] = i32::try_from(mirrored)
                    .map_err(|_| "Transform exceeds the i32 coordinate range")?;
                *p = (v[0], v[1], v[2]);
            }
            step.blocks.sort_unstable();
        }
        let before_schematic = self.schematic.clone();
        self.steps = next_steps;
        self.schematic = transformed;
        self.push_operation(
            targets,
            operation_scope,
            operation_kind,
            &before_schematic,
            &before_positions,
            Some(bounds),
            Some(final_bounds),
            OperationTransform::Flip {
                axis,
                plane: (min[index] as f64 + max[index] as f64) as f32 * 0.5,
            },
            duration_ms,
        )?;
        Ok(())
    }

    pub fn flip_x(&mut self, duration_ms: f32) -> Result<(), String> {
        self.flip_axis(Some("Main"), TransformAxis::X, duration_ms)
    }
    pub fn flip_y(&mut self, duration_ms: f32) -> Result<(), String> {
        self.flip_axis(Some("Main"), TransformAxis::Y, duration_ms)
    }
    pub fn flip_z(&mut self, duration_ms: f32) -> Result<(), String> {
        self.flip_axis(Some("Main"), TransformAxis::Z, duration_ms)
    }
    pub fn flip_region_x(&mut self, region: &str, duration_ms: f32) -> Result<(), String> {
        self.flip_axis(Some(region), TransformAxis::X, duration_ms)
    }
    pub fn flip_region_y(&mut self, region: &str, duration_ms: f32) -> Result<(), String> {
        self.flip_axis(Some(region), TransformAxis::Y, duration_ms)
    }
    pub fn flip_region_z(&mut self, region: &str, duration_ms: f32) -> Result<(), String> {
        self.flip_axis(Some(region), TransformAxis::Z, duration_ms)
    }
    pub fn flip_all_x(&mut self, duration_ms: f32) -> Result<(), String> {
        self.flip_axis(None, TransformAxis::X, duration_ms)
    }
    pub fn flip_all_y(&mut self, duration_ms: f32) -> Result<(), String> {
        self.flip_axis(None, TransformAxis::Y, duration_ms)
    }
    pub fn flip_all_z(&mut self, duration_ms: f32) -> Result<(), String> {
        self.flip_axis(None, TransformAxis::Z, duration_ms)
    }

    pub fn stamp_region(
        &mut self,
        source: &UniversalSchematic,
        region: &str,
        target: (i32, i32, i32),
        excluded_blocks: &[String],
        duration_ms: f32,
    ) -> Result<(), String> {
        self.validate_operation(duration_ms)?;
        let excluded: Vec<crate::block_state::BlockState> = excluded_blocks
            .iter()
            .map(|value| crate::block_state::BlockState::from_block_string(value))
            .collect::<Result<_, _>>()?;
        let source_region = source
            .get_region(region)
            .ok_or_else(|| format!("Region '{region}' not found"))?;
        let Some(bounds) = UniversalSchematic::region_stamp_bounds(source_region) else {
            let mut transformed = self.schematic.clone();
            transformed.stamp_region(source, region, target, &excluded)?;
            self.schematic = transformed;
            return Ok(());
        };
        let offset = (
            target.0 as i64 - bounds.min.0 as i64,
            target.1 as i64 - bounds.min.1 as i64,
            target.2 as i64 - bounds.min.2 as i64,
        );
        let mut written = Vec::new();
        let mut mappings = Vec::new();
        let mut excluded_cells = Vec::new();
        for x in bounds.min.0..=bounds.max.0 {
            for y in bounds.min.1..=bounds.max.1 {
                for z in bounds.min.2..=bounds.max.2 {
                    let Some(block) = source_region.get_block(x, y, z) else {
                        continue;
                    };
                    let destination = (
                        i32::try_from(x as i64 + offset.0)
                            .map_err(|_| "Stamp exceeds the i32 coordinate range")?,
                        i32::try_from(y as i64 + offset.1)
                            .map_err(|_| "Stamp exceeds the i32 coordinate range")?,
                        i32::try_from(z as i64 + offset.2)
                            .map_err(|_| "Stamp exceeds the i32 coordinate range")?,
                    );
                    if excluded.contains(block) {
                        excluded_cells.push([destination.0, destination.1, destination.2]);
                        continue;
                    }
                    written.push(destination);
                    mappings.push(((x, y, z), destination, block.to_string()));
                }
            }
        }
        let mut transformed = self.schematic.clone();
        transformed.stamp_region(source, region, target, &excluded)?;
        let block_entities: Vec<BlockEntityDelta> = mappings
            .iter()
            .filter_map(|(source_position, final_position, _)| {
                let source_state = source.get_block_entity_in_region(
                    region,
                    source_position.0,
                    source_position.1,
                    source_position.2,
                );
                let final_pos = crate::block_position::BlockPosition {
                    x: final_position.0,
                    y: final_position.1,
                    z: final_position.2,
                };
                let replaced_destination = self.schematic.get_block_entity(final_pos).cloned();
                let final_state = transformed.get_block_entity(final_pos).cloned();
                (source_state.is_some() || replaced_destination.is_some() || final_state.is_some())
                    .then_some(BlockEntityDelta {
                        source_position: [source_position.0, source_position.1, source_position.2],
                        final_position: [final_position.0, final_position.1, final_position.2],
                        source: source_state,
                        replaced_destination,
                        final_state,
                    })
            })
            .collect();
        let entities: Vec<EntityDelta> = source_region
            .entities
            .iter()
            .cloned()
            .map(|before| {
                let mut final_state = before.clone();
                final_state.position = (
                    before.position.0 + offset.0 as f64,
                    before.position.1 + offset.1 as f64,
                    before.position.2 + offset.2 as f64,
                );
                EntityDelta {
                    before,
                    final_state,
                }
            })
            .collect();
        written.sort_unstable();
        written.dedup();
        let source_positions = mappings.iter().map(|(position, _, _)| *position).collect();
        let start_ms = self.next_operation_start_ms();
        self.schematic = transformed;
        let targets = if written.is_empty() {
            Vec::new()
        } else {
            vec![self.install_stamp_draws(
                source,
                Some(region),
                source_positions,
                written,
                start_ms,
                duration_ms,
            )]
        };
        let before_bounds = OperationBounds::from(bounds.clone());
        let final_bounds = OperationBounds {
            min: [target.0, target.1, target.2],
            max: [
                i32::try_from(bounds.max.0 as i64 + offset.0)
                    .map_err(|_| "Stamp exceeds the i32 coordinate range")?,
                i32::try_from(bounds.max.1 as i64 + offset.1)
                    .map_err(|_| "Stamp exceeds the i32 coordinate range")?,
                i32::try_from(bounds.max.2 as i64 + offset.2)
                    .map_err(|_| "Stamp exceeds the i32 coordinate range")?,
            ],
        };
        let receipt = OperationReceipt {
            start_ms,
            duration_ms,
            id: self.operations.len() as u32,
            scope: OperationScope::StampRegion(region.to_string()),
            kind: OperationKind::Stamp,
            before_bounds: Some(before_bounds),
            final_bounds: Some(final_bounds),
            pivot2: Some(before_bounds.pivot2()),
            final_pivot2: Some(final_bounds.pivot2()),
            affine: Some(LatticeAffine {
                linear: [[1, 0, 0], [0, 1, 0], [0, 0, 1]],
                offset: [offset.0, offset.1, offset.2],
            }),
            cells: mappings
                .into_iter()
                .map(|(before, final_position, block)| CellDelta {
                    region: region.to_string(),
                    before_position: [before.0, before.1, before.2],
                    final_position: [final_position.0, final_position.1, final_position.2],
                    before_block: block.clone(),
                    final_block: block,
                })
                .collect(),
            excluded_cells,
            block_entities,
            entities,
        };
        self.operations.push(RecordedOperation {
            targets,
            transform: OperationTransform::Translate {
                inverse_delta: [-(offset.0 as f32), -(offset.1 as f32), -(offset.2 as f32)],
            },
            start_ms,
            duration_ms,
            appears: false,
            receipt,
        });
        Ok(())
    }

    pub fn stamp_box(
        &mut self,
        source: &UniversalSchematic,
        bounds: crate::bounding_box::BoundingBox,
        target: (i32, i32, i32),
        excluded_blocks: &[String],
        duration_ms: f32,
    ) -> Result<(), String> {
        self.validate_operation(duration_ms)?;
        let excluded: Vec<crate::block_state::BlockState> = excluded_blocks
            .iter()
            .map(|value| crate::block_state::BlockState::from_block_string(value))
            .collect::<Result<_, _>>()?;
        let offset = (
            target.0 as i64 - bounds.min.0 as i64,
            target.1 as i64 - bounds.min.1 as i64,
            target.2 as i64 - bounds.min.2 as i64,
        );
        let destination = |x: i32, y: i32, z: i32| -> Result<(i32, i32, i32), String> {
            Ok((
                i32::try_from(x as i64 + offset.0)
                    .map_err(|_| "Stamp exceeds the i32 coordinate range")?,
                i32::try_from(y as i64 + offset.1)
                    .map_err(|_| "Stamp exceeds the i32 coordinate range")?,
                i32::try_from(z as i64 + offset.2)
                    .map_err(|_| "Stamp exceeds the i32 coordinate range")?,
            ))
        };
        destination(bounds.min.0, bounds.min.1, bounds.min.2)?;
        destination(bounds.max.0, bounds.max.1, bounds.max.2)?;

        let mut written = Vec::new();
        let mut mappings = Vec::new();
        let mut excluded_cells = Vec::new();
        for x in bounds.min.0..=bounds.max.0 {
            for y in bounds.min.1..=bounds.max.1 {
                for z in bounds.min.2..=bounds.max.2 {
                    let Some(block) = source.get_block(x, y, z) else {
                        continue;
                    };
                    let final_position = destination(x, y, z)?;
                    if excluded.contains(block) {
                        excluded_cells.push([final_position.0, final_position.1, final_position.2]);
                        continue;
                    }
                    written.push(final_position);
                    mappings.push(((x, y, z), final_position, block.to_string()));
                }
            }
        }

        let mut transformed = self.schematic.clone();
        transformed.stamp_box(source, &bounds, target, &excluded)?;
        let block_entities: Vec<BlockEntityDelta> = mappings
            .iter()
            .filter_map(|(source_position, final_position, _)| {
                let source_pos = crate::block_position::BlockPosition {
                    x: source_position.0,
                    y: source_position.1,
                    z: source_position.2,
                };
                let final_pos = crate::block_position::BlockPosition {
                    x: final_position.0,
                    y: final_position.1,
                    z: final_position.2,
                };
                let source_state = source.get_block_entity(source_pos).cloned();
                let replaced_destination = self.schematic.get_block_entity(final_pos).cloned();
                let final_state = transformed.get_block_entity(final_pos).cloned();
                (source_state.is_some() || replaced_destination.is_some() || final_state.is_some())
                    .then_some(BlockEntityDelta {
                        source_position: [source_position.0, source_position.1, source_position.2],
                        final_position: [final_position.0, final_position.1, final_position.2],
                        source: source_state,
                        replaced_destination,
                        final_state,
                    })
            })
            .collect();
        let entities: Vec<EntityDelta> = source
            .get_entities_as_list()
            .into_iter()
            .filter(|entity| {
                bounds.contains((
                    entity.position.0.floor() as i32,
                    entity.position.1.floor() as i32,
                    entity.position.2.floor() as i32,
                ))
            })
            .map(|before| {
                let mut final_state = before.clone();
                final_state.position = (
                    before.position.0 + offset.0 as f64,
                    before.position.1 + offset.1 as f64,
                    before.position.2 + offset.2 as f64,
                );
                EntityDelta {
                    before,
                    final_state,
                }
            })
            .collect();
        written.sort_unstable();
        written.dedup();
        let source_positions = mappings.iter().map(|(position, _, _)| *position).collect();
        let start_ms = self.next_operation_start_ms();
        self.schematic = transformed;
        let targets = if written.is_empty() {
            Vec::new()
        } else {
            vec![self.install_stamp_draws(
                source,
                None,
                source_positions,
                written,
                start_ms,
                duration_ms,
            )]
        };

        let final_bounds = OperationBounds {
            min: [target.0, target.1, target.2],
            max: {
                let max = destination(bounds.max.0, bounds.max.1, bounds.max.2)?;
                [max.0, max.1, max.2]
            },
        };
        let before_bounds = OperationBounds::from(bounds);
        let receipt = OperationReceipt {
            start_ms,
            duration_ms,
            id: self.operations.len() as u32,
            scope: OperationScope::StampBox,
            kind: OperationKind::Stamp,
            before_bounds: Some(before_bounds),
            final_bounds: Some(final_bounds),
            pivot2: Some(before_bounds.pivot2()),
            final_pivot2: Some(final_bounds.pivot2()),
            affine: Some(LatticeAffine {
                linear: [[1, 0, 0], [0, 1, 0], [0, 0, 1]],
                offset: [offset.0, offset.1, offset.2],
            }),
            cells: mappings
                .into_iter()
                .map(|(before, final_position, block)| CellDelta {
                    region: "Main".to_string(),
                    before_position: [before.0, before.1, before.2],
                    final_position: [final_position.0, final_position.1, final_position.2],
                    before_block: block.clone(),
                    final_block: block,
                })
                .collect(),
            excluded_cells,
            block_entities,
            entities,
        };
        self.operations.push(RecordedOperation {
            targets,
            transform: OperationTransform::Translate {
                inverse_delta: [-(offset.0 as f32), -(offset.1 as f32), -(offset.2 as f32)],
            },
            start_ms,
            duration_ms,
            appears: false,
            receipt,
        });
        Ok(())
    }

    pub fn set_operation_gizmos(&mut self, enabled: bool) {
        self.operation_gizmos = enabled;
    }

    fn operation_bounds(operation: &RecordedOperation) -> Option<([f32; 3], [f32; 3])> {
        let bounds = operation.receipt.before_bounds.as_ref()?;
        Some((
            [
                bounds.min[0] as f32 - 0.5,
                bounds.min[1] as f32 - 0.5,
                bounds.min[2] as f32 - 0.5,
            ],
            [
                bounds.max[0] as f32 + 0.5,
                bounds.max[1] as f32 + 0.5,
                bounds.max[2] as f32 + 0.5,
            ],
        ))
    }

    fn receipt_bounds(bounds: &OperationBounds) -> ([f32; 3], [f32; 3]) {
        (
            [
                bounds.min[0] as f32 - 0.5,
                bounds.min[1] as f32 - 0.5,
                bounds.min[2] as f32 - 0.5,
            ],
            [
                bounds.max[0] as f32 + 0.5,
                bounds.max[1] as f32 + 0.5,
                bounds.max[2] as f32 + 0.5,
            ],
        )
    }

    fn line(
        frame: &mut super::Frame,
        kind: super::GizmoKind,
        start: [f32; 3],
        end: [f32; 3],
        color: [f32; 4],
    ) {
        frame.gizmos.push(super::GizmoLine {
            kind,
            start,
            end,
            color,
        });
    }

    fn box_lines(
        frame: &mut super::Frame,
        kind: super::GizmoKind,
        min: [f32; 3],
        max: [f32; 3],
        color: [f32; 4],
        matrix: [[f32; 4]; 4],
    ) {
        let corners = [
            [min[0], min[1], min[2]],
            [max[0], min[1], min[2]],
            [min[0], max[1], min[2]],
            [max[0], max[1], min[2]],
            [min[0], min[1], max[2]],
            [max[0], min[1], max[2]],
            [min[0], max[1], max[2]],
            [max[0], max[1], max[2]],
        ]
        .map(|p| super::operation::transform_point(matrix, p));
        for (a, b) in [
            (0, 1),
            (0, 2),
            (0, 4),
            (1, 3),
            (1, 5),
            (2, 3),
            (2, 6),
            (3, 7),
            (4, 5),
            (4, 6),
            (5, 7),
            (6, 7),
        ] {
            Self::line(frame, kind, corners[a], corners[b], color);
        }
    }

    fn append_operation_gizmos(
        &self,
        frame: &mut super::Frame,
        operation: &RecordedOperation,
        progress: f32,
    ) {
        let Some((min, max)) = Self::operation_bounds(operation) else {
            return;
        };
        let matrix = operation
            .targets
            .first()
            .and_then(|id| frame.pose(*id))
            .map(|pose| pose.to_matrix())
            .unwrap_or_else(super::operation::identity);
        Self::box_lines(
            frame,
            super::GizmoKind::RegionBounds,
            min,
            max,
            [0.15, 0.8, 1.0, 0.9],
            matrix,
        );
        let center = [
            (min[0] + max[0]) * 0.5,
            (min[1] + max[1]) * 0.5,
            (min[2] + max[2]) * 0.5,
        ];
        if operation.receipt.kind == OperationKind::Stamp {
            Self::box_lines(
                frame,
                super::GizmoKind::StampSourceBounds,
                min,
                max,
                [0.2, 0.75, 1.0, 0.75],
                super::operation::identity(),
            );
            if let Some(final_bounds) = operation.receipt.final_bounds.as_ref() {
                let (final_min, final_max) = Self::receipt_bounds(final_bounds);
                Self::box_lines(
                    frame,
                    super::GizmoKind::StampDestinationBounds,
                    final_min,
                    final_max,
                    [0.25, 1.0, 0.35, 0.9],
                    super::operation::identity(),
                );
            }
            for position in &operation.receipt.excluded_cells {
                let excluded = OperationBounds {
                    min: *position,
                    max: *position,
                };
                let (excluded_min, excluded_max) = Self::receipt_bounds(&excluded);
                Self::box_lines(
                    frame,
                    super::GizmoKind::ExcludedCell,
                    excluded_min,
                    excluded_max,
                    [1.0, 0.15, 0.15, 1.0],
                    super::operation::identity(),
                );
            }
        }
        match operation.transform {
            OperationTransform::Rotate {
                axis,
                inverse_degrees,
                pivot,
                ..
            } => {
                let radius =
                    ((max[0] - min[0]).max(max[1] - min[1]).max(max[2] - min[2]) * 0.65).max(1.0);
                let mut axis_start = pivot;
                let mut axis_end = pivot;
                let i = match axis {
                    TransformAxis::X => 0,
                    TransformAxis::Y => 1,
                    TransformAxis::Z => 2,
                };
                axis_start[i] -= radius;
                axis_end[i] += radius;
                Self::line(
                    frame,
                    super::GizmoKind::Pivot,
                    axis_start,
                    axis_end,
                    [1.0, 0.55, 0.1, 1.0],
                );
                let mut base = pivot;
                match axis {
                    TransformAxis::X => base[1] += radius,
                    TransformAxis::Y => base[0] += radius,
                    TransformAxis::Z => base[0] += radius,
                }
                let mut last = base;
                let segments = 24;
                for n in 1..=segments {
                    let q = n as f32 / segments as f32;
                    let arc = OperationTransform::Rotate {
                        axis,
                        inverse_degrees: inverse_degrees * progress * q,
                        pivot,
                        final_pivot: pivot,
                    };
                    let next = super::operation::transform_point(arc.matrix_at(1.0), base);
                    Self::line(
                        frame,
                        super::GizmoKind::RotationArc,
                        last,
                        next,
                        [1.0, 0.55, 0.1, 1.0],
                    );
                    last = next;
                }
            }
            OperationTransform::Flip { axis, plane } => {
                let p = match axis {
                    TransformAxis::X => [
                        [plane, min[1], min[2]],
                        [plane, max[1], min[2]],
                        [plane, max[1], max[2]],
                        [plane, min[1], max[2]],
                    ],
                    TransformAxis::Y => [
                        [min[0], plane, min[2]],
                        [max[0], plane, min[2]],
                        [max[0], plane, max[2]],
                        [min[0], plane, max[2]],
                    ],
                    TransformAxis::Z => [
                        [min[0], min[1], plane],
                        [max[0], min[1], plane],
                        [max[0], max[1], plane],
                        [min[0], max[1], plane],
                    ],
                };
                for i in 0..4 {
                    Self::line(
                        frame,
                        super::GizmoKind::SymmetryPlane,
                        p[i],
                        p[(i + 1) % 4],
                        [0.8, 0.35, 1.0, 0.9],
                    );
                }
            }
            OperationTransform::Translate { inverse_delta } => {
                let destination = [
                    center[0] - inverse_delta[0],
                    center[1] - inverse_delta[1],
                    center[2] - inverse_delta[2],
                ];
                Self::line(
                    frame,
                    super::GizmoKind::TranslationArrow,
                    center,
                    destination,
                    [1.0, 0.65, 0.1, 1.0],
                );
                if operation.receipt.kind == OperationKind::Stamp {
                    for i in 0..3 {
                        let mut a = destination;
                        let mut b = destination;
                        a[i] -= 0.4;
                        b[i] += 0.4;
                        Self::line(
                            frame,
                            super::GizmoKind::DestinationAnchor,
                            a,
                            b,
                            [1.0, 0.2, 0.2, 1.0],
                        );
                    }
                }
            }
        }
    }

    pub fn frame_at(&self, t_ms: f32) -> super::Frame {
        let mut frame = self.timeline().seek(t_ms);
        for (id, pose) in &mut frame.poses {
            if let Some(step) = self.steps.get(*id as usize) {
                if t_ms < step.visible_from_ms
                    || step.visible_until_ms.is_some_and(|end| t_ms >= end)
                {
                    pose.opacity = 0.0;
                }
            }
            let mut operation_matrix = super::operation::identity();
            for operation in &self.operations {
                if operation.targets.contains(id) {
                    let progress = if operation.duration_ms <= f32::EPSILON {
                        (t_ms >= operation.start_ms) as u8 as f32
                    } else {
                        ((t_ms - operation.start_ms) / operation.duration_ms).clamp(0.0, 1.0)
                    };
                    if operation.appears {
                        pose.opacity *= progress;
                    }
                    operation_matrix = super::operation::multiply(
                        operation_matrix,
                        operation.transform.matrix_at(progress),
                    );
                }
            }
            pose.matrix = Some(super::operation::multiply(
                operation_matrix,
                pose.to_matrix(),
            ));
        }
        if self.operation_gizmos {
            for operation in &self.operations {
                let end = operation.start_ms + operation.duration_ms;
                if t_ms >= operation.start_ms && t_ms <= end {
                    let progress = if operation.duration_ms <= f32::EPSILON {
                        1.0
                    } else {
                        ((t_ms - operation.start_ms) / operation.duration_ms).clamp(0.0, 1.0)
                    };
                    self.append_operation_gizmos(&mut frame, operation, progress);
                }
            }
        }
        frame
    }

    pub fn animate_camera(&mut self, clip: Clip, offset_ms: f32) {
        self.camera.push((clip, offset_ms));
    }

    pub fn groups(&self) -> Vec<Group> {
        self.steps
            .iter()
            .enumerate()
            .map(|(i, step)| Group::new(i as GroupId, step.blocks.clone()))
            .collect()
    }

    /// Mesh every animation draw from the immutable schematic snapshot owned by
    /// that draw. This preserves pre-operation directional states while a
    /// transform is moving and authoritative final states after the endpoint.
    #[cfg(feature = "meshing")]
    pub fn mesh_outputs(
        &self,
        pack: &crate::meshing::ResourcePackSource,
        config: &crate::meshing::MeshConfig,
    ) -> crate::meshing::Result<Vec<crate::meshing::MeshOutput>> {
        let mut outputs = Vec::with_capacity(self.steps.len());
        for (id, step) in self.steps.iter().enumerate() {
            let group = Group::new(id as GroupId, step.blocks.clone());
            let mut meshed = step.mesh_source.mesh_groups_in_region(
                pack,
                config,
                step.mesh_region.as_deref(),
                &[group],
            )?;
            outputs.push(meshed.remove(0));
        }
        Ok(outputs)
    }

    pub fn timeline(&self) -> Timeline {
        let groups = self.groups();
        let mut timeline = Timeline::new(groups);
        let delays = self.delays();
        for (i, step) in self.steps.iter().enumerate() {
            let offset = if step.visible_from_ms > 0.0 {
                0.0
            } else {
                delays[i]
            };
            timeline.add(
                step.effect
                    .clone()
                    .unwrap_or_else(|| self.default_effect.clone()),
                Target::Group(i as GroupId),
                offset,
            );
        }
        for (clip, offset) in &self.camera {
            timeline.add(clip.clone(), Target::Camera, *offset);
        }
        timeline
    }

    pub fn duration_ms(&self) -> f32 {
        self.next_operation_start_ms()
            .max(self.timeline().duration_ms())
    }

    /// Sample deterministic frames and optionally hold the final state.
    ///
    /// Loop captures round the requested frame count to the nearest whole
    /// frame, then partition `[0, period)` evenly so the full cycle is sampled
    /// without duplicating its endpoint.
    pub fn frames(&self, fps: f64, hold_ms: f32) -> Vec<super::Frame> {
        let timeline = self.timeline();
        let fps = fps.max(1.0);
        let base_duration = self.next_operation_start_ms().max(timeline.duration_ms()) as f64;
        let duration = self
            .loop_period_ms
            .map(f64::from)
            .unwrap_or_else(|| base_duration + hold_ms.max(0.0) as f64);
        let count = ((duration / 1000.0) * fps).round().max(1.0) as usize;
        let step_ms = if self.loop_period_ms.is_some() {
            duration / count as f64
        } else {
            1000.0 / fps
        };
        if self.loop_period_ms.is_some() {
            return (0..count)
                .map(|i| self.frame_at((i as f64 * step_ms) as f32))
                .collect();
        }
        let mut times: Vec<f32> = (0..count)
            .map(|i| (i as f64 * step_ms) as f32)
            .filter(|time| *time < duration as f32)
            .collect();
        times.push(base_duration as f32);
        if duration > base_duration {
            times.push(duration as f32);
        }
        times.sort_by(f32::total_cmp);
        times.dedup_by(|a, b| (*a - *b).abs() <= 0.001);
        times.into_iter().map(|time| self.frame_at(time)).collect()
    }

    fn delays(&self) -> Vec<f32> {
        let Some(total) = self.stagger_total_ms else {
            return (0..self.steps.len())
                .map(|i| i as f32 * self.step_ms + self.stagger_offset_ms)
                .collect();
        };
        if self.steps.len() <= 1 {
            return vec![self.stagger_offset_ms; self.steps.len()];
        }
        let min = self
            .steps
            .iter()
            .map(|s| s.key)
            .fold(f32::INFINITY, f32::min);
        let max = self
            .steps
            .iter()
            .map(|s| s.key)
            .fold(f32::NEG_INFINITY, f32::max);
        let span = max - min;
        if span.abs() <= f32::EPSILON {
            return vec![self.stagger_offset_ms; self.steps.len()];
        }
        self.steps
            .iter()
            .map(|s| (s.key - min) / span * total + self.stagger_offset_ms)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::animation::{presets, Property};
    use crate::building::{BrushEnum, Curve3D, ShapeEnum, SolidBrush, TubePath};

    #[test]
    fn records_single_mutations_and_explicit_groups_as_animation_targets() {
        let mut animation = BuildAnimation::new("toy");
        animation.begin_group(None).unwrap();
        animation.set_block(0, 0, 0, "minecraft:stone").unwrap();
        animation.set_block(1, 0, 0, "minecraft:stone").unwrap();
        let floor = animation.end_group().unwrap();
        let prop = animation.set_block(0, 1, 0, "minecraft:furnace").unwrap();

        assert_eq!(floor, 0);
        assert_eq!(prop, 1);
        assert_eq!(animation.groups()[0].blocks.len(), 2);
        assert_eq!(animation.groups()[1].blocks, vec![(0, 1, 0)]);
    }

    #[test]
    fn replacing_a_block_transfers_the_position_to_the_latest_step() {
        let mut animation = BuildAnimation::new("replacement");
        animation.begin_group(None).unwrap();
        animation
            .set_block(0, 1, 0, "minecraft:oak_planks")
            .unwrap();
        animation
            .set_block(1, 1, 0, "minecraft:oak_planks")
            .unwrap();
        animation.end_group().unwrap();

        animation
            .set_block(0, 1, 0, "minecraft:wall_torch[facing=south]")
            .unwrap();

        let groups = animation.groups();
        assert_eq!(groups[0].blocks, vec![(1, 1, 0)]);
        assert_eq!(groups[1].blocks, vec![(0, 1, 0)]);
        assert_eq!(
            groups
                .iter()
                .flat_map(|group| &group.blocks)
                .filter(|&&position| position == (0, 1, 0))
                .count(),
            1,
            "a replaced coordinate must render in only its latest animation step"
        );
        assert_eq!(
            animation
                .schematic()
                .get_block(0, 1, 0)
                .expect("replacement block")
                .get_name(),
            "minecraft:wall_torch"
        );
    }

    #[test]
    fn operation_effect_overrides_the_animation_default() {
        let mut animation = BuildAnimation::new("toy");
        animation.set_default_effect(presets::drop_and_pop(500.0, 5.0));
        animation.set_block(0, 0, 0, "minecraft:stone").unwrap();
        let custom = animation
            .with_effect(presets::spin_in(800.0, 1.0))
            .set_block(1, 0, 0, "minecraft:stone")
            .unwrap();
        let plain = animation.set_block(2, 0, 0, "minecraft:stone").unwrap();

        let timeline = animation.timeline();
        assert_eq!(timeline.seek(0.0).pose(0).unwrap().translate[1], 5.0);
        assert_eq!(
            timeline
                .seek(animation.step_ms())
                .pose(custom)
                .unwrap()
                .rotate_deg[1],
            360.0
        );
        assert_eq!(
            timeline
                .seek(animation.step_ms() * 2.0)
                .pose(plain)
                .unwrap()
                .translate[1],
            5.0,
            "with_effect must be consumed by exactly one operation"
        );
    }

    #[test]
    fn camera_uses_the_same_clip_timeline_as_groups() {
        let mut animation = BuildAnimation::new("camera");
        animation.set_block(0, 0, 0, "minecraft:stone").unwrap();
        animation.animate_camera(presets::turntable(1_000.0), 0.0);

        let camera = animation.timeline().seek(500.0).camera.unwrap();
        assert!((camera.yaw - 180.0).abs() < 0.01);
    }

    #[test]
    fn failed_operation_consumes_the_pending_effect() {
        let mut animation = BuildAnimation::new("failed-operation-effect");
        let failed = animation
            .with_effect(presets::spin_in(800.0, 1.0))
            .set_block(0, 0, 0, "not a valid block[");
        assert!(failed.is_err());

        let plain = animation.set_block(1, 0, 0, "minecraft:stone").unwrap();
        let timeline = animation.timeline();
        let frame = timeline.seek(0.0);
        let pose = frame.pose(plain).unwrap();
        assert!(pose.rotate_deg.iter().all(|value| value.abs() < 1e-4));
    }

    #[test]
    fn with_effect_applies_to_the_next_group_as_one_target() {
        let mut animation = BuildAnimation::new("group-effect");
        animation
            .with_effect(presets::spin_in(700.0, 1.0))
            .begin_group(None)
            .unwrap();
        animation.set_block(0, 0, 0, "minecraft:stone").unwrap();
        animation.set_block(1, 0, 0, "minecraft:stone").unwrap();
        let group = animation.end_group().unwrap();
        let plain = animation.set_block(2, 0, 0, "minecraft:stone").unwrap();

        let timeline = animation.timeline();
        assert_eq!(timeline.seek(0.0).pose(group).unwrap().rotate_deg[1], 360.0);
        assert_eq!(
            timeline
                .seek(animation.step_ms())
                .pose(plain)
                .unwrap()
                .translate[1],
            4.5,
            "the group must consume the pending effect"
        );
    }

    #[test]
    fn with_effect_applies_to_entity_recording_and_is_consumed() {
        let mut animation = BuildAnimation::new("entity-effect");
        let custom = animation
            .with_effect(presets::spin_in(700.0, 1.0))
            .record_entity_position(0.5, 1.0, 0.5)
            .unwrap();
        let plain = animation.record_entity_position(2.5, 1.0, 0.5).unwrap();

        let timeline = animation.timeline();
        assert_eq!(
            timeline.seek(0.0).pose(custom).unwrap().rotate_deg[1],
            360.0
        );
        assert_eq!(
            timeline
                .seek(animation.step_ms())
                .pose(plain)
                .unwrap()
                .translate[1],
            4.5
        );
    }

    #[test]
    fn keyed_groups_can_drive_a_trefoil_style_stagger() {
        let mut animation = BuildAnimation::new("curve");
        for i in 0..3 {
            animation.begin_group_with_key(None, i as f32).unwrap();
            animation.set_block(i, 0, 0, "minecraft:stone").unwrap();
            animation.end_group().unwrap();
        }
        animation.set_stagger_total_ms(Some(1_000.0));
        animation.set_default_effect(presets::pop_in(200.0));

        let timeline = animation.timeline();
        assert_eq!(timeline.seek(0.0).pose(0).unwrap().scale, [0.0; 3]);
        assert_eq!(timeline.seek(0.0).pose(2).unwrap().scale, [0.0; 3]);
        assert!(timeline.seek(600.0).pose(0).unwrap().scale[0] > 0.99);
        assert_eq!(timeline.seek(600.0).pose(2).unwrap().scale, [0.0; 3]);
    }

    #[test]
    fn parametric_fill_records_one_ordered_group_per_curve_partition() {
        let curve = Curve3D::new(vec![(0.0, 0.0, 0.0), (9.0, 0.0, 0.0)], false).unwrap();
        let shape = ShapeEnum::TubePath(TubePath::new(curve, 0.6).unwrap());
        let brush = BrushEnum::Solid(SolidBrush::new(crate::BlockState::new("minecraft:stone")));
        let mut animation = BuildAnimation::new("parametric");

        animation.fill_along_parameter(&shape, &brush, 4).unwrap();

        assert_eq!(animation.groups().len(), 4);
        assert_eq!(
            animation
                .steps
                .iter()
                .map(|step| step.blocks.len())
                .sum::<usize>(),
            10
        );
        assert_eq!(
            animation
                .groups()
                .iter()
                .flat_map(|group| group.blocks.iter())
                .collect::<std::collections::HashSet<_>>()
                .len(),
            10,
            "every filled voxel must belong to exactly one animation group"
        );
        assert_eq!(
            animation.schematic().get_block(9, 0, 0).unwrap().get_name(),
            "minecraft:stone"
        );
        assert!(animation
            .steps
            .windows(2)
            .all(|pair| pair[0].key < pair[1].key));
    }

    #[test]
    fn loop_period_and_negative_stagger_phase_capture_one_seamless_cycle() {
        let mut animation = BuildAnimation::new("loop");
        for i in 0..3 {
            animation.begin_group_with_key(None, i as f32).unwrap();
            animation.set_block(i, 0, 0, "minecraft:stone").unwrap();
            animation.end_group().unwrap();
        }
        let effect = AnimationEffect::new(1_000.0)
            .keyframe(Property::ScaleUniform, 0.0, 0.0, Easing::Linear)
            .keyframe(Property::ScaleUniform, 0.1, 1.0, Easing::Linear)
            .keyframe(Property::ScaleUniform, 0.58, 1.0, Easing::Linear)
            .keyframe(Property::ScaleUniform, 0.68, 0.0, Easing::Linear)
            .keyframe(Property::ScaleUniform, 1.0, 0.0, Easing::Linear)
            .repeat_forever();
        animation.set_default_effect(effect);
        animation.set_stagger_total_ms(Some(400.0));
        animation.set_stagger_offset_ms(-1_000.0);
        animation.set_loop_period_ms(Some(1_000.0));

        let frames = animation.frames(20.0, 5_000.0);

        assert_eq!(
            frames.len(),
            20,
            "loop capture must ignore one-shot hold time"
        );
        assert!(frames.first().unwrap().pose(2).unwrap().scale[0] > 0.0);
        assert!(frames.last().unwrap().pose(2).unwrap().scale[0] > 0.0);
    }

    #[test]
    fn non_integral_loop_frame_counts_still_sample_the_complete_period() {
        let mut animation = BuildAnimation::new("fractional-loop");
        animation.set_block(0, 0, 0, "minecraft:stone").unwrap();
        animation.set_loop_period_ms(Some(1_025.0));

        let frames = animation.frames(20.0, 0.0);

        assert_eq!(frames.len(), 21);
        let expected_last_time = 1_025.0 * 20.0 / 21.0;
        assert!((frames.last().unwrap().time_ms - expected_last_time).abs() < 0.001);
        assert!(frames.last().unwrap().time_ms < 1_025.0);
    }

    #[test]
    fn frame_capture_can_add_a_readme_hold_after_the_timeline() {
        let mut animation = BuildAnimation::new("hold");
        animation.set_block(0, 0, 0, "minecraft:stone").unwrap();
        let frames = animation.frames(20.0, 1_000.0);
        assert_eq!(frames.len(), 32); // 30 cadence samples + exact timeline and hold endpoints
        assert!(frames
            .iter()
            .any(|frame| (frame.time_ms - 480.0).abs() < 0.001));
        assert!((frames.last().unwrap().time_ms - 1_480.0).abs() < 0.001);
        assert!(frames.last().unwrap().pose(0).unwrap().scale[0] > 0.99);
    }

    #[test]
    fn custom_effect_tracks_are_shared_by_all_language_adapters() {
        let effect = AnimationEffect::new(500.0)
            .tween(Property::Y, 4.0, 0.0, crate::animation::Easing::Linear)
            .tween(
                Property::ScaleUniform,
                0.0,
                1.0,
                crate::animation::Easing::out_back(),
            );
        assert_eq!(effect.clip().sample(0.0).translate[1], 4.0);
        assert_eq!(effect.clip().sample(0.0).scale, [0.0; 3]);
        assert!((effect.clip().sample(500.0).scale[0] - 1.0).abs() < 0.001);
    }
}
