//! Per-group animatable state.
//!
//! Named `Pose` rather than `Transform`: [`crate::diff::Transform`] already owns
//! that name and means something different (a rigid alignment between two
//! builds).
//!
//! Rotations are in **degrees**, matching the reference animation APIs — it
//! keeps `degToRad` noise out of every call site.

use serde::{Deserialize, Serialize};

/// A 4x4 column-vector matrix, matching the renderer's convention.
pub type Mat4 = [[f32; 4]; 4];

/// The animatable state of one group at one instant.
///
/// [`Pose::IDENTITY`] leaves geometry exactly where the mesher put it, so an
/// un-animated group renders identically to the non-animated path.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Pose {
    pub translate: [f32; 3],
    /// Euler XYZ, in degrees.
    pub rotate_deg: [f32; 3],
    pub scale: [f32; 3],
    /// Origin for rotation and scale, in world space. Defaults to the group's
    /// centroid so "scale in place" needs no arithmetic from the caller.
    pub pivot: [f32; 3],
    /// 0..1. Folded into `tint.a` by the renderer.
    pub opacity: f32,
    /// Multiplied into the sampled base colour.
    pub tint: [f32; 4],
    /// Added after lighting. Alpha is unused; kept for uniform alignment.
    pub emissive: [f32; 4],
    /// Optional precomposed model matrix used by exact operation animations.
    #[serde(default)]
    pub matrix: Option<Mat4>,
}

impl Pose {
    pub const IDENTITY: Pose = Pose {
        translate: [0.0; 3],
        rotate_deg: [0.0; 3],
        scale: [1.0; 3],
        pivot: [0.0; 3],
        opacity: 1.0,
        tint: [1.0, 1.0, 1.0, 1.0],
        emissive: [0.0; 4],
        matrix: None,
    };

    /// An identity pose rotating and scaling about `pivot`.
    pub fn about(pivot: [f32; 3]) -> Self {
        Pose {
            pivot,
            ..Pose::IDENTITY
        }
    }

    /// True when this pose leaves geometry untouched, letting the renderer skip
    /// the per-draw uniform update.
    pub fn is_identity(&self) -> bool {
        *self == Pose::IDENTITY
    }

    /// Compose to a model matrix: `translate * to_pivot * rotate * scale * from_pivot`.
    pub fn to_matrix(&self) -> Mat4 {
        if let Some(matrix) = self.matrix {
            return matrix;
        }
        let [rx, ry, rz] = self.rotate_deg.map(f32::to_radians);
        let (sx, cx) = rx.sin_cos();
        let (sy, cy) = ry.sin_cos();
        let (sz, cz) = rz.sin_cos();

        // R = Rz * Ry * Rx (XYZ Euler applied x first), then scale columns.
        let r = [
            [cy * cz, cz * sx * sy - cx * sz, cx * cz * sy + sx * sz],
            [cy * sz, cx * cz + sx * sy * sz, -cz * sx + cx * sy * sz],
            [-sy, cy * sx, cx * cy],
        ];
        let [gx, gy, gz] = self.scale;
        let m = [
            [r[0][0] * gx, r[0][1] * gy, r[0][2] * gz],
            [r[1][0] * gx, r[1][1] * gy, r[1][2] * gz],
            [r[2][0] * gx, r[2][1] * gy, r[2][2] * gz],
        ];

        // Translation folds in the pivot round-trip: p + t - M*p
        let [px, py, pz] = self.pivot;
        let tx = self.translate[0] + px - (m[0][0] * px + m[0][1] * py + m[0][2] * pz);
        let ty = self.translate[1] + py - (m[1][0] * px + m[1][1] * py + m[1][2] * pz);
        let tz = self.translate[2] + pz - (m[2][0] * px + m[2][1] * py + m[2][2] * pz);

        // Column-major: each inner array is a column.
        [
            [m[0][0], m[1][0], m[2][0], 0.0],
            [m[0][1], m[1][1], m[2][1], 0.0],
            [m[0][2], m[1][2], m[2][2], 0.0],
            [tx, ty, tz, 1.0],
        ]
    }

