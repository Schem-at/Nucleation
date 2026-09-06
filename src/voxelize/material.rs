//! glTF surface appearance sampled in linear light. Alpha coverage is separate
//! from transmission: a cutout is a hole, glass is a visible surface.
use super::model::{MeshTriangle, TextureImage};
use super::shape::{barycentric, closest_point_on_triangle};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum AlphaMode {
    #[default]
    Opaque,
    Mask(f32),
    Blend,
}

#[derive(Clone)]
pub struct MaterialTexture {
    pub image: Arc<TextureImage>,
    pub tex_coord: u32,
    pub wrap_s: gltf::texture::WrappingMode,
    pub wrap_t: gltf::texture::WrappingMode,
}

impl MaterialTexture {
    /// RGB is sRGB for base/emissive textures; transmission is linear data.
    fn sample(&self, uv: [f32; 2], srgb: bool) -> [f32; 4] {
        use gltf::texture::WrappingMode::*;
        let wrap = |i: i32, n: i32, mode| match mode {
            ClampToEdge => i.clamp(0, n - 1),
            MirroredRepeat => {
                let i = i.rem_euclid(2 * n);
                if i < n {
                    i
                } else {
                    2 * n - 1 - i
                }
            }
            Repeat => i.rem_euclid(n),
        };
        let image = &self.image;
        let x = uv[0] * image.width as f32 - 0.5;
        let y = uv[1] * image.height as f32 - 0.5;
        let (ix, iy) = (x.floor() as i32, y.floor() as i32);
        let (fx, fy) = (x - x.floor(), y - y.floor());
        let mut out = [0.0; 4];
        for (dx, dy, weight) in [
            (0, 0, (1.0 - fx) * (1.0 - fy)),
            (1, 0, fx * (1.0 - fy)),
            (0, 1, (1.0 - fx) * fy),
            (1, 1, fx * fy),
        ] {
            let px = wrap(ix + dx, image.width as i32, self.wrap_s) as usize;
            let py = wrap(iy + dy, image.height as i32, self.wrap_t) as usize;
            for (c, value) in out.iter_mut().enumerate() {
                let v = image.pixels[(py * image.width as usize + px) * 4 + c] as f32 / 255.0;
                *value += weight * if srgb && c < 3 { srgb_to_linear(v) } else { v };
            }
        }
        out
    }
}

#[derive(Clone)]
pub struct MeshMaterial {
    pub base_texture: Option<MaterialTexture>,
    pub base_factor: [f32; 4],
    pub alpha_mode: AlphaMode,
    pub transmission: f32,
    pub transmission_texture: Option<MaterialTexture>,
    pub emissive_factor: [f32; 3],
    pub emissive_texture: Option<MaterialTexture>,
}

