//! Camera configuration and view/projection math for rendering.

use crate::meshing::MeshOutput;

/// Camera projection mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Projection {
    /// Standard perspective projection (default).
    #[default]
    Perspective,
    /// Parallel (orthographic) projection — no perspective foreshortening.
    Orthographic,
}

/// Camera configuration for rendering.
#[derive(Clone, Debug)]
pub struct CameraConfig {
    pub yaw_deg: f32,
    pub pitch_deg: f32,
    pub zoom: f32,
    pub fov_deg: f32,
    /// Optional explicit orbit target. When set, the camera orbits and
    /// aims at this point instead of the model's bounding-box centroid.
    pub target: Option<[f32; 3]>,
    /// Projection mode.
    pub projection: Projection,
    /// Optional solid RGBA clear color (linear 0.0–1.0). `None` uses the
    /// default sky / HDRI behavior.
    pub background: Option<[f32; 4]>,
    /// Fit the camera to the model's bounding *sphere* instead of the
    /// yaw-dependent projected box corners. The sphere is rotation
    /// invariant, so orbiting the camera (turntables) keeps a constant
    /// distance instead of pulsing with the silhouette.
    pub sphere_fit: bool,
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            yaw_deg: 45.0,
            pitch_deg: 30.0,
            zoom: 1.0,
            fov_deg: 45.0,
            target: None,
            projection: Projection::Perspective,
            background: None,
            sphere_fit: false,
        }
    }
}

/// Compute merged bounding box across multiple meshes.
pub fn merged_bounds(meshes: &[MeshOutput]) -> ([f32; 3], [f32; 3]) {
    let mut min = [f32::MAX; 3];
    let mut max = [f32::MIN; 3];
    for m in meshes {
        for i in 0..3 {
            min[i] = min[i].min(m.bounds.min[i]);
            max[i] = max[i].max(m.bounds.max[i]);
        }
    }
    (min, max)
}

