//! Mesh model loading: GLB (glTF binary) and minimal OBJ, plus the `fit`
//! normalization that maps a model into voxel space.

use super::material::{AlphaMode, MaterialTexture, MeshMaterial};
use std::borrow::Cow;
use std::sync::Arc;

/// A decoded RGBA8 texture image.
#[derive(Clone)]
pub struct TextureImage {
    pub width: u32,
    pub height: u32,
    /// RGBA8, row-major, `width * height * 4` bytes.
    pub pixels: Vec<u8>,
}

impl TextureImage {
    /// Bilinear sample at (u, v) with repeat wrapping. Returns RGB.
    pub fn sample_bilinear(&self, u: f32, v: f32) -> [u8; 3] {
        let w = self.width as f32;
        let h = self.height as f32;
        // Repeat wrap into [0, 1).
        let u = u - u.floor();
        let v = v - v.floor();
        // Texel-center convention: uv 0..1 spans the full image.
        let x = (u * w - 0.5).max(0.0);
        let y = (v * h - 0.5).max(0.0);
        let x0 = x.floor() as u32;
        let y0 = y.floor() as u32;
        let x1 = (x0 + 1).min(self.width - 1);
        let y1 = (y0 + 1).min(self.height - 1);
        let fx = x - x0 as f32;
        let fy = y - y0 as f32;
        let texel = |px: u32, py: u32| -> [f32; 3] {
            let i = ((py * self.width + px) * 4) as usize;
            [
                self.pixels[i] as f32,
                self.pixels[i + 1] as f32,
                self.pixels[i + 2] as f32,
            ]
        };
        let c00 = texel(x0, y0);
        let c10 = texel(x1, y0);
        let c01 = texel(x0, y1);
        let c11 = texel(x1, y1);
        let mut out = [0u8; 3];
        for (i, o) in out.iter_mut().enumerate() {
            let top = c00[i] * (1.0 - fx) + c10[i] * fx;
            let bot = c01[i] * (1.0 - fx) + c11[i] * fx;
            *o = (top * (1.0 - fy) + bot * fy).round().clamp(0.0, 255.0) as u8;
        }
        out
    }
}

/// One triangle in model space, with optional per-vertex UVs and material.
#[derive(Clone, Copy)]
pub struct MeshTriangle {
    pub positions: [[f32; 3]; 3],
    pub uvs: Option<[[f32; 2]; 3]>,
    pub emissive_uvs: Option<[[f32; 2]; 3]>,
    pub transmission_uvs: Option<[[f32; 2]; 3]>,
    /// Linear vertex colour, multiplied by the material base colour.
    pub colors: Option<[[f32; 4]; 3]>,
    /// Index into [`MeshModel::materials`].
    pub material: Option<u32>,
}

/// Triangles in model space plus glTF surface materials and shared textures.
///
/// Load with [`MeshModel::from_glb_bytes`] / [`MeshModel::from_obj_str`], then
/// normalize into voxel space with [`MeshModel::fit`].
#[derive(Clone)]
pub struct MeshModel {
    pub triangles: Vec<MeshTriangle>,
    /// One slot per glTF material, followed by the implicit default material.
    pub materials: Vec<Option<MeshMaterial>>,
}

impl MeshModel {
    /// Axis-aligned bounding box over all triangle vertices.
    /// `None` for an empty model.
    pub fn aabb(&self) -> Option<([f32; 3], [f32; 3])> {
        let mut min = [f32::INFINITY; 3];
        let mut max = [f32::NEG_INFINITY; 3];
        for tri in &self.triangles {
            for p in &tri.positions {
                for a in 0..3 {
                    min[a] = min[a].min(p[a]);
                    max[a] = max[a].max(p[a]);
                }
            }
        }
        if self.triangles.is_empty() {
            None
        } else {
            Some((min, max))
        }
    }

    /// Uniform-scale and translate the model so its largest dimension equals
    /// `target_size`, centered on x/z (midpoint at x = 0, z = 0) with the base
    /// resting at y = 0.
    pub fn fit(&mut self, target_size: f32) {
        let Some((min, max)) = self.aabb() else {
            return;
        };
        let extent = [max[0] - min[0], max[1] - min[1], max[2] - min[2]];
        let largest = extent[0].max(extent[1]).max(extent[2]);
        let scale = if largest > 1e-12 {
            target_size / largest
        } else {
            1.0
        };
        let anchor = [(min[0] + max[0]) * 0.5, min[1], (min[2] + max[2]) * 0.5];
        for tri in &mut self.triangles {
            for p in &mut tri.positions {
                for a in 0..3 {
                    p[a] = (p[a] - anchor[a]) * scale;
                }
            }
        }
    }

