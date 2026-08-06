//! TNT and explosions — `PrimedTnt` + `ServerExplosion`, bytecode shapes.
//!
//! What is modelled: the primed-TNT entity's fall and fuse; the 1352-ray
//! destruction sweep with per-block blast resistance; `getSeenPercent`'s
//! sample grid over an entity's box; and the knockback/damage a caught
//! entity takes. What is not: drops from destroyed blocks (vanilla rolls a
//! `1/power` chance per block with world random — nothing measured reads
//! the drops), and vanilla's shuffle of the destruction list before removal
//! (blocks are removed in sorted order here; the update-order difference is
//! real but nothing measured distinguishes it yet).
//!
//! Randomness: vanilla draws each ray's intensity from the **world** random,
//! whose state at explosion time is unknowable from a save. With a seeded
//! simulation the engine draws from its own `JavaRandom` stream —
//! deterministic, but a *different* stream than any real server. The
//! measured machines are built so the outcome does not depend on the rolls
//! (obsidian shields and point-blank targets); without a seed every roll
//! sits on its mean, which makes intensity exactly `power`.

use crate::entity::CollisionWorld;
use crate::pos::Pos;
use crate::rng::JavaRandom;

/// `PrimedTnt`, the entity: a falling 0.98-cube with a fuse.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimedTnt {
    /// Trace-stable id, from the shared entity counter.
    pub id: u32,
    /// Feet-centre position.
    pub pos: [f64; 3],
    /// Velocity, blocks per tick.
    pub vel: [f64; 3],
    /// Ticks until detonation; explodes when it reaches zero.
    pub fuse: i32,
    /// Set when discarded.
    pub removed: bool,
}

/// `PrimedTnt`'s hitbox: `sized(0.98F, 0.98F)`.
pub fn tnt_aabb(pos: [f64; 3]) -> ([f64; 3], [f64; 3]) {
    let (x, y, z) = (pos[0], pos[1], pos[2]);
    ([x - 0.49, y, z - 0.49], [x + 0.49, y + 0.98, z + 0.49])
}

/// One tick of `PrimedTnt.tick`, up to but not including the explosion:
/// gravity 0.04, move, 0.98 drag, 0.7/-0.5 ground response, fuse countdown.
/// Returns `true` when the fuse ran out — the caller owns the explosion,
/// because the explosion needs the whole simulation.
pub fn tick_tnt(tnt: &mut PrimedTnt, world: &dyn CollisionWorld) -> bool {
    tnt.vel[1] -= 0.04;
    let (min, max) = tnt_aabb(tnt.pos);
    let (moved, on_ground) = move_box(world, min, max, tnt.vel);
    for axis in 0..3 {
        tnt.pos[axis] += moved[axis];
        // An axis the move clipped keeps no velocity — `Entity.move` zeroes
        // the component that hit.
        if moved[axis] != tnt.vel[axis] {
            tnt.vel[axis] = 0.0;
        }
        tnt.vel[axis] *= 0.98;
    }
    if on_ground {
        tnt.vel[0] *= 0.7;
        tnt.vel[2] *= 0.7;
        tnt.vel[1] *= -0.5;
    }
    tnt.fuse -= 1;
    tnt.fuse <= 0
}