/// Compute view-projection and inverse view-projection matrices.
pub fn compute_view_proj(
    bounds_min: [f32; 3],
    bounds_max: [f32; 3],
    aspect: f32,
    camera: &CameraConfig,
) -> ([[f32; 4]; 4], [[f32; 4]; 4]) {
    let center = camera.target.unwrap_or([
        (bounds_min[0] + bounds_max[0]) * 0.5,
        (bounds_min[1] + bounds_max[1]) * 0.5,
        (bounds_min[2] + bounds_max[2]) * 0.5,
    ]);

    let yaw = camera.yaw_deg.to_radians();
    let pitch = camera.pitch_deg.to_radians();
    let fov = camera.fov_deg.to_radians();

    let dir = normalize3([
        -(pitch.cos() * yaw.sin()),
        -(pitch.sin()),
        -(pitch.cos() * yaw.cos()),
    ]);

    let forward = dir;
    let right = normalize3(cross3(forward, [0.0, 1.0, 0.0]));
    let up = cross3(right, forward);

    let corners = [
        [bounds_min[0], bounds_min[1], bounds_min[2]],
        [bounds_max[0], bounds_min[1], bounds_min[2]],
        [bounds_min[0], bounds_max[1], bounds_min[2]],
        [bounds_max[0], bounds_max[1], bounds_min[2]],
        [bounds_min[0], bounds_min[1], bounds_max[2]],
        [bounds_max[0], bounds_min[1], bounds_max[2]],
        [bounds_min[0], bounds_max[1], bounds_max[2]],
        [bounds_max[0], bounds_max[1], bounds_max[2]],
    ];

    let (view_proj, inv_view_proj) = match camera.projection {
        Projection::Perspective => {
            let half_fov_y = fov * 0.5;
            let half_fov_x = (half_fov_y.tan() * aspect).atan();

            let mut max_dist = 1.0f32;
            if camera.sphere_fit {
                // Rotation-invariant: fit the bounding sphere around `center`.
                let mut radius = 0.0f32;
                for c in &corners {
                    let rel = sub3(*c, center);
                    radius = radius.max(dot3(rel, rel).sqrt());
                }
                let half_min = half_fov_x.min(half_fov_y);
                max_dist = (radius / half_min.sin().max(1e-4)).max(1.0);
            } else {
                for c in &corners {
                    let rel = sub3(*c, center);
                    let proj_right = dot3(rel, right).abs();
                    let proj_up = dot3(rel, up).abs();
                    let proj_depth = -dot3(rel, forward);
                    let dist_h = proj_right / half_fov_x.tan() + proj_depth;
                    let dist_v = proj_up / half_fov_y.tan() + proj_depth;
                    max_dist = max_dist.max(dist_h).max(dist_v);
                }
            }

            // zoom is a true zoom factor: >1 moves closer, <1 further out.
            let distance = max_dist * 1.1 / camera.zoom.max(1e-3);
            let eye = [
                center[0] - dir[0] * distance,
                center[1] - dir[1] * distance,
                center[2] - dir[2] * distance,
            ];

            let view = look_at(eye, center, [0.0, 1.0, 0.0]);
            let near = distance * 0.01;
            let far = distance * 10.0;
            let proj = perspective(fov, aspect, near, far);
            let view_proj = mat4_mul(proj, view);
            (view_proj, mat4_inverse(view_proj))
        }
        Projection::Orthographic => {
            let mut ext_h = 0.0f32;
            let mut ext_v = 0.0f32;
            let mut ext_depth = 0.0f32;
            for c in &corners {
                let rel = sub3(*c, center);
                ext_h = ext_h.max(dot3(rel, right).abs());
                ext_v = ext_v.max(dot3(rel, up).abs());
                ext_depth = ext_depth.max(dot3(rel, forward).abs());
            }

            // Half-extents of the ortho window, fitting both axes, scaled by
            // zoom (a true zoom factor: >1 magnifies, <1 shrinks). Sphere
            // fit swaps the yaw-dependent extents for the rotation-invariant
            // bounding-sphere radius so turntables hold a constant framing.
            let fitted = if camera.sphere_fit {
                let mut radius = 0.0f32;
                for c in &corners {
                    let rel = sub3(*c, center);
                    radius = radius.max(dot3(rel, rel).sqrt());
                }
                (radius / aspect.min(1.0)).max(0.5)
            } else {
                (ext_v.max(ext_h / aspect)).max(0.5)
            };
            let half_h = fitted * 1.1 / camera.zoom.max(1e-3);
            let half_w = half_h * aspect;

            // Stand far enough back that all geometry sits between near and far.
            let standoff = ext_depth + ext_h + ext_v + 1.0;
            let eye = [
                center[0] - dir[0] * standoff,
                center[1] - dir[1] * standoff,
                center[2] - dir[2] * standoff,
            ];

            let view = look_at(eye, center, [0.0, 1.0, 0.0]);
            let near = 0.01;
            let far = standoff * 2.0 + 1.0;
            let proj = ortho(-half_w, half_w, -half_h, half_h, near, far);
            let view_proj = mat4_mul(proj, view);
            (view_proj, mat4_inverse(view_proj))
        }
    };

    (view_proj, inv_view_proj)
}

pub fn look_at(eye: [f32; 3], target: [f32; 3], up: [f32; 3]) -> [[f32; 4]; 4] {
    let f = normalize3(sub3(target, eye));
    let s = normalize3(cross3(f, up));
    let u = cross3(s, f);
    [
        [s[0], u[0], -f[0], 0.0],
        [s[1], u[1], -f[1], 0.0],
        [s[2], u[2], -f[2], 0.0],
        [-dot3(s, eye), -dot3(u, eye), dot3(f, eye), 1.0],
    ]
}

pub fn perspective(fov_y: f32, aspect: f32, near: f32, far: f32) -> [[f32; 4]; 4] {
    let f = 1.0 / (fov_y * 0.5).tan();
    let nf = 1.0 / (near - far);
    [
        [f / aspect, 0.0, 0.0, 0.0],
        [0.0, f, 0.0, 0.0],
        [0.0, 0.0, far * nf, -1.0],
        [0.0, 0.0, near * far * nf, 0.0],
    ]
}