    /// Parse a binary glTF (`.glb`) with embedded buffers/images: full node
    /// hierarchy traversal with transforms applied, all triangle-mode
    /// primitives (non-triangle modes are ignored), default-pose skinning, and
    /// base colour, alpha coverage, emission and transmission materials.
    pub fn from_glb_bytes(data: &[u8]) -> Result<MeshModel, String> {
        let gltf = gltf::Gltf::from_slice(data).map_err(|e| format!("GLB parse error: {e}"))?;
        let doc = gltf.document;
        let blob = gltf.blob;

        // Resolve buffers: BIN chunk or embedded data: URIs only.
        let mut buffers: Vec<Cow<'_, [u8]>> = Vec::with_capacity(doc.buffers().count());
        for buffer in doc.buffers() {
            let data = match buffer.source() {
                gltf::buffer::Source::Bin => Cow::Borrowed(
                    blob.as_deref()
                        .ok_or_else(|| "GLB references BIN chunk but has none".to_string())?,
                ),
                gltf::buffer::Source::Uri(uri) => Cow::Owned(
                    decode_data_uri(uri)
                        .ok_or_else(|| format!("unsupported external buffer URI in GLB: {uri}"))?,
                ),
            };
            if data.len() < buffer.length() {
                return Err(format!(
                    "buffer {} too short: {} < {}",
                    buffer.index(),
                    data.len(),
                    buffer.length()
                ));
            }
            buffers.push(data);
        }

        // Decode images (best-effort: an undecodable image just loses its texture).
        let mut images: Vec<Option<Arc<TextureImage>>> = Vec::with_capacity(doc.images().count());
        for img in doc.images() {
            let bytes: Option<Cow<'_, [u8]>> = match img.source() {
                gltf::image::Source::View { view, .. } => {
                    let buf = &buffers[view.buffer().index()];
                    buf.get(view.offset()..view.offset() + view.length())
                        .map(Cow::Borrowed)
                }
                gltf::image::Source::Uri { uri, .. } => decode_data_uri(uri).map(Cow::Owned),
            };
            let decoded = bytes
                .and_then(|b| image::load_from_memory(&b).ok())
                .map(|d| {
                    let rgba = d.into_rgba8();
                    Arc::new(TextureImage {
                        width: rgba.width(),
                        height: rgba.height(),
                        pixels: rgba.into_raw(),
                    })
                });
            images.push(decoded);
        }

        let texture = |info: gltf::texture::Info| -> Option<MaterialTexture> {
            let t = info.texture();
            Some(MaterialTexture {
                image: images[t.source().index()].clone()?,
                tex_coord: info.tex_coord(),
                wrap_s: t.sampler().wrap_s(),
                wrap_t: t.sampler().wrap_t(),
            })
        };
        let mut materials: Vec<Option<MeshMaterial>> = doc
            .materials()
            .map(|mat| {
                let pbr = mat.pbr_metallic_roughness();
                let transmission = mat.transmission();
                Some(MeshMaterial {
                    base_texture: pbr.base_color_texture().and_then(&texture),
                    base_factor: pbr.base_color_factor(),
                    alpha_mode: match mat.alpha_mode() {
                        gltf::material::AlphaMode::Opaque => AlphaMode::Opaque,
                        gltf::material::AlphaMode::Mask => {
                            AlphaMode::Mask(mat.alpha_cutoff().unwrap_or(0.5))
                        }
                        gltf::material::AlphaMode::Blend => AlphaMode::Blend,
                    },
                    transmission: transmission
                        .as_ref()
                        .map_or(0.0, |t| t.transmission_factor()),
                    transmission_texture: transmission
                        .and_then(|t| t.transmission_texture())
                        .and_then(&texture),
                    emissive_factor: mat
                        .emissive_factor()
                        .map(|v| v * mat.emissive_strength().unwrap_or(1.0)),
                    emissive_texture: mat.emissive_texture().and_then(&texture),
                })
            })
            .collect();

