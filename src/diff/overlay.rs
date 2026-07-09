//! glTF surgery: append emissive translucent marker boxes onto a meshed GLB.
//!
//! The schematic mesher's `to_glb` lives in an external crate and quantizes its
//! own node, so we don't touch it — we parse the GLB container, append plain
//! f32 box geometry to the BIN buffer, and attach a second identity-transform
//! node with emissive materials. The build node stays quantized; the glow node
//! is plain f32, and the two coexist.

use serde_json::{json, Value};

use crate::diff::Diff;

/// Appearance of the overlay marker boxes.
#[derive(Clone, Debug)]
pub struct OverlayOptions {
    /// Units each box extends past every block face (0.08 ≈ a 1.16× cube).
    pub inflate: f32,
    /// `KHR_materials_emissive_strength` — glow intensity.
    pub emissive_strength: f32,
    /// Base-color alpha — lower is more transparent.
    pub alpha: f32,
    pub color_added: [f32; 3],
    pub color_removed: [f32; 3],
    pub color_changed: [f32; 3],
    pub color_swapped: [f32; 3],
}

impl Default for OverlayOptions {
    fn default() -> Self {
        Self {
            inflate: 0.08,
            emissive_strength: 1.4,
            alpha: 0.22,
            color_added: [0.1, 1.0, 0.05],
            color_removed: [1.0, 0.04, 0.04],
            color_changed: [1.0, 0.75, 0.0],
            color_swapped: [0.05, 0.45, 1.0],
        }
    }
}

/// Error injecting the overlay (the input wasn't a parseable GLB container).
#[derive(Debug)]
pub struct OverlayError(pub String);
impl std::fmt::Display for OverlayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::error::Error for OverlayError {}

// 8 cube corners (unit ±1) and the 12 triangles over them.
const CORNERS: [[f32; 3]; 8] = [
    [-1., -1., -1.],
    [1., -1., -1.],
    [-1., 1., -1.],
    [1., 1., -1.],
    [-1., -1., 1.],
    [1., -1., 1.],
    [-1., 1., 1.],
    [1., 1., 1.],
];
const FACES: [u32; 36] = [
    0, 1, 3, 0, 3, 2, 4, 6, 7, 4, 7, 5, 0, 4, 5, 0, 5, 1, 2, 3, 7, 2, 7, 6, 0, 2, 6, 0, 6, 4, 1, 5,
    7, 1, 7, 3,
];

