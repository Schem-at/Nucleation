//! Mesh model loading: GLB (glTF binary) and minimal OBJ, plus the `fit`
//! normalization that maps a model into voxel space.

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
    /// Index into [`MeshModel::materials`].
    pub material: Option<u32>,
}

/// Triangles in model space plus a material → decoded-RGBA-image table.
///
/// Load with [`MeshModel::from_glb_bytes`] / [`MeshModel::from_obj_str`], then
/// normalize into voxel space with [`MeshModel::fit`].
pub struct MeshModel {
    pub triangles: Vec<MeshTriangle>,
    /// One slot per glTF material; `None` when the material carries neither a
    /// base-color texture nor a usable base-color factor.
    pub materials: Vec<Option<TextureImage>>,
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
        let anchor = [
            (min[0] + max[0]) * 0.5,
            min[1],
            (min[2] + max[2]) * 0.5,
        ];
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
    /// primitives (non-triangle modes are ignored), and base-color textures
    /// decoded to RGBA (materials without a texture fall back to a 1×1 image
    /// of their base-color factor).
    pub fn from_glb_bytes(data: &[u8]) -> Result<MeshModel, String> {
        let gltf = gltf::Gltf::from_slice(data).map_err(|e| format!("GLB parse error: {e}"))?;
        let doc = gltf.document;
        let blob = gltf.blob;

        // Resolve buffers: BIN chunk or embedded data: URIs only.
        let mut buffers: Vec<Vec<u8>> = Vec::with_capacity(doc.buffers().count());
        for buffer in doc.buffers() {
            let data = match buffer.source() {
                gltf::buffer::Source::Bin => blob
                    .clone()
                    .ok_or_else(|| "GLB references BIN chunk but has none".to_string())?,
                gltf::buffer::Source::Uri(uri) => decode_data_uri(uri).ok_or_else(|| {
                    format!("unsupported external buffer URI in GLB: {uri}")
                })?,
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
        let mut images: Vec<Option<TextureImage>> = Vec::with_capacity(doc.images().count());
        for img in doc.images() {
            let bytes: Option<Vec<u8>> = match img.source() {
                gltf::image::Source::View { view, .. } => {
                    let buf = &buffers[view.buffer().index()];
                    buf.get(view.offset()..view.offset() + view.length())
                        .map(|s| s.to_vec())
                }
                gltf::image::Source::Uri { uri, .. } => decode_data_uri(uri),
            };
            let decoded = bytes.and_then(|b| image::load_from_memory(&b).ok()).map(|d| {
                let rgba = d.to_rgba8();
                TextureImage {
                    width: rgba.width(),
                    height: rgba.height(),
                    pixels: rgba.into_raw(),
                }
            });
            images.push(decoded);
        }

        // Material table: base-color texture, else 1x1 base-color factor.
        let mut materials: Vec<Option<TextureImage>> = Vec::new();
        for mat in doc.materials() {
            if mat.index().is_none() {
                continue; // default material handled by `material: None`
            }
            let pbr = mat.pbr_metallic_roughness();
            let tex = pbr
                .base_color_texture()
                .and_then(|info| images[info.texture().source().index()].clone());
            let entry = tex.or_else(|| {
                let f = pbr.base_color_factor();
                Some(TextureImage {
                    width: 1,
                    height: 1,
                    pixels: vec![
                        (f[0].clamp(0.0, 1.0) * 255.0).round() as u8,
                        (f[1].clamp(0.0, 1.0) * 255.0).round() as u8,
                        (f[2].clamp(0.0, 1.0) * 255.0).round() as u8,
                        (f[3].clamp(0.0, 1.0) * 255.0).round() as u8,
                    ],
                })
            });
            materials.push(entry);
        }

        let mut triangles = Vec::new();
        let scenes: Vec<gltf::Scene> = doc.scenes().collect();
        for scene in &scenes {
            for node in scene.nodes() {
                visit_node(&node, IDENTITY, &buffers, &mut triangles);
            }
        }
        // Models with no scene at all: fall back to walking every root-less node.
        if scenes.is_empty() {
            for node in doc.nodes() {
                visit_node(&node, IDENTITY, &buffers, &mut triangles);
            }
        }

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
                        let ti = parts
                            .next()
                            .and_then(|t| resolve_index(t, texcoords.len()));
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
                            material: None,
                        });
                    }
                }
                _ => {} // vn, o, g, s, usemtl, mtllib, ... ignored
            }
        }

        if triangles.is_empty() {
            return Err("OBJ contains no triangles".to_string());
        }
        Ok(MeshModel {
            triangles,
            materials: Vec::new(),
        })
    }
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

fn visit_node(
    node: &gltf::Node,
    parent: Mat4,
    buffers: &[Vec<u8>],
    triangles: &mut Vec<MeshTriangle>,
) {
    let world = mat_mul(&parent, &node.transform().matrix());
    if let Some(mesh) = node.mesh() {
        for prim in mesh.primitives() {
            if prim.mode() != gltf::mesh::Mode::Triangles {
                continue;
            }
            let reader = prim.reader(|buffer| buffers.get(buffer.index()).map(|v| &v[..]));
            let Some(positions) = reader.read_positions() else {
                continue;
            };
            let positions: Vec<[f32; 3]> =
                positions.map(|p| transform_point(&world, p)).collect();
            let uvs: Option<Vec<[f32; 2]>> = reader
                .read_tex_coords(0)
                .map(|tc| tc.into_f32().collect());
            let indices: Vec<u32> = match reader.read_indices() {
                Some(ix) => ix.into_u32().collect(),
                None => (0..positions.len() as u32).collect(),
            };
            let material = prim.material().index().map(|i| i as u32);
            for chunk in indices.chunks_exact(3) {
                let [a, b, c] = [chunk[0] as usize, chunk[1] as usize, chunk[2] as usize];
                if a >= positions.len() || b >= positions.len() || c >= positions.len() {
                    continue;
                }
                let tri_uvs = uvs.as_ref().and_then(|uv| {
                    (a < uv.len() && b < uv.len() && c < uv.len())
                        .then(|| [uv[a], uv[b], uv[c]])
                });
                triangles.push(MeshTriangle {
                    positions: [positions[a], positions[b], positions[c]],
                    uvs: tri_uvs,
                    material,
                });
            }
        }
    }
    for child in node.children() {
        visit_node(&child, world, buffers, triangles);
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