/// Orthographic projection matrix matching the wgpu NDC convention (z in
/// [0, 1]) and the same right-handed, looking-down-`-z` view space as
/// [`perspective`]. Column-major storage to match the rest of this module.
pub fn ortho(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> [[f32; 4]; 4] {
    let rl = 1.0 / (right - left);
    let tb = 1.0 / (top - bottom);
    let nf = 1.0 / (near - far);
    [
        [2.0 * rl, 0.0, 0.0, 0.0],
        [0.0, 2.0 * tb, 0.0, 0.0],
        [0.0, 0.0, nf, 0.0],
        [-(right + left) * rl, -(top + bottom) * tb, near * nf, 1.0],
    ]
}

pub fn mat4_mul(a: [[f32; 4]; 4], b: [[f32; 4]; 4]) -> [[f32; 4]; 4] {
    let mut out = [[0.0f32; 4]; 4];
    for i in 0..4 {
        for j in 0..4 {
            out[i][j] =
                a[0][j] * b[i][0] + a[1][j] * b[i][1] + a[2][j] * b[i][2] + a[3][j] * b[i][3];
        }
    }
    out
}

/// 4x4 matrix inverse (general, cofactor expansion).
pub fn mat4_inverse(m: [[f32; 4]; 4]) -> [[f32; 4]; 4] {
    let m00 = m[0][0];
    let m01 = m[0][1];
    let m02 = m[0][2];
    let m03 = m[0][3];
    let m10 = m[1][0];
    let m11 = m[1][1];
    let m12 = m[1][2];
    let m13 = m[1][3];
    let m20 = m[2][0];
    let m21 = m[2][1];
    let m22 = m[2][2];
    let m23 = m[2][3];
    let m30 = m[3][0];
    let m31 = m[3][1];
    let m32 = m[3][2];
    let m33 = m[3][3];

    let a2323 = m22 * m33 - m23 * m32;
    let a1323 = m21 * m33 - m23 * m31;
    let a1223 = m21 * m32 - m22 * m31;
    let a0323 = m20 * m33 - m23 * m30;
    let a0223 = m20 * m32 - m22 * m30;
    let a0123 = m20 * m31 - m21 * m30;
    let a2313 = m12 * m33 - m13 * m32;
    let a1313 = m11 * m33 - m13 * m31;
    let a1213 = m11 * m32 - m12 * m31;
    let a2312 = m12 * m23 - m13 * m22;
    let a1312 = m11 * m23 - m13 * m21;
    let a1212 = m11 * m22 - m12 * m21;
    let a0313 = m10 * m33 - m13 * m30;
    let a0213 = m10 * m32 - m12 * m30;
    let a0312 = m10 * m23 - m13 * m20;
    let a0212 = m10 * m22 - m12 * m20;
    let a0113 = m10 * m31 - m11 * m30;
    let a0112 = m10 * m21 - m11 * m20;

    let det = m00 * (m11 * a2323 - m12 * a1323 + m13 * a1223)
        - m01 * (m10 * a2323 - m12 * a0323 + m13 * a0223)
        + m02 * (m10 * a1323 - m11 * a0323 + m13 * a0123)
        - m03 * (m10 * a1223 - m11 * a0223 + m12 * a0123);

    let inv_det = 1.0 / det;

    [
        [
            inv_det * (m11 * a2323 - m12 * a1323 + m13 * a1223),
            inv_det * -(m01 * a2323 - m02 * a1323 + m03 * a1223),
            inv_det * (m01 * a2313 - m02 * a1313 + m03 * a1213),
            inv_det * -(m01 * a2312 - m02 * a1312 + m03 * a1212),
        ],
        [
            inv_det * -(m10 * a2323 - m12 * a0323 + m13 * a0223),
            inv_det * (m00 * a2323 - m02 * a0323 + m03 * a0223),
            inv_det * -(m00 * a2313 - m02 * a0313 + m03 * a0213),
            inv_det * (m00 * a2312 - m02 * a0312 + m03 * a0212),
        ],
        [
            inv_det * (m10 * a1323 - m11 * a0323 + m13 * a0123),
            inv_det * -(m00 * a1323 - m01 * a0323 + m03 * a0123),
            inv_det * (m00 * a1313 - m01 * a0313 + m03 * a0113),
            inv_det * -(m00 * a1312 - m01 * a0312 + m03 * a0112),
        ],
        [
            inv_det * -(m10 * a1223 - m11 * a0223 + m12 * a0123),
            inv_det * (m00 * a1223 - m01 * a0223 + m02 * a0123),
            inv_det * -(m00 * a1213 - m01 * a0213 + m02 * a0113),
            inv_det * (m00 * a1212 - m01 * a0212 + m02 * a0112),
        ],
    ]
}

pub fn sub3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

pub fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

pub fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

pub fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let len = dot3(v, v).sqrt();
    if len < 1e-10 {
        return [0.0, 0.0, 0.0];
    }
    [v[0] / len, v[1] / len, v[2] / len]
}

