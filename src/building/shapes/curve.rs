use std::collections::BTreeSet;

use super::{ParametricShape, Shape};

/// A finite sampled polyline with arc-length parameterisation.
///
/// Curves are limited to [`Curve3D::MAX_POINTS`]. Segment and total lengths
/// that overflow `f64` are rejected before the curve can be voxelised.
#[derive(Debug, Clone)]
pub struct Curve3D {
    points: Vec<(f64, f64, f64)>,
    closed: bool,
    segments: Vec<CurveSegment>,
    total_length: f64,
}

#[derive(Debug, Clone, Copy)]
struct CurveSegment {
    start: (f64, f64, f64),
    end: (f64, f64, f64),
    length: f64,
    cumulative_length: f64,
}

impl Curve3D {
    /// Hard cap applied before segment construction and FFI allocation.
    pub const MAX_POINTS: usize = 100_000;

    pub fn new(points: Vec<(f64, f64, f64)>, closed: bool) -> Result<Self, String> {
        let minimum = if closed { 3 } else { 2 };
        if points.len() < minimum {
            return Err(format!(
                "a {} curve requires at least {minimum} points",
                if closed { "closed" } else { "open" }
            ));
        }
        if points.len() > Self::MAX_POINTS {
            return Err(format!(
                "curve exceeds the maximum of {} points",
                Self::MAX_POINTS
            ));
        }
        if points
            .iter()
            .flat_map(|p| [p.0, p.1, p.2])
            .any(|value| !value.is_finite())
        {
            return Err("curve points must be finite".into());
        }

        let pair_count = if closed {
            points.len()
        } else {
            points.len() - 1
        };
        let mut segments = Vec::with_capacity(pair_count);
        let mut total_length = 0.0;
        for index in 0..pair_count {
            let start = points[index];
            let end = points[(index + 1) % points.len()];
            let dx = end.0 - start.0;
            let dy = end.1 - start.1;
            let dz = end.2 - start.2;
            if !dx.is_finite() || !dy.is_finite() || !dz.is_finite() {
                return Err("curve segment delta overflowed".into());
            }
            let length = (dx * dx + dy * dy + dz * dz).sqrt();
            if !length.is_finite() {
                return Err("curve segment length overflowed".into());
            }
            if length <= f64::EPSILON {
                continue;
            }
            segments.push(CurveSegment {
                start,
                end,
                length,
                cumulative_length: total_length,
            });
            total_length += length;
            if !total_length.is_finite() {
                return Err("curve total length overflowed".into());
            }
        }
        if segments.is_empty() {
            return Err("curve must contain at least one non-zero segment".into());
        }

        Ok(Self {
            points,
            closed,
            segments,
            total_length,
        })
    }

    pub fn points(&self) -> &[(f64, f64, f64)] {
        &self.points
    }

    pub fn is_closed(&self) -> bool {
        self.closed
    }

    fn closest_point_info(&self, point: (f64, f64, f64)) -> ClosestPoint {
        let mut best = ClosestPoint {
            distance: f64::INFINITY,
            position: self.segments[0].start,
            parameter: 0.0,
        };
        for segment in &self.segments {
            let dx = segment.end.0 - segment.start.0;
            let dy = segment.end.1 - segment.start.1;
            let dz = segment.end.2 - segment.start.2;
            let length_sq = segment.length * segment.length;
            let local = (((point.0 - segment.start.0) * dx
                + (point.1 - segment.start.1) * dy
                + (point.2 - segment.start.2) * dz)
                / length_sq)
                .clamp(0.0, 1.0);
            let position = (
                segment.start.0 + local * dx,
                segment.start.1 + local * dy,
                segment.start.2 + local * dz,
            );
            let rx = point.0 - position.0;
            let ry = point.1 - position.1;
            let rz = point.2 - position.2;
            let distance = (rx * rx + ry * ry + rz * rz).sqrt();
            if distance < best.distance {
                best = ClosestPoint {
                    distance,
                    position,
                    parameter: ((segment.cumulative_length + local * segment.length)
                        / self.total_length)
                        .clamp(0.0, 1.0),
                };
            }
        }
        best
    }
}

#[derive(Debug, Clone, Copy)]
struct ClosestPoint {
    distance: f64,
    position: (f64, f64, f64),
    parameter: f64,
}

/// A voxel tube around a [`Curve3D`].
///
/// Construction validates voxel-coordinate bounds and rejects shapes whose
/// estimated candidate or closest-segment work exceeds internal safety
/// budgets. This makes malformed finite inputs fail before enumeration.
#[derive(Debug, Clone)]
pub struct TubePath {
    curve: Curve3D,
    radius: f64,
    reach: i32,
    sample_counts: Vec<usize>,
}

impl TubePath {
    const MAX_CANDIDATE_VISITS: u128 = 10_000_000;
    const MAX_DISTANCE_CHECKS: u128 = 500_000_000;