impl Diff {
    /// Inject emissive translucent marker boxes (one per change cell) onto an
    /// already-meshed GLB. Boxes are centered on each block (matching the
    /// mesher's convention) and inflated by `opts.inflate` past each face.
    /// Colors: added/removed/changed/swapped per `opts`.
    pub fn to_overlay_glb(
        &self,
        after_glb: &[u8],
        opts: &OverlayOptions,
    ) -> Result<Vec<u8>, OverlayError> {
        let e = |m: &str| OverlayError(m.to_string());
        if after_glb.len() < 20 || &after_glb[0..4] != b"glTF" {
            return Err(e("not a GLB container"));
        }
        let rd = |o: usize| u32::from_le_bytes(after_glb[o..o + 4].try_into().unwrap()) as usize;
        let json_len = rd(12);
        let json_bytes = after_glb
            .get(20..20 + json_len)
            .ok_or_else(|| e("truncated JSON chunk"))?;
        let bin_hdr = 20 + json_len;
        let bin_len = rd(bin_hdr);
        let bin = after_glb
            .get(bin_hdr + 8..bin_hdr + 8 + bin_len)
            .ok_or_else(|| e("truncated BIN chunk"))?
            .to_vec();
        let mut root: Value =
            serde_json::from_slice(json_bytes).map_err(|e2| e(&e2.to_string()))?;

        let cats: [(Vec<(i32, i32, i32)>, [f32; 3]); 4] = [
            (
                self.added.iter().map(|(p, _)| *p).collect(),
                opts.color_added,
            ),
            (
                self.removed.iter().map(|(p, _)| *p).collect(),
                opts.color_removed,
            ),
            (
                self.changed.iter().map(|(p, _, _)| *p).collect(),
                opts.color_changed,
            ),
            (
                self.swapped.iter().map(|(p, _, _)| *p).collect(),
                opts.color_swapped,
            ),
        ];
        let half = 0.5 + opts.inflate;
        let base = bin.len();
        let mut blob: Vec<u8> = Vec::new();
        let mut new_bvs: Vec<Value> = Vec::new();
        let mut new_accs: Vec<Value> = Vec::new();
        let mut new_mats: Vec<Value> = Vec::new();
        let mut primitives: Vec<Value> = Vec::new();
        let bv0 = root["bufferViews"].as_array().map_or(0, |a| a.len());
        let ac0 = root["accessors"].as_array().map_or(0, |a| a.len());
        let mt0 = root["materials"].as_array().map_or(0, |a| a.len());

        for (cells, col) in cats.iter().filter(|(c, _)| !c.is_empty()) {
            let mut verts: Vec<[f32; 3]> = Vec::with_capacity(cells.len() * 8);
            let mut idx: Vec<u32> = Vec::with_capacity(cells.len() * 36);
            for (bi, (x, y, z)) in cells.iter().enumerate() {
                let c = [*x as f32, *y as f32, *z as f32];
                for corner in CORNERS {
                    verts.push([
                        c[0] + half * corner[0],
                        c[1] + half * corner[1],
                        c[2] + half * corner[2],
                    ]);
                }
                for f in FACES {
                    idx.push(f + (bi as u32) * 8);
                }
            }
            let mut mn = [f32::MAX; 3];
            let mut mx = [f32::MIN; 3];
            for v in &verts {
                for k in 0..3 {
                    mn[k] = mn[k].min(v[k]);
                    mx[k] = mx[k].max(v[k]);
                }
            }
            let pos_off = base + blob.len();
            for v in &verts {
                for comp in v {
                    blob.extend_from_slice(&comp.to_le_bytes());
                }
            }
            let idx_off = base + blob.len();
            for i in &idx {
                blob.extend_from_slice(&i.to_le_bytes());
            }
            let bv_pos = bv0 + new_bvs.len();
            new_bvs.push(
                json!({"buffer":0,"byteOffset":pos_off,"byteLength":verts.len()*12,"target":34962}),
            );
            let bv_idx = bv0 + new_bvs.len();
            new_bvs.push(
                json!({"buffer":0,"byteOffset":idx_off,"byteLength":idx.len()*4,"target":34963}),
            );
            let ac_pos = ac0 + new_accs.len();
            new_accs.push(json!({"bufferView":bv_pos,"componentType":5126,"count":verts.len(),"type":"VEC3","min":mn,"max":mx}));
            let ac_idx = ac0 + new_accs.len();
            new_accs.push(
                json!({"bufferView":bv_idx,"componentType":5125,"count":idx.len(),"type":"SCALAR"}),
            );
            let mat = mt0 + new_mats.len();
            new_mats.push(json!({
                "pbrMetallicRoughness":{"baseColorFactor":[col[0],col[1],col[2],opts.alpha],"metallicFactor":0.0,"roughnessFactor":1.0},
                "emissiveFactor":col,
                "extensions":{"KHR_materials_emissive_strength":{"emissiveStrength":opts.emissive_strength}},
                "alphaMode":"BLEND",
                "doubleSided":true
            }));
            primitives
                .push(json!({"attributes":{"POSITION":ac_pos},"indices":ac_idx,"material":mat}));
        }
        if primitives.is_empty() {
            return Ok(after_glb.to_vec());
        }

        let ext = |root: &mut Value, key: &str, items: Vec<Value>| {
            if root[key].is_array() {
                root[key].as_array_mut().unwrap().extend(items);
            } else {
                root[key] = Value::Array(items);
            }
        };
        ext(&mut root, "bufferViews", new_bvs);
        ext(&mut root, "accessors", new_accs);
        ext(&mut root, "materials", new_mats);
        let mesh_idx = root["meshes"].as_array().map_or(0, |a| a.len());
        root["meshes"]
            .as_array_mut()
            .ok_or_else(|| e("no meshes array"))?
            .push(json!({ "primitives": primitives }));
        let node_idx = root["nodes"].as_array().map_or(0, |a| a.len());
        root["nodes"]
            .as_array_mut()
            .ok_or_else(|| e("no nodes array"))?
            .push(json!({ "mesh": mesh_idx }));
        root["scenes"][0]["nodes"]
            .as_array_mut()
            .ok_or_else(|| e("no scene nodes"))?
            .push(json!(node_idx));
        root["buffers"][0]["byteLength"] = json!(base + blob.len());
        let used = root["extensionsUsed"]
            .as_array_mut()
            .ok_or_else(|| e("no extensionsUsed"))?;
        if !used.iter().any(|x| x == "KHR_materials_emissive_strength") {
            used.push(json!("KHR_materials_emissive_strength"));
        }

        let mut json_out = serde_json::to_vec(&root).map_err(|e2| e(&e2.to_string()))?;
        while json_out.len() % 4 != 0 {
            json_out.push(b' ');
        }
        let mut bin_out = bin;
        bin_out.extend_from_slice(&blob);
        while bin_out.len() % 4 != 0 {
            bin_out.push(0);
        }
        let total = 12 + 8 + json_out.len() + 8 + bin_out.len();
        let mut glb = Vec::with_capacity(total);
        glb.extend_from_slice(b"glTF");
        glb.extend_from_slice(&2u32.to_le_bytes());
        glb.extend_from_slice(&(total as u32).to_le_bytes());
        glb.extend_from_slice(&(json_out.len() as u32).to_le_bytes());
        glb.extend_from_slice(&0x4E4F_534Au32.to_le_bytes()); // "JSON"
        glb.extend_from_slice(&json_out);
        glb.extend_from_slice(&(bin_out.len() as u32).to_le_bytes());
        glb.extend_from_slice(&0x004E_4942u32.to_le_bytes()); // "BIN\0"
        glb.extend_from_slice(&bin_out);
        Ok(glb)
    }
}