#[cfg(test)]
mod ortho_tests {
    use super::*;

    // Matrices are column-major: ndc[j] = Σ_i m[i][j] * v[i].
    fn transform(m: [[f32; 4]; 4], v: [f32; 4]) -> [f32; 4] {
        let mut out = [0.0f32; 4];
        for j in 0..4 {
            out[j] = m[0][j] * v[0] + m[1][j] * v[1] + m[2][j] * v[2] + m[3][j] * v[3];
        }
        out
    }

    #[test]
    fn ortho_maps_box_to_ndc() {
        // left/right/bottom/top = ±1, near=1, far=3. View-space looks down -z,
        // so view z = -near maps to NDC z 0, view z = -far maps to NDC z 1.
        let m = ortho(-1.0, 1.0, -1.0, 1.0, 1.0, 3.0);
        let ndc = transform(m, [0.5, -0.5, -2.0, 1.0]);
        assert!((ndc[0] - 0.5).abs() < 1e-5, "x={}", ndc[0]);
        assert!((ndc[1] + 0.5).abs() < 1e-5, "y={}", ndc[1]);
        assert!((ndc[2] - 0.5).abs() < 1e-5, "z={}", ndc[2]); // mid-depth
        assert!((ndc[3] - 1.0).abs() < 1e-5, "w={}", ndc[3]); // no perspective divide
    }

    #[test]
    fn ortho_near_and_far_planes() {
        let m = ortho(-2.0, 2.0, -2.0, 2.0, 1.0, 5.0);
        let near = transform(m, [0.0, 0.0, -1.0, 1.0]);
        let far = transform(m, [0.0, 0.0, -5.0, 1.0]);
        assert!((near[2] - 0.0).abs() < 1e-5, "near z={}", near[2]);
        assert!((far[2] - 1.0).abs() < 1e-5, "far z={}", far[2]);
    }
}

#[cfg(test)]
mod view_proj_tests {
    use super::*;

    fn transform(m: [[f32; 4]; 4], v: [f32; 4]) -> [f32; 4] {
        let mut out = [0.0f32; 4];
        for j in 0..4 {
            out[j] = m[0][j] * v[0] + m[1][j] * v[1] + m[2][j] * v[2] + m[3][j] * v[3];
        }
        out
    }

    #[test]
    fn orthographic_fits_all_corners_in_ndc() {
        let cam = CameraConfig {
            yaw_deg: 30.0,
            pitch_deg: 25.0,
            zoom: 1.0,
            fov_deg: 45.0,
            target: None,
            projection: Projection::Orthographic,
            background: None,
            sphere_fit: false,
        };
        let (vp, _) = compute_view_proj([0.0, 0.0, 0.0], [4.0, 2.0, 6.0], 16.0 / 9.0, &cam);

        let corners = [
            [0.0, 0.0, 0.0],
            [4.0, 0.0, 0.0],
            [0.0, 2.0, 0.0],
            [4.0, 2.0, 0.0],
            [0.0, 0.0, 6.0],
            [4.0, 0.0, 6.0],
            [0.0, 2.0, 6.0],
            [4.0, 2.0, 6.0],
        ];
        let mut max_xy = 0.0f32;
        for c in &corners {
            let ndc = transform(vp, [c[0], c[1], c[2], 1.0]);
            // Orthographic: w stays 1 (no perspective divide).
            assert!((ndc[3] - 1.0).abs() < 1e-4, "w={}", ndc[3]);
            // All geometry inside the clip box.
            assert!(ndc[0].abs() <= 1.05, "x out of range: {}", ndc[0]);
            assert!(ndc[1].abs() <= 1.05, "y out of range: {}", ndc[1]);
            assert!(
                ndc[2] >= -0.001 && ndc[2] <= 1.001,
                "z out of range: {}",
                ndc[2]
            );
            max_xy = max_xy.max(ndc[0].abs()).max(ndc[1].abs());
        }
        // The framing should roughly fill the viewport (1/1.1 ≈ 0.9).
        assert!(max_xy > 0.8, "geometry too small in frame: {}", max_xy);
    }
}