    /// The matrix for transforming normals: the inverse transpose of the upper
    /// 3x3, returned as three columns.
    ///
    /// Without this, a rotated or non-uniformly scaled group shades wrong in a
    /// way that reads as a lighting bug rather than a transform bug.
    pub fn normal_matrix(&self) -> [[f32; 3]; 3] {
        let m = self.to_matrix();
        let a = [
            [m[0][0], m[1][0], m[2][0]],
            [m[0][1], m[1][1], m[2][1]],
            [m[0][2], m[1][2], m[2][2]],
        ];
        let det = a[0][0] * (a[1][1] * a[2][2] - a[1][2] * a[2][1])
            - a[0][1] * (a[1][0] * a[2][2] - a[1][2] * a[2][0])
            + a[0][2] * (a[1][0] * a[2][1] - a[1][1] * a[2][0]);
        if det.abs() < 1e-9 {
            // Degenerate (e.g. scale 0 mid-animation): fall back to identity
            // rather than emitting NaNs into the shader.
            return [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        }
        let inv_det = 1.0 / det;
        // inverse-transpose == cofactor matrix / det
        let c = [
            [
                (a[1][1] * a[2][2] - a[1][2] * a[2][1]) * inv_det,
                (a[1][2] * a[2][0] - a[1][0] * a[2][2]) * inv_det,
                (a[1][0] * a[2][1] - a[1][1] * a[2][0]) * inv_det,
            ],
            [
                (a[0][2] * a[2][1] - a[0][1] * a[2][2]) * inv_det,
                (a[0][0] * a[2][2] - a[0][2] * a[2][0]) * inv_det,
                (a[0][1] * a[2][0] - a[0][0] * a[2][1]) * inv_det,
            ],
            [
                (a[0][1] * a[1][2] - a[0][2] * a[1][1]) * inv_det,
                (a[0][2] * a[1][0] - a[0][0] * a[1][2]) * inv_det,
                (a[0][0] * a[1][1] - a[0][1] * a[1][0]) * inv_det,
            ],
        ];
        // Columns for the shader.
        [
            [c[0][0], c[1][0], c[2][0]],
            [c[0][1], c[1][1], c[2][1]],
            [c[0][2], c[1][2], c[2][2]],
        ]
    }

    /// Transform a point by this pose (host-side, for tests and bounds work).
    pub fn apply(&self, p: [f32; 3]) -> [f32; 3] {
        let m = self.to_matrix();
        [
            m[0][0] * p[0] + m[1][0] * p[1] + m[2][0] * p[2] + m[3][0],
            m[0][1] * p[0] + m[1][1] * p[1] + m[2][1] * p[2] + m[3][1],
            m[0][2] * p[0] + m[1][2] * p[1] + m[2][2] * p[2] + m[3][2],
        ]
    }
}

impl Default for Pose {
    fn default() -> Self {
        Pose::IDENTITY
    }
}

/// Translation, unit quaternion `[x, y, z, w]`, and scale of a column-major
/// affine matrix (`m[3]` holds the translation, as [`Pose::to_matrix`] writes
/// it). A mirrored matrix (negative determinant) keeps the sign on the X scale
/// so glTF viewers reproduce the flip.
pub fn decompose_trs(m: Mat4) -> ([f32; 3], [f32; 4], [f32; 3]) {
    let translation = [m[3][0], m[3][1], m[3][2]];
    let column = |i: usize| [m[i][0], m[i][1], m[i][2]];
    let len = |v: [f32; 3]| (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    let (cx, cy, cz) = (column(0), column(1), column(2));
    let det = cx[0] * (cy[1] * cz[2] - cy[2] * cz[1]) - cx[1] * (cy[0] * cz[2] - cy[2] * cz[0])
        + cx[2] * (cy[0] * cz[1] - cy[1] * cz[0]);
    let mut scale = [len(cx), len(cy), len(cz)];
    if det < 0.0 {
        scale[0] = -scale[0];
    }
    let safe = |s: f32| if s.abs() < 1e-12 { 1.0 } else { s };
    // r[c] is column c of the pure rotation.
    let r = [
        [
            cx[0] / safe(scale[0]),
            cx[1] / safe(scale[0]),
            cx[2] / safe(scale[0]),
        ],
        [
            cy[0] / safe(scale[1]),
            cy[1] / safe(scale[1]),
            cy[2] / safe(scale[1]),
        ],
        [
            cz[0] / safe(scale[2]),
            cz[1] / safe(scale[2]),
            cz[2] / safe(scale[2]),
        ],
    ];
    // Row-major view m_rc = r[c][r] for the standard matrix → quaternion form.
    let (m00, m11, m22) = (r[0][0], r[1][1], r[2][2]);
    let (m01, m02, m10, m12, m20, m21) = (r[1][0], r[2][0], r[0][1], r[2][1], r[0][2], r[1][2]);
    let trace = m00 + m11 + m22;
    let q = if trace > 0.0 {
        let s = (trace + 1.0).sqrt() * 2.0;
        [(m21 - m12) / s, (m02 - m20) / s, (m10 - m01) / s, 0.25 * s]
    } else if m00 > m11 && m00 > m22 {
        let s = (1.0 + m00 - m11 - m22).sqrt() * 2.0;
        [0.25 * s, (m01 + m10) / s, (m02 + m20) / s, (m21 - m12) / s]
    } else if m11 > m22 {
        let s = (1.0 + m11 - m00 - m22).sqrt() * 2.0;
        [(m01 + m10) / s, 0.25 * s, (m12 + m21) / s, (m02 - m20) / s]
    } else {
        let s = (1.0 + m22 - m00 - m11).sqrt() * 2.0;
        [(m02 + m20) / s, (m12 + m21) / s, 0.25 * s, (m10 - m01) / s]
    };
    let n = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3])
        .sqrt()
        .max(1e-12);
    (translation, [q[0] / n, q[1] / n, q[2] / n, q[3] / n], scale)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decompose_recovers_translate_rotate_scale() {
        let mut pose = Pose::about([0.0; 3]);
        pose.translate = [2.0, 3.0, 4.0];
        pose.rotate_deg = [0.0, 90.0, 0.0];
        pose.scale = [2.0, 2.0, 2.0];
        let (t, q, s) = decompose_trs(pose.to_matrix());
        for (a, b) in t.iter().zip([2.0, 3.0, 4.0]) {
            assert!((a - b).abs() < 1e-4, "{t:?}");
        }
        for (a, b) in s.iter().zip([2.0, 2.0, 2.0]) {
            assert!((a - b).abs() < 1e-4, "{s:?}");
        }
        let half = (45f32).to_radians();
        let expected = [0.0, half.sin(), 0.0, half.cos()];
        let dot: f32 = q.iter().zip(expected).map(|(a, b)| a * b).sum();
        assert!(dot.abs() > 0.9999, "quaternion {q:?} vs {expected:?}");
        // Round trip: rebuilding the matrix from TRS reproduces the original.
        let mut rebuilt = Pose::about([0.0; 3]);
        rebuilt.translate = t;
        rebuilt.rotate_deg = [0.0, 90.0, 0.0];
        rebuilt.scale = s;
        for (a, b) in rebuilt
            .to_matrix()
            .iter()
            .flatten()
            .zip(pose.to_matrix().iter().flatten())
        {
            assert!((a - b).abs() < 1e-4);
        }
    }