        materials.push(Some(MeshMaterial::default()));
        let mut triangles = Vec::new();
        // A glTF can contain alternative scenes; import its default scene only.
        let roots: Vec<_> = if let Some(scene) = doc.default_scene().or_else(|| doc.scenes().next())
        {
            scene.nodes().collect()
        } else {
            let children: std::collections::HashSet<_> = doc
                .nodes()
                .flat_map(|n| n.children().map(|c| c.index()).collect::<Vec<_>>())
                .collect();
            doc.nodes()
                .filter(|n| !children.contains(&n.index()))
                .collect()
        };
        let mut worlds = vec![IDENTITY; doc.nodes().count()];
        for node in &roots {
            node_worlds(node, IDENTITY, &mut worlds);
        }
        for node in &roots {
            visit_node(node, &worlds, &buffers, &materials, &mut triangles);
        }

        drop_degenerate_triangles(&mut triangles, "GLB");
        if triangles.is_empty() {
            return Err("GLB contains no triangles".to_string());
        }
        Ok(MeshModel {
            triangles,
            materials,
        })
    }

    /// Minimal OBJ parser: `v`/`vt`/`f` lines only, polygon faces are
    /// fan-triangulated, negative (relative) indices supported. No materials.
    pub fn from_obj_str(text: &str) -> Result<MeshModel, String> {
        let mut positions: Vec<[f32; 3]> = Vec::new();
        let mut texcoords: Vec<[f32; 2]> = Vec::new();
        let mut triangles: Vec<MeshTriangle> = Vec::new();

        for (line_no, raw) in text.lines().enumerate() {
            let line = raw.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            let mut it = line.split_whitespace();
            let tag = it.next().unwrap();
            let err = |msg: &str| format!("OBJ line {}: {}", line_no + 1, msg);
            match tag {
                "v" => {
                    let mut p = [0f32; 3];
                    for slot in &mut p {
                        *slot = it
                            .next()
                            .and_then(|t| t.parse().ok())
                            .ok_or_else(|| err("bad vertex"))?;
                    }
                    positions.push(p);
                }
                "vt" => {
                    let u: f32 = it
                        .next()
                        .and_then(|t| t.parse().ok())
                        .ok_or_else(|| err("bad texcoord"))?;
                    let v: f32 = it.next().and_then(|t| t.parse().ok()).unwrap_or(0.0);
                    texcoords.push([u, v]);
                }
                "f" => {
                    let mut verts: Vec<(usize, Option<usize>)> = Vec::new();
                    for tok in it {
                        let mut parts = tok.split('/');
                        let vi = parts
                            .next()
                            .and_then(|t| resolve_index(t, positions.len()))
                            .ok_or_else(|| err("bad face index"))?;
                        let ti = parts.next().and_then(|t| resolve_index(t, texcoords.len()));
                        verts.push((vi, ti));
                    }
                    if verts.len() < 3 {
                        return Err(err("face with fewer than 3 vertices"));
                    }
                    for i in 1..verts.len() - 1 {
                        let corners = [verts[0], verts[i], verts[i + 1]];
                        let uvs = if corners.iter().all(|(_, t)| t.is_some()) {
                            Some([
                                texcoords[corners[0].1.unwrap()],
                                texcoords[corners[1].1.unwrap()],
                                texcoords[corners[2].1.unwrap()],
                            ])
                        } else {
                            None
                        };
                        triangles.push(MeshTriangle {
                            positions: [
                                positions[corners[0].0],
                                positions[corners[1].0],
                                positions[corners[2].0],
                            ],
                            uvs,
                            emissive_uvs: None,
                            transmission_uvs: None,
                            colors: None,
                            material: None,
                        });
                    }
                }
                _ => {} // vn, o, g, s, usemtl, mtllib, ... ignored
            }
        }

        drop_degenerate_triangles(&mut triangles, "OBJ");
        if triangles.is_empty() {
            return Err("OBJ contains no triangles".to_string());
        }
        Ok(MeshModel {
            triangles,
            materials: Vec::new(),
        })
    }
}

/// Smallest triangle area a loader keeps. Below this the triangle is
/// geometrically degenerate: two of its vertices coincide, or all three are
/// collinear.
const MIN_TRIANGLE_AREA: f64 = 1e-12;