/// Axis-by-axis box move against full-cube solidity — the slice of
/// `Entity.move` these entities need. Y first, then X, then Z (vanilla's
/// collision order for a mover with no step-up). Returns the movement
/// actually made and whether the downward leg hit ground.
///
/// Partial blocks are read at [`CollisionWorld::solid_height`] for the
/// downward leg (a TNT block-drop rests on a hopper's funnel floor like
/// anything else) and as full cells sideways — every measured flight path
/// runs between full cubes.
pub fn move_box(
    world: &dyn CollisionWorld,
    min: [f64; 3],
    max: [f64; 3],
    vel: [f64; 3],
) -> ([f64; 3], bool) {
    let mut moved = [0.0; 3];
    let mut cur_min = min;
    let mut cur_max = max;
    let mut on_ground = false;
    for axis in [1usize, 0, 2] {
        let mut d = vel[axis];
        if d == 0.0 {
            moved[axis] = 0.0;
            continue;
        }
        let positive = d > 0.0;
        // Cells the box would sweep through on this axis.
        let lead = if positive {
            cur_max[axis]
        } else {
            cur_min[axis]
        };
        let target = lead + d;
        let (lo, hi) = if positive {
            (lead.floor() as i32, target.floor() as i32)
        } else {
            (target.floor() as i32, (lead - 1e-9).floor() as i32)
        };
        let perp = |axis: usize| -> (usize, usize) {
            match axis {
                0 => (1, 2),
                1 => (0, 2),
                _ => (0, 1),
            }
        };
        let (a, b) = perp(axis);
        let cells_a = (cur_min[a].floor() as i32)..=((cur_max[a] - 1e-9).floor() as i32);
        'scan: for step in lo..=hi {
            for ca in cells_a.clone() {
                let cells_b = (cur_min[b].floor() as i32)..=((cur_max[b] - 1e-9).floor() as i32);
                for cb in cells_b {
                    let mut cell = [0i32; 3];
                    cell[axis] = step;
                    cell[a] = ca;
                    cell[b] = cb;
                    let cell = Pos::new(cell[0], cell[1], cell[2]);
                    if axis == 1 && !positive {
                        // Downward: land on the cell's surface height.
                        let height = if world.is_solid(cell) {
                            world.solid_height(cell)
                        } else {
                            continue;
                        };
                        let surface = f64::from(cell.y) + height;
                        if surface > target && surface <= lead + 1e-9 {
                            d = surface - lead;
                            on_ground = true;
                            break 'scan;
                        }
                    } else if world.is_solid(cell) {
                        let face = f64::from(step) + if positive { 0.0 } else { 1.0 };
                        d = if positive {
                            (face - lead).max(0.0).min(d)
                        } else {
                            (face - lead).min(0.0).max(d)
                        };
                        if d.abs() < 1e-9 {
                            d = 0.0;
                        }
                        break 'scan;
                    }
                }
            }
        }
        moved[axis] = d;
        cur_min[axis] += d;
        cur_max[axis] += d;
    }
    (moved, on_ground)
}

/// `ServerExplosion.calculateExplodedPositions`: the sixteen-cubed boundary
/// rays, 0.3-step march, per-block resistance — returning the cells the
/// blast clears, sorted. `resistance` answers per *cell* (`None` is air) and
/// `destructible` is the damage calculator's `shouldBlockExplode` — a primed
/// TNT cart answers `false` for every rail and everything directly under
/// one, which is the whole of lithium's rail-shielding machine.
pub fn destruction_set(
    center: [f64; 3],
    power: f32,
    mut roll: impl FnMut() -> f32,
    mut resistance: impl FnMut(Pos) -> Option<f32>,
    mut destructible: impl FnMut(Pos) -> bool,
) -> Vec<Pos> {
    let mut cleared: Vec<Pos> = Vec::new();
    for jx in 0..16 {
        for jy in 0..16 {
            for jz in 0..16 {
                if jx != 0 && jx != 15 && jy != 0 && jy != 15 && jz != 0 && jz != 15 {
                    continue;
                }
                let dir = [
                    f64::from(jx) / 15.0 * 2.0 - 1.0,
                    f64::from(jy) / 15.0 * 2.0 - 1.0,
                    f64::from(jz) / 15.0 * 2.0 - 1.0,
                ];
                let norm = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
                let dir = [dir[0] / norm, dir[1] / norm, dir[2] / norm];
                let mut intensity = power * (0.7 + roll() * 0.6);
                let mut point = center;
                while intensity > 0.0 {
                    let cell = Pos::new(
                        point[0].floor() as i32,
                        point[1].floor() as i32,
                        point[2].floor() as i32,
                    );
                    if let Some(resist) = resistance(cell) {
                        intensity -= (resist + 0.3) * 0.3;
                    }
                    if intensity > 0.0
                        && !cleared.contains(&cell)
                        && resistance(cell).is_some()
                        && destructible(cell)
                    {
                        cleared.push(cell);
                    }
                    point[0] += dir[0] * 0.3;
                    point[1] += dir[1] * 0.3;
                    point[2] += dir[2] * 0.3;
                    intensity -= 0.225_000_01;
                }
            }
        }
    }
    cleared.sort_by_key(|p| (p.y, p.z, p.x));
    cleared
}