    #[test]
    fn decompose_keeps_flips_on_the_x_scale() {
        let mut m = super::super::operation::identity();
        m[0][0] = -1.0;
        let (_, q, s) = decompose_trs(m);
        assert!(
            (s[0] + 1.0).abs() < 1e-6 && (s[1] - 1.0).abs() < 1e-6,
            "{s:?}"
        );
        assert!(
            (q[3] - 1.0).abs() < 1e-6,
            "no rotation for a pure mirror: {q:?}"
        );
    }

    #[test]
    fn decompose_handles_a_collapsed_scale() {
        let mut pose = Pose::about([1.0, 1.0, 1.0]);
        pose.scale = [0.0; 3];
        let (t, q, s) = decompose_trs(pose.to_matrix());
        assert!(t.iter().chain(&q).chain(&s).all(|v| v.is_finite()));
        assert!(s.iter().all(|v| v.abs() < 1e-6));
    }

    fn close(a: [f32; 3], b: [f32; 3], eps: f32) -> bool {
        (0..3).all(|i| (a[i] - b[i]).abs() < eps)
    }

    #[test]
    fn identity_matrix_is_identity() {
        let m = Pose::IDENTITY.to_matrix();
        let expect = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        assert_eq!(m, expect);
        assert!(Pose::IDENTITY.is_identity());
    }