#[cfg(test)]
mod tests {
    use super::OverlayOptions;
    use crate::diff::{diff, DiffSpec};
    use crate::fingerprint::testgen::{edited, filled_box};
    use crate::fingerprint::FingerprintSpec;

    /// A minimal valid 2-chunk GLB (JSON+BIN) with one empty buffer, enough for
    /// the surgery to parse and append to. Built inline to avoid needing a pack.
    fn empty_glb() -> Vec<u8> {
        let json = br#"{"asset":{"version":"2.0"},"buffers":[{"byteLength":0}],"bufferViews":[],"accessors":[],"materials":[],"meshes":[],"nodes":[{"mesh":0}],"scenes":[{"nodes":[0]}],"scene":0,"extensionsUsed":[]}"#;
        let mut j = json.to_vec();
        while !j.len().is_multiple_of(4) {
            j.push(b' ');
        }
        let bin: Vec<u8> = Vec::new();
        let total = 12 + 8 + j.len() + 8 + bin.len();
        let mut g = Vec::new();
        g.extend_from_slice(b"glTF");
        g.extend_from_slice(&2u32.to_le_bytes());
        g.extend_from_slice(&(total as u32).to_le_bytes());
        g.extend_from_slice(&(j.len() as u32).to_le_bytes());
        g.extend_from_slice(&0x4E4F_534Au32.to_le_bytes());
        g.extend_from_slice(&j);
        g.extend_from_slice(&(bin.len() as u32).to_le_bytes());
        g.extend_from_slice(&0x004E_4942u32.to_le_bytes());
        g.extend_from_slice(&bin);
        g
    }

    #[test]
    fn overlay_appends_valid_glb() {
        let a = filled_box((0, 0, 0), (4, 0, 4), "minecraft:stone");
        let (b, _) = edited(&a, 3);
        let d = diff(&a, &b, &DiffSpec::from_preset(FingerprintSpec::exact()));
        let out = d
            .to_overlay_glb(&empty_glb(), &OverlayOptions::default())
            .unwrap();

        assert_eq!(&out[0..4], b"glTF");
        let len = u32::from_le_bytes(out[8..12].try_into().unwrap()) as usize;
        assert_eq!(len, out.len(), "declared length matches actual");
        assert!(out.len() > empty_glb().len(), "geometry was appended");
    }

    #[test]
    fn overlay_rejects_non_glb() {
        let a = filled_box((0, 0, 0), (1, 0, 1), "minecraft:stone");
        let (b, _) = edited(&a, 1);
        let d = diff(&a, &b, &DiffSpec::from_preset(FingerprintSpec::exact()));
        assert!(d
            .to_overlay_glb(b"not a glb", &OverlayOptions::default())
            .is_err());
    }
}
