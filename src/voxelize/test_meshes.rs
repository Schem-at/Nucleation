//! Deterministic meshes shared by the voxelize benches, the in-crate
//! regression test and the golden fixture test. Kept out of the public docs
//! but compiled unconditionally: benches are separate crates and cannot see
//! a `cfg(test)` module.

/// A closed UV sphere of radius 1 centred on the origin, as Wavefront OBJ.
/// `stacks` latitude divisions by `sectors` longitude divisions gives
/// `2 * sectors * (stacks - 1)` triangles.
#[doc(hidden)]
pub fn uv_sphere_obj(stacks: usize, sectors: usize) -> String {
    assert!(stacks >= 2 && sectors >= 3, "degenerate sphere");
    let mut out = String::with_capacity(stacks * sectors * 32);
    for i in 0..=stacks {
        let phi = std::f64::consts::PI * (i as f64) / (stacks as f64);
        let (sp, cp) = phi.sin_cos();
        for j in 0..sectors {
            let theta = 2.0 * std::f64::consts::PI * (j as f64) / (sectors as f64);
            let (st, ct) = theta.sin_cos();
            out.push_str(&format!("v {:.6} {:.6} {:.6}\n", sp * ct, cp, sp * st));
        }
    }
    // Vertex index of latitude ring `i`, longitude `j`, 1 based for OBJ.
    let vid = |i: usize, j: usize| -> usize { i * sectors + (j % sectors) + 1 };
    for i in 0..stacks {
        for j in 0..sectors {
            let (a, b) = (vid(i, j), vid(i, j + 1));
            let (c, d) = (vid(i + 1, j + 1), vid(i + 1, j));
            if i == 0 {
                out.push_str(&format!("f {a} {c} {d}\n"));
            } else if i == stacks - 1 {
                out.push_str(&format!("f {a} {b} {c}\n"));
            } else {
                out.push_str(&format!("f {a} {b} {c}\n"));
                out.push_str(&format!("f {a} {c} {d}\n"));
            }
        }
    }
    out
}

/// The 5,000 triangle sphere the performance work is measured against.
#[doc(hidden)]
pub fn sphere_5k() -> String {
    uv_sphere_obj(51, 50)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::voxelize::MeshModel;

    #[test]
    fn sphere_5k_has_the_expected_triangle_count() {
        let model = MeshModel::from_obj_str(&sphere_5k()).expect("sphere parses");
        assert_eq!(model.triangles.len(), 5_000);
    }
}