    #[test]
    fn translation_moves_a_point() {
        let p = Pose {
            translate: [3.0, -2.0, 5.0],
            ..Pose::IDENTITY
        };
        assert!(close(p.apply([1.0, 1.0, 1.0]), [4.0, -1.0, 6.0], 1e-5));
    }

    #[test]
    fn scale_about_pivot_keeps_the_pivot_fixed() {
        let pose = Pose {
            scale: [2.0, 2.0, 2.0],
            pivot: [5.0, 0.0, 5.0],
            ..Pose::IDENTITY
        };
        // The pivot itself must not move — this is what "scale in place" means.
        assert!(close(pose.apply([5.0, 0.0, 5.0]), [5.0, 0.0, 5.0], 1e-5));
        // A point one unit away doubles its offset.
        assert!(close(pose.apply([6.0, 0.0, 5.0]), [7.0, 0.0, 5.0], 1e-5));
    }

    #[test]
    fn rotation_about_pivot_keeps_the_pivot_fixed() {
        let pose = Pose {
            rotate_deg: [0.0, 90.0, 0.0],
            pivot: [2.0, 0.0, 2.0],
            ..Pose::IDENTITY
        };
        assert!(close(pose.apply([2.0, 0.0, 2.0]), [2.0, 0.0, 2.0], 1e-5));
    }

    #[test]
    fn yaw_90_maps_x_to_minus_z() {
        // Right-handed, Y up: +X rotated 90° about Y lands on -Z.
        let pose = Pose {
            rotate_deg: [0.0, 90.0, 0.0],
            ..Pose::IDENTITY
        };
        assert!(close(pose.apply([1.0, 0.0, 0.0]), [0.0, 0.0, -1.0], 1e-5));
    }

    #[test]
    fn full_turn_returns_to_start() {
        let pose = Pose {
            rotate_deg: [360.0, 360.0, 360.0],
            ..Pose::IDENTITY
        };
        assert!(close(pose.apply([1.0, 2.0, 3.0]), [1.0, 2.0, 3.0], 1e-4));
    }

    #[test]
    fn normal_matrix_is_rotation_for_rigid_poses() {
        // For a pure rotation the inverse-transpose equals the rotation itself.
        let pose = Pose {
            rotate_deg: [0.0, 90.0, 0.0],
            ..Pose::IDENTITY
        };
        let n = pose.normal_matrix();
        let m = pose.to_matrix();
        for c in 0..3 {
            for r in 0..3 {
                assert!(
                    (n[c][r] - m[c][r]).abs() < 1e-4,
                    "normal matrix should match rotation at [{c}][{r}]"
                );
            }
        }
    }

    #[test]
    fn normal_matrix_compensates_non_uniform_scale() {
        // Squash in X: normals must stretch the other way, or lighting is wrong.
        let pose = Pose {
            scale: [0.5, 1.0, 1.0],
            ..Pose::IDENTITY
        };
        let n = pose.normal_matrix();
        assert!(
            (n[0][0] - 2.0).abs() < 1e-4,
            "x normal should scale by 1/0.5"
        );
        assert!((n[1][1] - 1.0).abs() < 1e-4);
    }

    /// A zero scale mid-animation (a block popping in from nothing) must not
    /// put NaNs in the shader.
    #[test]
    fn degenerate_scale_falls_back_to_identity_normals() {
        let pose = Pose {
            scale: [0.0, 0.0, 0.0],
            ..Pose::IDENTITY
        };
        let n = pose.normal_matrix();
        assert!(n.iter().flatten().all(|v| v.is_finite()));
        assert_eq!(n, [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]);
    }

    #[test]
    fn about_sets_only_the_pivot() {
        let p = Pose::about([1.0, 2.0, 3.0]);
        assert_eq!(p.pivot, [1.0, 2.0, 3.0]);
        assert_eq!(p.scale, [1.0; 3]);
        assert_eq!(p.translate, [0.0; 3]);
    }
}