impl Default for MeshMaterial {
    fn default() -> Self {
        Self {
            base_texture: None,
            base_factor: [1.0; 4],
            alpha_mode: AlphaMode::Opaque,
            transmission: 0.0,
            transmission_texture: None,
            emissive_factor: [0.0; 3],
            emissive_texture: None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SurfaceSample {
    pub base: [f32; 3],
    pub emissive: [f32; 3],
    pub translucent: bool,
    pub visible: bool,
    pub textured: bool,
}
impl Default for SurfaceSample {
    fn default() -> Self {
        Self {
            base: [srgb_to_linear(208.0 / 255.0); 3],
            emissive: [0.0; 3],
            translucent: false,
            visible: true,
            textured: false,
        }
    }
}
impl SurfaceSample {
    /// Bake emission after directional lighting, so the eye does not turn
    /// black on the unlit side. Clamp HDR emission to the export's sRGB gamut.
    pub fn rgb(&self, brightness: f32) -> [u8; 3] {
        std::array::from_fn(|c| linear_to_srgb(self.base[c] * brightness + self.emissive[c]))
    }
}

pub(super) fn sample_triangle(
    materials: &[Option<MeshMaterial>],
    tri: &MeshTriangle,
    p: [f32; 3],
) -> SurfaceSample {
    let Some(mat) = tri
        .material
        .and_then(|i| materials.get(i as usize))
        .and_then(Option::as_ref)
    else {
        return SurfaceSample::default();
    };
    let (a, b, c) = barycentric(closest_point_on_triangle(p, &tri.positions), &tri.positions);
    let weights = [a, b, c];
    let uv = |coords: Option<[[f32; 2]; 3]>| {
        coords.map(|coords| {
            std::array::from_fn(|axis| (0..3).map(|i| coords[i][axis] * weights[i]).sum())
        })
    };
    let tex = |texture: &Option<MaterialTexture>, coords, srgb| {
        texture
            .as_ref()
            .zip(uv(coords))
            .map(|(t, uv)| t.sample(uv, srgb))
            .unwrap_or([1.0; 4])
    };
    let base = tex(&mat.base_texture, tri.uvs, true);
    let vertex: [f32; 4] = tri
        .colors
        .map(|colors| {
            std::array::from_fn(|channel| (0..3).map(|i| colors[i][channel] * weights[i]).sum())
        })
        .unwrap_or([1.0; 4]);
    let alpha = (base[3] * mat.base_factor[3] * vertex[3]).clamp(0.0, 1.0);
    let visible = match mat.alpha_mode {
        AlphaMode::Opaque => true,
        AlphaMode::Mask(cutoff) => alpha >= cutoff,
        AlphaMode::Blend => alpha > 1.0 / 255.0,
    };
    let transmission =
        mat.transmission * tex(&mat.transmission_texture, tri.transmission_uvs, false)[0];
    let translucent = transmission > 0.01 || (mat.alpha_mode == AlphaMode::Blend && alpha < 0.99);
    let emission = tex(&mat.emissive_texture, tri.emissive_uvs, true);
    SurfaceSample {
        base: std::array::from_fn(|c| base[c] * mat.base_factor[c] * vertex[c]),
        emissive: std::array::from_fn(|c| emission[c] * mat.emissive_factor[c]),
        translucent,
        visible,
        textured: true,
    }
}

pub(super) fn srgb_to_linear(v: f32) -> f32 {
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}
fn linear_to_srgb(v: f32) -> u8 {
    let v = v.clamp(0.0, 1.0);
    let s = if v <= 0.0031308 {
        12.92 * v
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    };
    (s * 255.0).round().clamp(0.0, 255.0) as u8
}

#[cfg(test)]
mod tests {
    use super::*;
    fn triangle() -> MeshTriangle {
        MeshTriangle {
            positions: [[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 2.0, 0.0]],
            uvs: Some([[0.5; 2]; 3]),
            emissive_uvs: Some([[0.5; 2]; 3]),
            transmission_uvs: Some([[0.5; 2]; 3]),
            colors: None,
            material: Some(0),
        }
    }
    fn texture(rgba: [u8; 4]) -> MaterialTexture {
        MaterialTexture {
            image: Arc::new(TextureImage {
                width: 1,
                height: 1,
                pixels: rgba.to_vec(),
            }),
            tex_coord: 0,
            wrap_s: gltf::texture::WrappingMode::Repeat,
            wrap_t: gltf::texture::WrappingMode::Repeat,
        }
    }
    fn sample(mat: MeshMaterial) -> SurfaceSample {
        sample_triangle(&[Some(mat)], &triangle(), [0.5, 0.5, 0.5])
    }
    #[test]
    fn alpha_coverage_does_not_confuse_a_hole_with_glass() {
        let mut mat = MeshMaterial {
            base_texture: Some(texture([200, 100, 50, 0])),
            ..Default::default()
        };
        assert!(sample(mat.clone()).visible, "OPAQUE ignores texture alpha");
        assert!(!sample(mat.clone()).translucent);
        mat.alpha_mode = AlphaMode::Mask(0.5);
        assert!(!sample(mat.clone()).visible);
        mat.base_texture = Some(texture([200, 100, 50, 128]));
        assert!(sample(mat.clone()).visible);
        assert!(
            !sample(mat.clone()).translucent,
            "surviving MASK texels are opaque"
        );
        mat.alpha_mode = AlphaMode::Blend;
        assert!(sample(mat.clone()).visible && sample(mat.clone()).translucent);
        mat.base_factor[3] = 0.0;
        assert!(!sample(mat).visible);
    }
    #[test]
    fn base_colour_factor_and_vertex_colour_multiply_in_linear_light() {
        let mat = MeshMaterial {
            base_texture: Some(texture([128, 128, 128, 255])),
            base_factor: [0.5, 1.0, 1.0, 1.0],
            ..Default::default()
        };
        let mut tri = triangle();
        tri.colors = Some([[1.0, 0.5, 0.0, 1.0]; 3]);
        assert_eq!(
            sample_triangle(&[Some(mat)], &tri, [0.5, 0.5, 0.5]).rgb(1.0),
            [92, 92, 0]
        );
    }
    #[test]
    fn transmission_uses_its_linear_red_channel_and_does_not_need_blend() {
        let mut mat = MeshMaterial {
            transmission: 1.0,
            transmission_texture: Some(texture([0, 255, 255, 255])),
            ..Default::default()
        };
        assert!(!sample(mat.clone()).translucent);
        mat.transmission_texture = Some(texture([255, 0, 0, 255]));
        let s = sample(mat);
        assert!(s.visible && s.translucent);
    }
    #[test]
    fn emissive_texture_survives_zero_incident_light() {
        let mat = MeshMaterial {
            base_factor: [0.0, 0.0, 0.0, 1.0],
            emissive_factor: [2.0; 3],
            emissive_texture: Some(texture([0, 128, 255, 255])),
            ..Default::default()
        };
        assert_eq!(sample(mat).rgb(0.0), [0, 176, 255]);
    }
    #[test]
    fn sampler_wraps_both_sides_of_a_texture_seam() {
        let mut t = texture([0, 0, 0, 255]);
        t.image = Arc::new(TextureImage {
            width: 2,
            height: 1,
            pixels: vec![0, 0, 0, 255, 255, 255, 255, 255],
        });
        assert!((t.sample([0.0, 0.5], true)[0] - 0.5).abs() < 1e-6);
        t.wrap_s = gltf::texture::WrappingMode::ClampToEdge;
        assert_eq!(t.sample([0.0, 0.5], true)[0], 0.0);
        t.wrap_s = gltf::texture::WrappingMode::MirroredRepeat;
        assert_eq!(t.sample([-0.25, 0.5], true)[0], 0.0);
    }
}