/// Twice the area of a triangle, in f64 so a sliver whose f32 cross product
/// would flush to zero is still measured honestly.
fn double_area(positions: &[[f32; 3]; 3]) -> f64 {
    let p = |i: usize, a: usize| positions[i][a] as f64;
    let e1 = [p(1, 0) - p(0, 0), p(1, 1) - p(0, 1), p(1, 2) - p(0, 2)];
    let e2 = [p(2, 0) - p(0, 0), p(2, 1) - p(0, 1), p(2, 2) - p(0, 2)];
    let n = [
        e1[1] * e2[2] - e1[2] * e2[1],
        e1[2] * e2[0] - e1[0] * e2[2],
        e1[0] * e2[1] - e1[1] * e2[0],
    ];
    (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt()
}

/// Drop zero-area triangles at load, returning how many went.
///
/// A degenerate triangle poisons every query that touches it: the closest
/// point on it is computed by dividing by its area, so the answer is NaN, and
/// NaN loses no comparison. The nearest-triangle search would then keep it as
/// its best candidate forever and never take its early out, turning one bad
/// triangle into a full walk of the mesh per voxel. They carry no surface, so
/// dropping them changes no geometry.
fn drop_degenerate_triangles(triangles: &mut Vec<MeshTriangle>, source: &str) -> usize {
    let before = triangles.len();
    triangles.retain(|t| double_area(&t.positions) * 0.5 > MIN_TRIANGLE_AREA);
    let dropped = before - triangles.len();
    if dropped > 0 {
        log::debug!(
            "{source} load: dropped {dropped} degenerate triangle(s) of {before} \
             (area at or below {MIN_TRIANGLE_AREA:e})"
        );
    }
    dropped
}

/// OBJ index token → 0-based index (`1`-based positives, negative = relative
/// to the end of the list so far). `None` on empty/invalid/out-of-range.
fn resolve_index(token: &str, len: usize) -> Option<usize> {
    if token.is_empty() {
        return None;
    }
    let i: i64 = token.parse().ok()?;
    let idx = if i > 0 {
        i - 1
    } else if i < 0 {
        len as i64 + i
    } else {
        return None;
    };
    (0..len as i64).contains(&idx).then_some(idx as usize)
}

type Mat4 = [[f32; 4]; 4];

const IDENTITY: Mat4 = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

/// Column-major (glTF convention) matrix product `a * b`.
fn mat_mul(a: &Mat4, b: &Mat4) -> Mat4 {
    let mut out = [[0f32; 4]; 4];
    for (col, out_col) in out.iter_mut().enumerate() {
        for row in 0..4 {
            let mut acc = 0.0;
            for k in 0..4 {
                acc += a[k][row] * b[col][k];
            }
            out_col[row] = acc;
        }
    }
    out
}

fn transform_point(m: &Mat4, p: [f32; 3]) -> [f32; 3] {
    let mut out = [0f32; 3];
    for (row, o) in out.iter_mut().enumerate() {
        *o = m[0][row] * p[0] + m[1][row] * p[1] + m[2][row] * p[2] + m[3][row];
    }
    out
}

fn node_worlds(node: &gltf::Node, parent: Mat4, worlds: &mut [Mat4]) {
    let world = mat_mul(&parent, &node.transform().matrix());
    worlds[node.index()] = world;
    for child in node.children() {
        node_worlds(&child, world, worlds);
    }
}

fn visit_node(
    node: &gltf::Node,
    worlds: &[Mat4],
    buffers: &[Cow<'_, [u8]>],
    materials: &[Option<MeshMaterial>],
    triangles: &mut Vec<MeshTriangle>,
) {
    let world = worlds[node.index()];
    let joints: Option<Vec<Mat4>> = node.skin().map(|skin| {
        let inverse: Vec<_> = skin
            .reader(|b| buffers.get(b.index()).map(AsRef::as_ref))
            .read_inverse_bind_matrices()
            .map(Iterator::collect)
            .unwrap_or_else(|| vec![IDENTITY; skin.joints().count()]);
        skin.joints()
            .enumerate()
            .map(|(i, j)| mat_mul(&worlds[j.index()], inverse.get(i).unwrap_or(&IDENTITY)))
            .collect()
    });
    if let Some(mesh) = node.mesh() {
        for prim in mesh.primitives() {
            if prim.mode() != gltf::mesh::Mode::Triangles {
                continue;
            }
            let reader = prim.reader(|buffer| buffers.get(buffer.index()).map(AsRef::as_ref));
            let Some(positions) = reader.read_positions() else {
                continue;
            };
            let joint_ids: Option<Vec<_>> = reader.read_joints(0).map(|j| j.into_u16().collect());
            let weights: Option<Vec<_>> = reader.read_weights(0).map(|w| w.into_f32().collect());
            let positions: Vec<[f32; 3]> = positions
                .enumerate()
                .map(|(i, p)| {
                    if let Some((matrices, ids, weights)) = joints
                        .as_ref()
                        .zip(joint_ids.as_ref().and_then(|v| v.get(i)))
                        .zip(weights.as_ref().and_then(|v| v.get(i)))
                        .map(|((a, b), c)| (a, b, c))
                    {
                        let mut out = [0.0; 3];
                        let mut total = 0.0;
                        for (&id, &w) in ids.iter().zip(weights) {
                            if w > 0.0 {
                                if let Some(m) = matrices.get(id as usize) {
                                    let q = transform_point(m, p);
                                    for a in 0..3 {
                                        out[a] += q[a] * w;
                                    }
                                    total += w;
                                }
                            }
                        }
                        if total > 0.0 {
                            return out.map(|v| v / total);
                        }
                    }
                    transform_point(&world, p)
                })
                .collect();
            let material = Some(prim.material().index().unwrap_or(materials.len() - 1) as u32);
            let mat = material
                .and_then(|i| materials.get(i as usize))
                .and_then(Option::as_ref);
            let read_uv = |t: Option<&MaterialTexture>| -> Option<Vec<[f32; 2]>> {
                reader
                    .read_tex_coords(t.map_or(0, |t| t.tex_coord))
                    .map(|tc| tc.into_f32().collect())
            };
            let uvs = read_uv(mat.and_then(|m| m.base_texture.as_ref()));
            let emissive_uvs = read_uv(mat.and_then(|m| m.emissive_texture.as_ref()));
            let transmission_uvs = read_uv(mat.and_then(|m| m.transmission_texture.as_ref()));
            let colors: Option<Vec<_>> = reader.read_colors(0).map(|c| c.into_rgba_f32().collect());
            let indices: Vec<u32> = match reader.read_indices() {
                Some(ix) => ix.into_u32().collect(),
                None => (0..positions.len() as u32).collect(),
            };
            for chunk in indices.chunks_exact(3) {
                let [a, b, c] = [chunk[0] as usize, chunk[1] as usize, chunk[2] as usize];
                if a >= positions.len() || b >= positions.len() || c >= positions.len() {
                    continue;
                }
                let uv = |uv: &Option<Vec<[f32; 2]>>| {
                    uv.as_ref()
                        .and_then(|uv| Some([*uv.get(a)?, *uv.get(b)?, *uv.get(c)?]))
                };
                triangles.push(MeshTriangle {
                    positions: [positions[a], positions[b], positions[c]],
                    uvs: uv(&uvs),
                    emissive_uvs: uv(&emissive_uvs),
                    transmission_uvs: uv(&transmission_uvs),
                    colors: colors
                        .as_ref()
                        .and_then(|v| Some([*v.get(a)?, *v.get(b)?, *v.get(c)?])),
                    material,
                });
            }
        }
    }
    for child in node.children() {
        visit_node(&child, worlds, buffers, materials, triangles);
    }
}

/// Decode an RFC 2397 `data:` URI (base64 payloads only). `None` for anything else.
fn decode_data_uri(uri: &str) -> Option<Vec<u8>> {
    let rest = uri.strip_prefix("data:")?;
    let (_mime, payload) = rest.split_once(";base64,")?;
    base64_decode(payload)
}

/// Tiny standard-alphabet base64 decoder (the `base64` crate is gated behind
/// the `bridge` feature; voxelize must work without it).
fn base64_decode(s: &str) -> Option<Vec<u8>> {
    fn val(c: u8) -> Option<u32> {
        match c {
            b'A'..=b'Z' => Some((c - b'A') as u32),
            b'a'..=b'z' => Some((c - b'a' + 26) as u32),
            b'0'..=b'9' => Some((c - b'0' + 52) as u32),
            b'+' => Some(62),
            b'/' => Some(63),
            _ => None,
        }
    }
    let bytes: Vec<u8> = s
        .bytes()
        .filter(|b| !b.is_ascii_whitespace() && *b != b'=')
        .collect();
    let mut out = Vec::with_capacity(bytes.len() * 3 / 4);
    for chunk in bytes.chunks(4) {
        let mut acc = 0u32;
        for (i, &b) in chunk.iter().enumerate() {
            acc |= val(b)? << (18 - 6 * i);
        }
        let n = chunk.len();
        if n < 2 {
            return None;
        }
        out.push((acc >> 16) as u8);
        if n > 2 {
            out.push((acc >> 8) as u8);
        }
        if n > 3 {
            out.push(acc as u8);
        }
    }
    Some(out)
}