/// `Explosion.getSeenPercent`: the fraction of a sample grid over `bb` with
/// a clear line to the explosion centre.
///
/// Vanilla clips against real collision shapes; this walks cells on a voxel
/// DDA and asks full-cube solidity — exact wherever the occluders are full
/// cubes, which every measured shield is.
pub fn seen_percent(
    center: [f64; 3],
    bb_min: [f64; 3],
    bb_max: [f64; 3],
    world: &dyn CollisionWorld,
) -> f64 {
    let d0 = 1.0 / ((bb_max[0] - bb_min[0]) * 2.0 + 1.0);
    let d1 = 1.0 / ((bb_max[1] - bb_min[1]) * 2.0 + 1.0);
    let d2 = 1.0 / ((bb_max[2] - bb_min[2]) * 2.0 + 1.0);
    let f = (1.0 - (1.0 / d0).floor() * d0) / 2.0;
    let f1 = (1.0 - (1.0 / d2).floor() * d2) / 2.0;
    if d0 < 0.0 || d1 < 0.0 || d2 < 0.0 {
        return 0.0;
    }
    let mut seen = 0u32;
    let mut total = 0u32;
    let mut fx = 0.0f64;
    while fx <= 1.0 {
        let mut fy = 0.0f64;
        while fy <= 1.0 {
            let mut fz = 0.0f64;
            while fz <= 1.0 {
                let sample = [
                    bb_min[0] + fx * (bb_max[0] - bb_min[0]) + f,
                    bb_min[1] + fy * (bb_max[1] - bb_min[1]),
                    bb_min[2] + fz * (bb_max[2] - bb_min[2]) + f1,
                ];
                if line_clear(sample, center, world) {
                    seen += 1;
                }
                total += 1;
                fz += d2;
            }
            fy += d1;
        }
        fx += d0;
    }
    if total == 0 {
        0.0
    } else {
        f64::from(seen) / f64::from(total)
    }
}

/// Whether the open segment between two points crosses no solid cell —
/// Amanatides–Woo voxel traversal, endpoints' own cells included the way
/// vanilla's `clip` includes them (a sample inside a block is blocked).
fn line_clear(from: [f64; 3], to: [f64; 3], world: &dyn CollisionWorld) -> bool {
    let mut cell = [
        from[0].floor() as i32,
        from[1].floor() as i32,
        from[2].floor() as i32,
    ];
    let end = [
        to[0].floor() as i32,
        to[1].floor() as i32,
        to[2].floor() as i32,
    ];
    let delta = [to[0] - from[0], to[1] - from[1], to[2] - from[2]];
    let mut t_max = [f64::INFINITY; 3];
    let mut t_delta = [f64::INFINITY; 3];
    let mut step = [0i32; 3];
    for axis in 0..3 {
        if delta[axis] > 0.0 {
            step[axis] = 1;
            t_delta[axis] = 1.0 / delta[axis];
            t_max[axis] = (f64::from(cell[axis]) + 1.0 - from[axis]) / delta[axis];
        } else if delta[axis] < 0.0 {
            step[axis] = -1;
            t_delta[axis] = -1.0 / delta[axis];
            t_max[axis] = (from[axis] - f64::from(cell[axis])) / -delta[axis];
        }
    }
    loop {
        if world.is_solid(Pos::new(cell[0], cell[1], cell[2])) {
            return false;
        }
        if cell == end {
            return true;
        }
        let axis = (0..3)
            .min_by(|x, y| t_max[*x].partial_cmp(&t_max[*y]).expect("finite"))
            .expect("three axes");
        if t_max[axis] > 1.0 {
            // Ran past the segment without reaching the end cell — floating
            // point put `end` on a boundary; nothing solid was crossed.
            return true;
        }
        cell[axis] += step[axis];
        t_max[axis] += t_delta[axis];
    }
}

/// One ray-intensity roll: the seeded stream when there is one, the mean —
/// which makes intensity exactly `power` — when there is not.
pub fn intensity_roll(rng: Option<&mut JavaRandom>) -> f32 {
    match rng {
        Some(rng) => rng.next_float(),
        None => 0.5,
    }
}
