//! Pixel-level render regression harness.
//!
//! The rendering module's unit tests only cover pure logic (clear-colour math,
//! `RenderConfig` fields) — nothing touches the GPU. This example actually
//! renders, so a graphics-stack change (e.g. a wgpu major upgrade) can be
//! checked for visual regressions.
//!
//!     cargo run --release --example render_smoke --features rendering -- <pack.zip> <out_dir>
//!
//! Run it before and after the change, then compare the two output dirs.
//! Deterministic: fixed geometry, fixed camera, no randomness.

use nucleation::rendering::RenderConfig;
use nucleation::UniversalSchematic;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let pack_path = args.next().unwrap_or_else(|| {
        eprintln!("usage: render_smoke <pack.zip> <out_dir>");
        std::process::exit(2);
    });
    let out_dir = args
        .next()
        .unwrap_or_else(|| "render_work/smoke".to_string());
    std::fs::create_dir_all(&out_dir)?;

    let pack = nucleation::meshing::ResourcePackSource::from_file(&pack_path)?;
    let schem = build_scene();

    // Each case targets a different renderer path.
    let cases: Vec<(&str, RenderConfig)> = vec![
        ("iso", {
            let mut c = RenderConfig::isometric();
            c.width = 480;
            c.height = 360;
            c
        }),
        ("persp", {
            let mut c = RenderConfig::default();
            c.width = 480;
            c.height = 360;
            c.yaw = 35.0;
            c.pitch = 25.0;
            c
        }),
        // Alpha < 1.0 exercises the transparent-clear path.
        ("transparent", {
            let mut c = RenderConfig::isometric();
            c.width = 320;
            c.height = 240;
            c.background = Some([0.0, 0.0, 0.0, 0.0]);
            c
        }),
        // Solid clear colour, and a yaw that changes the visible silhouette.
        ("yaw120", {
            let mut c = RenderConfig::isometric();
            c.width = 320;
            c.height = 240;
            c.yaw = 120.0;
            c.background = Some([1.0, 1.0, 1.0, 1.0]);
            c
        }),
    ];

    for (name, cfg) in &cases {
        let path = format!("{out_dir}/{name}.png");
        schem.render_to_file(&pack, &path, cfg)?;
        let bytes = std::fs::read(&path)?;
        println!("{name:12} {:>8} bytes  {}", bytes.len(), sha256_hex(&bytes));
    }
    println!("\nwrote {} renders to {out_dir}/", cases.len());
    Ok(())
}

/// A deliberately varied scene: opaque cubes, a non-cube model, a transparent
/// block, and an emissive one — so the atlas, the AO bake, and the
/// opaque/transparent layer split are all exercised.
fn build_scene() -> UniversalSchematic {
    let mut s = UniversalSchematic::new("render_smoke".to_string());
    // Opaque floor: two materials, so the atlas holds more than one texture.
    for x in 0..8 {
        for z in 0..8 {
            let block = if (x + z) % 2 == 0 {
                "minecraft:stone"
            } else {
                "minecraft:oak_planks"
            };
            s.set_block_from_string(x, 0, z, block).ok();
        }
    }
    // A step pattern: concave corners give the AO bake something to do.
    for i in 0..4 {
        for z in 0..8 {
            s.set_block_from_string(i, 1 + i, z, "minecraft:stone_bricks")
                .ok();
        }
    }
    // Non-cube geometry (stairs + slab) exercises the model resolver.
    s.set_block_from_string(5, 1, 2, "minecraft:oak_stairs[facing=east]")
        .ok();
    s.set_block_from_string(5, 1, 4, "minecraft:stone_slab[type=bottom]")
        .ok();
    // Transparent + emissive: separate mesh layers.
    s.set_block_from_string(6, 1, 6, "minecraft:glass").ok();
    s.set_block_from_string(7, 1, 7, "minecraft:glowstone").ok();
    s
}

/// Minimal SHA-256 so the harness has no extra dependency.
fn sha256_hex(data: &[u8]) -> String {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let mut msg = data.to_vec();
    let bitlen = (data.len() as u64) * 8;
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bitlen.to_be_bytes());
    for chunk in msg.chunks(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                chunk[i * 4],
                chunk[i * 4 + 1],
                chunk[i * 4 + 2],
                chunk[i * 4 + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh) =
            (h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]);
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        for (i, v) in [a, b, c, d, e, f, g, hh].iter().enumerate() {
            h[i] = h[i].wrapping_add(*v);
        }
    }
    h.iter().map(|x| format!("{x:08x}")).collect::<String>()[..16].to_string()
}