    pub fn new(curve: Curve3D, radius: f64) -> Result<Self, String> {
        if !radius.is_finite() || radius <= 0.0 {
            return Err("tube radius must be finite and greater than zero".into());
        }

        let reach_f64 = radius.ceil();
        if reach_f64 > i32::MAX as f64 {
            return Err("tube radius exceeds the supported voxel range".into());
        }
        let reach = reach_f64 as i32;
        let coordinate_margin = reach as f64 + 1.0;
        let minimum_coordinate = i32::MIN as f64 + coordinate_margin;
        let maximum_coordinate = i32::MAX as f64 - coordinate_margin;
        if curve
            .points()
            .iter()
            .flat_map(|point| [point.0, point.1, point.2])
            .any(|value| value < minimum_coordinate || value > maximum_coordinate)
        {
            return Err("tube bounds exceed the supported voxel coordinate range".into());
        }

        let side = (reach as u128)
            .checked_mul(2)
            .and_then(|value| value.checked_add(1))
            .ok_or_else(|| "tube candidate volume overflowed".to_string())?;
        let candidates_per_sample = side
            .checked_mul(side)
            .and_then(|value| value.checked_mul(side))
            .ok_or_else(|| "tube candidate volume overflowed".to_string())?;

        let mut sample_counts = Vec::with_capacity(curve.segments.len());
        let mut sample_centers = 0_u128;
        for segment in &curve.segments {
            let raw_samples = (segment.length * 2.0).ceil().max(1.0);
            if !raw_samples.is_finite() || raw_samples > usize::MAX as f64 {
                return Err("tube segment requires too many samples".into());
            }
            let samples = raw_samples as usize;
            sample_centers = sample_centers
                .checked_add(samples as u128 + 1)
                .ok_or_else(|| "tube sample count overflowed".to_string())?;
            sample_counts.push(samples);
        }

        let candidate_visits = sample_centers
            .checked_mul(candidates_per_sample)
            .ok_or_else(|| "tube voxelization work estimate overflowed".to_string())?;
        if candidate_visits > Self::MAX_CANDIDATE_VISITS {
            return Err("tube exceeds the voxel candidate budget".into());
        }
        let distance_checks = candidate_visits
            .checked_mul(curve.segments.len() as u128)
            .ok_or_else(|| "tube distance-check estimate overflowed".to_string())?;
        if distance_checks > Self::MAX_DISTANCE_CHECKS {
            return Err("tube exceeds the distance-check budget".into());
        }

        Ok(Self {
            curve,
            radius,
            reach,
            sample_counts,
        })
    }

    pub fn curve(&self) -> &Curve3D {
        &self.curve
    }

    fn voxel_points(&self) -> Vec<(i32, i32, i32)> {
        let mut points = BTreeSet::new();
        let reach = self.reach;
        for (segment, &samples) in self.curve.segments.iter().zip(&self.sample_counts) {
            for sample in 0..=samples {
                let t = sample as f64 / samples as f64;
                let center = (
                    segment.start.0 + (segment.end.0 - segment.start.0) * t,
                    segment.start.1 + (segment.end.1 - segment.start.1) * t,
                    segment.start.2 + (segment.end.2 - segment.start.2) * t,
                );
                let cx = center.0.round() as i32;
                let cy = center.1.round() as i32;
                let cz = center.2.round() as i32;
                for dx in -reach..=reach {
                    for dy in -reach..=reach {
                        for dz in -reach..=reach {
                            let voxel = (cx + dx, cy + dy, cz + dz);
                            if self.contains(voxel.0, voxel.1, voxel.2) {
                                points.insert(voxel);
                            }
                        }
                    }
                }
            }
        }
        points.into_iter().collect()
    }
}

impl Shape for TubePath {
    fn contains(&self, x: i32, y: i32, z: i32) -> bool {
        self.curve
            .closest_point_info((x as f64, y as f64, z as f64))
            .distance
            <= self.radius
    }

    fn points(&self) -> Vec<(i32, i32, i32)> {
        self.voxel_points()
    }

    fn normal_at(&self, x: i32, y: i32, z: i32) -> (f64, f64, f64) {
        let closest = self
            .curve
            .closest_point_info((x as f64, y as f64, z as f64));
        let normal = (
            x as f64 - closest.position.0,
            y as f64 - closest.position.1,
            z as f64 - closest.position.2,
        );
        let length = (normal.0 * normal.0 + normal.1 * normal.1 + normal.2 * normal.2).sqrt();
        if length <= f64::EPSILON {
            (0.0, 1.0, 0.0)
        } else {
            (normal.0 / length, normal.1 / length, normal.2 / length)
        }
    }

    fn bounds(&self) -> (i32, i32, i32, i32, i32, i32) {
        let mut min = (f64::INFINITY, f64::INFINITY, f64::INFINITY);
        let mut max = (f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);
        for point in self.curve.points() {
            min.0 = min.0.min(point.0);
            min.1 = min.1.min(point.1);
            min.2 = min.2.min(point.2);
            max.0 = max.0.max(point.0);
            max.1 = max.1.max(point.1);
            max.2 = max.2.max(point.2);
        }
        (
            (min.0 - self.radius).floor() as i32,
            (min.1 - self.radius).floor() as i32,
            (min.2 - self.radius).floor() as i32,
            (max.0 + self.radius).ceil() as i32,
            (max.1 + self.radius).ceil() as i32,
            (max.2 + self.radius).ceil() as i32,
        )
    }

    fn for_each_point<F>(&self, mut f: F)
    where
        F: FnMut(i32, i32, i32),
    {
        for (x, y, z) in self.voxel_points() {
            f(x, y, z);
        }
    }
}

impl ParametricShape for TubePath {
    fn parameter_at(&self, x: i32, y: i32, z: i32) -> f64 {
        self.curve
            .closest_point_info((x as f64, y as f64, z as f64))
            .parameter
    }
}
