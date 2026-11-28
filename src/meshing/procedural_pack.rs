//! Built-in procedural Minecraft block resource pack.
//!
//! Synthesizes crisp 16×16 RGBA pixel-art textures, multi-face block models,
//! fluid stills/flows, cross-plant flora, redstone state variants, and full
//! 16-color concrete/wool/terracotta spectra entirely in memory — zero
//! external files and zero proprietary Mojang assets required.
//!
//! Exposed as [`super::ResourcePackSource::builtin_procedural`] (Rust) and
//! `ResourcePack.builtin_procedural()` across all Diplomat bindings, and used
//! as the zero-config fallback in `nucleation mesh` / `nucleation render`.

use super::ResourcePackSource;
use schematic_mesher::ResourcePack;

#[inline]
fn clamp_byte(v: f32) -> u8 {
    v.round().clamp(0.0, 255.0) as u8
}

#[inline]
fn tex_hash(x: u32, y: u32, salt: u32) -> f32 {
    let mut n = x
        .wrapping_mul(374_761_393)
        .wrapping_add(y.wrapping_mul(668_265_263))
        .wrapping_add(salt.wrapping_mul(1_442_695_041));
    n = (n ^ (n >> 13)).wrapping_mul(1_274_126_177);
    (n ^ (n >> 16)) as f32 / 4_294_967_295.0
}

fn make_tex(mut painter: impl FnMut(u32, u32) -> [f32; 4]) -> Vec<u8> {
    let mut px = vec![0u8; 16 * 16 * 4];
    for y in 0..16u32 {
        for x in 0..16u32 {
            let [r, g, b, a] = painter(x, y);
            let i = ((y * 16 + x) * 4) as usize;
            px[i] = clamp_byte(r);
            px[i + 1] = clamp_byte(g);
            px[i + 2] = clamp_byte(b);
            px[i + 3] = clamp_byte(a);
        }
    }
    px
}

fn solid_speckle(rgb: [f32; 3], amp: f32, salt: u32) -> Vec<u8> {
    make_tex(|x, y| {
        let d = (tex_hash(x, y, salt) - 0.5) * 2.0 * amp;
        [rgb[0] + d, rgb[1] + d, rgb[2] + d, 255.0]
    })
}

fn brick_tex(base: [f32; 3], mortar: [f32; 3], mossy: bool, salt: u32) -> Vec<u8> {
    make_tex(|x, y| {
        let row = y >> 3;
        let is_mortar = y == 0 || y == 8 || ((x + if row != 0 { 8 } else { 0 }) & 15) == 0;
        let n = (tex_hash(x, y, salt) - 0.5) * 16.0;
        if is_mortar {
            return [
                mortar[0] + n * 0.4,
                mortar[1] + n * 0.4,
                mortar[2] + n * 0.4,
                255.0,
            ];
        }
        if mossy && tex_hash(x >> 1, y >> 1, 91) > 0.72 {
            return [76.0 + n * 0.6, 114.0 + n * 0.8, 48.0 + n * 0.5, 255.0];
        }
        let bevel = if y == 1 || y == 9 {
            10.0
        } else if y == 7 || y == 15 {
            -10.0
        } else {
            0.0
        };
        [base[0] + n + bevel, base[1] + n + bevel, base[2] + n + bevel, 255.0]
    })
}

fn planks_tex(base: [f32; 3], seam: [f32; 3], salt: u32) -> Vec<u8> {
    make_tex(|x, y| {
        let board = y >> 2;
        let is_h = (y & 3) == 0;
        let is_v = (x == 7 && (board & 1) == 0) || (x == 15 && (board & 1) == 1);
        let grain = (tex_hash(x >> 1, y, salt + board * 13) - 0.5) * 18.0;
        if is_h || is_v {
            return [seam[0], seam[1], seam[2], 255.0];
        }
        let shade = match y & 3 {
            1 => 8.0,
            3 => -8.0,
            _ => 0.0,
        };
        [base[0] + grain + shade, base[1] + grain + shade, base[2] + grain + shade, 255.0]
    })
}

fn cobble_tex(base: [f32; 3], dark: [f32; 3], salt: u32) -> Vec<u8> {
    make_tex(|x, y| {
        let shift = ((y >> 2) & 1) * 2;
        let cx = (x + shift) / 4;
        let cy = y / 4;
        let edge = (x + shift) % 4 == 0 || y % 4 == 0;
        let tone = (tex_hash(cx, cy, salt) - 0.5) * 32.0;
        let micro = (tex_hash(x, y, salt + 7) - 0.5) * 10.0;
        if edge {
            [dark[0] + micro, dark[1] + micro, dark[2] + micro, 255.0]
        } else {
            [
                base[0] + tone + micro,
                base[1] + tone + micro,
                base[2] + tone + micro,
                255.0,
            ]
        }
    })
}

fn add_cube_all(pack: &mut ResourcePackSource, name: &str, pixels: Vec<u8>) {
    let block = format!("minecraft:{name}");
    let tex = format!("minecraft:block/{name}");
    let bs = format!(r##"{{"variants":{{"":{{"model":"{tex}"}}}}}}"##);
    let model = format!(
        r##"{{"textures":{{"all":"{tex}"}},"elements":[{{"from":[0,0,0],"to":[16,16,16],"faces":{{"down":{{"texture":"#all","cullface":"down"}},"up":{{"texture":"#all","cullface":"up"}},"north":{{"texture":"#all","cullface":"north"}},"south":{{"texture":"#all","cullface":"south"}},"west":{{"texture":"#all","cullface":"west"}},"east":{{"texture":"#all","cullface":"east"}}}}}}]}}"##
    );
    let _ = pack.add_blockstate_json(&block, &bs);
    let _ = pack.add_model_json(&tex, &model);
    let _ = pack.add_texture(&tex, 16, 16, pixels);
}

fn add_cube_top_side_bottom(
    pack: &mut ResourcePackSource,
    name: &str,
    top: Vec<u8>,
    side: Vec<u8>,
    bottom: Vec<u8>,
) {
    let block = format!("minecraft:{name}");
    let model_name = format!("minecraft:block/{name}");
    let t_top = format!("minecraft:block/{name}_top");
    let t_side = format!("minecraft:block/{name}_side");
    let t_bot = format!("minecraft:block/{name}_bottom");
    let bs = format!(r##"{{"variants":{{"":{{"model":"{model_name}"}}}}}}"##);
    let model = format!(
        r##"{{"textures":{{"top":"{t_top}","side":"{t_side}","bottom":"{t_bot}"}},"elements":[{{"from":[0,0,0],"to":[16,16,16],"faces":{{"up":{{"texture":"#top","cullface":"up"}},"down":{{"texture":"#bottom","cullface":"down"}},"north":{{"texture":"#side","cullface":"north"}},"south":{{"texture":"#side","cullface":"south"}},"west":{{"texture":"#side","cullface":"west"}},"east":{{"texture":"#side","cullface":"east"}}}}}}]}}"##
    );
    let _ = pack.add_blockstate_json(&block, &bs);
    let _ = pack.add_model_json(&model_name, &model);
    let _ = pack.add_texture(&t_top, 16, 16, top);
    let _ = pack.add_texture(&t_side, 16, 16, side);
    let _ = pack.add_texture(&t_bot, 16, 16, bottom);
}

fn add_cross_plant(pack: &mut ResourcePackSource, name: &str, pixels: Vec<u8>) {
    let block = format!("minecraft:{name}");
    let tex = format!("minecraft:block/{name}");
    let bs = format!(r##"{{"variants":{{"":{{"model":"{tex}"}}}}}}"##);
    let model = format!(
        r##"{{"textures":{{"cross":"{tex}"}},"elements":[{{"from":[2,0,8],"to":[14,14,8],"rotation":{{"origin":[8,8,8],"axis":"y","angle":45,"rescale":true}},"shade":false,"faces":{{"north":{{"texture":"#cross"}},"south":{{"texture":"#cross"}}}}}},{{"from":[8,0,2],"to":[8,14,14],"rotation":{{"origin":[8,8,8],"axis":"y","angle":45,"rescale":true}},"shade":false,"faces":{{"east":{{"texture":"#cross"}},"west":{{"texture":"#cross"}}}}}}]}}"##
    );
    let _ = pack.add_blockstate_json(&block, &bs);
    let _ = pack.add_model_json(&tex, &model);
    let _ = pack.add_texture(&tex, 16, 16, pixels);
}

/// Build the built-in procedural Minecraft block resource pack.
pub(crate) fn build_builtin_procedural_pack() -> ResourcePackSource {
    let mut pack = ResourcePackSource::from_resource_pack(ResourcePack::new());

    // 1. Grass block & soils
    let dirt_px = make_tex(|x, y| {
        let n = (tex_hash(x, y, 11) - 0.5) * 24.0;
        let pebble = if tex_hash(x, y, 29) > 0.9 { 28.0 } else { 0.0 };
        [134.0 + n + pebble, 96.0 + n * 0.8 + pebble, 67.0 + n * 0.6 + pebble, 255.0]
    });
    let grass_top_px = make_tex(|x, y| {
        let n = (tex_hash(x, y, 17) - 0.5) * 26.0;
        let blade = if tex_hash(x, y, 37) > 0.72 { 16.0 } else { 0.0 };
        [102.0 + n * 0.7 + blade * 0.6, 164.0 + n + blade, 66.0 + n * 0.5, 255.0]
    });
    let grass_side_px = make_tex(|x, y| {
        let fringe = 2 + (tex_hash(x, 0, 41) * 3.6) as u32;
        if y < fringe {
            let n = (tex_hash(x, y, 17) - 0.5) * 22.0;
            [98.0 + n * 0.7, 158.0 + n, 62.0 + n * 0.5, 255.0]
        } else {
            let n = (tex_hash(x, y, 11) - 0.5) * 24.0;
            [134.0 + n, 96.0 + n * 0.8, 67.0 + n * 0.6, 255.0]
        }
    });
    add_cube_top_side_bottom(&mut pack, "grass_block", grass_top_px.clone(), grass_side_px, dirt_px.clone());
    add_cube_all(&mut pack, "dirt", dirt_px.clone());
    add_cube_all(&mut pack, "coarse_dirt", solid_speckle([122.0, 86.0, 58.0], 20.0, 12));
    add_cube_all(&mut pack, "moss_block", grass_top_px);

    // 2. Logs & leaves
    let oak_bark = make_tex(|x, y| {
        let ridge = if (x + (tex_hash(x, y >> 2, 51) * 2.0) as u32) % 3 == 0 {
            -16.0
        } else {
            8.0
        };
        let n = (tex_hash(x, y, 52) - 0.5) * 12.0;
        [108.0 + ridge + n, 84.0 + ridge * 0.8 + n, 52.0 + ridge * 0.6 + n, 255.0]
    });
    let oak_top = make_tex(|x, y| {
        let r = ((x as f32 - 7.5).powi(2) + (y as f32 - 7.5).powi(2)).sqrt();
        if r > 6.6 {
            return [102.0, 78.0, 48.0, 255.0];
        }
        let ring = if (r * 2.2).sin() > 0.0 { 12.0 } else { -10.0 };
        [174.0 + ring, 140.0 + ring * 0.85, 92.0 + ring * 0.65, 255.0]
    });
    add_cube_top_side_bottom(&mut pack, "oak_log", oak_top.clone(), oak_bark, oak_top.clone());

    let spruce_bark = make_tex(|x, y| {
        let ridge = if x % 3 == 0 { -14.0 } else { 6.0 };
        let n = (tex_hash(x, y, 55) - 0.5) * 10.0;
        [62.0 + ridge + n, 44.0 + ridge * 0.7 + n, 28.0 + ridge * 0.5 + n, 255.0]
    });
    add_cube_top_side_bottom(&mut pack, "spruce_log", oak_top.clone(), spruce_bark, oak_top.clone());

    let birch_bark = make_tex(|x, y| {
        let speck = if (y % 4 == 1) && tex_hash(x >> 1, y, 58) > 0.74 {
            -155.0
        } else {
            0.0
        };
        let n = (tex_hash(x, y, 59) - 0.5) * 10.0;
        [226.0 + speck + n, 224.0 + speck + n, 218.0 + speck + n, 255.0]
    });
    add_cube_top_side_bottom(&mut pack, "birch_log", oak_top.clone(), birch_bark, oak_top);

    add_cube_all(
        &mut pack,
        "oak_leaves",
        make_tex(|x, y| {
            let c = tex_hash(x >> 1, y >> 1, 61);
            let n = (tex_hash(x, y, 62) - 0.5) * 22.0;
            if c < 0.24 {
                [46.0 + n * 0.5, 86.0 + n * 0.6, 30.0 + n * 0.4, 255.0]
            } else {
                [76.0 + n * 0.7, 134.0 + n, 46.0 + n * 0.5, 255.0]
            }
        }),
    );
    add_cube_all(
        &mut pack,
        "spruce_leaves",
        make_tex(|x, y| {
            let c = tex_hash(x >> 1, y >> 1, 67);
            let n = (tex_hash(x, y, 68) - 0.5) * 18.0;
            if c < 0.25 {
                [36.0 + n * 0.4, 64.0 + n * 0.5, 42.0 + n * 0.4, 255.0]
            } else {
                [58.0 + n * 0.6, 98.0 + n * 0.8, 64.0 + n * 0.6, 255.0]
            }
        }),
    );
    add_cube_all(
        &mut pack,
        "birch_leaves",
        make_tex(|x, y| {
            let c = tex_hash(x >> 1, y >> 1, 69);
            let n = (tex_hash(x, y, 70) - 0.5) * 20.0;
            if c < 0.24 {
                [66.0 + n * 0.5, 112.0 + n * 0.6, 46.0 + n * 0.4, 255.0]
            } else {
                [102.0 + n * 0.7, 152.0 + n, 68.0 + n * 0.5, 255.0]
            }
        }),
    );

    // 3. Sandstone, stone, bricks, planks & minerals
    let sand_px = solid_speckle([222.0, 208.0, 162.0], 12.0, 71);
    let sandstone_side = make_tex(|x, y| {
        let n = (tex_hash(x, y, 73) - 0.5) * 12.0;
        let band = if y < 4 {
            10.0
        } else if y == 4 {
            -18.0
        } else if y > 12 {
            -10.0
        } else {
            0.0
        };
        [218.0 + band + n, 204.0 + band + n, 156.0 + band + n, 255.0]
    });
    add_cube_top_side_bottom(
        &mut pack,
        "sandstone",
        sand_px.clone(),
        sandstone_side,
        cobble_tex([208.0, 192.0, 146.0], [176.0, 160.0, 118.0], 75),
    );
    add_cube_all(&mut pack, "smooth_sandstone", sand_px.clone());
    add_cube_all(&mut pack, "sand", sand_px);

    add_cube_all(&mut pack, "stone", solid_speckle([130.0, 132.0, 134.0], 14.0, 101));
    add_cube_all(
        &mut pack,
        "smooth_stone",
        make_tex(|x, y| {
            let n = (tex_hash(x, y, 100) - 0.5) * 7.0;
            let outer = x == 0 || y == 0 || x == 15 || y == 15;
            let inner_hi = x == 1 || y == 1;
            let inner_lo = x == 14 || y == 14;
            let base = if outer {
                138.0
            } else if inner_hi {
                172.0
            } else if inner_lo {
                150.0
            } else {
                160.0
            };
            [base + n, base + 1.0 + n, base + 3.0 + n, 255.0]
        }),
    );
    add_cube_all(&mut pack, "andesite", solid_speckle([144.0, 146.0, 144.0], 18.0, 102));
    add_cube_all(&mut pack, "diorite", solid_speckle([196.0, 196.0, 198.0], 26.0, 99));
    add_cube_all(&mut pack, "granite", solid_speckle([152.0, 108.0, 92.0], 20.0, 98));
    add_cube_all(&mut pack, "tuff", solid_speckle([108.0, 109.0, 101.0], 16.0, 103));
    add_cube_all(
        &mut pack,
        "deepslate",
        make_tex(|x, y| {
            let stripe = match y % 4 {
                0 => -12.0,
                2 => 8.0,
                _ => 0.0,
            } + (tex_hash(x, y, 104) - 0.5) * 12.0;
            [72.0 + stripe, 74.0 + stripe, 82.0 + stripe, 255.0]
        }),
    );
    add_cube_all(&mut pack, "cobblestone", cobble_tex([126.0, 128.0, 128.0], [86.0, 88.0, 90.0], 105));
    add_cube_all(&mut pack, "stone_bricks", brick_tex([132.0, 134.0, 134.0], [88.0, 90.0, 92.0], false, 106));
    add_cube_all(&mut pack, "mossy_stone_bricks", brick_tex([128.0, 132.0, 128.0], [82.0, 92.0, 78.0], true, 107));
    add_cube_all(&mut pack, "deepslate_bricks", brick_tex([74.0, 76.0, 82.0], [46.0, 48.0, 54.0], false, 108));
    add_cube_all(&mut pack, "bricks", brick_tex([154.0, 90.0, 72.0], [198.0, 192.0, 182.0], false, 115));
    add_cube_all(&mut pack, "quartz_block", solid_speckle([236.0, 232.0, 224.0], 6.0, 116));
    add_cube_all(&mut pack, "oak_planks", planks_tex([168.0, 134.0, 84.0], [116.0, 88.0, 52.0], 109));
    add_cube_all(&mut pack, "spruce_planks", planks_tex([116.0, 86.0, 52.0], [78.0, 54.0, 32.0], 110));
    add_cube_all(&mut pack, "birch_planks", planks_tex([204.0, 186.0, 132.0], [154.0, 138.0, 94.0], 117));
    add_cube_all(&mut pack, "dark_oak_planks", planks_tex([72.0, 48.0, 28.0], [46.0, 30.0, 16.0], 111));
    add_cube_all(
        &mut pack,
        "calcite",
        make_tex(|x, y| {
            let v = if (x + 32 - y) % 7 == 0 { -16.0 } else { 0.0 } + (tex_hash(x, y, 112) - 0.5) * 10.0;
            [224.0 + v, 223.0 + v, 214.0 + v, 255.0]
        }),
    );
    add_cube_all(&mut pack, "snow_block", solid_speckle([242.0, 248.0, 252.0], 6.0, 113));
    add_cube_all(&mut pack, "packed_ice", solid_speckle([144.0, 182.0, 248.0], 12.0, 118));
    add_cube_all(&mut pack, "gravel", cobble_tex([132.0, 128.0, 126.0], [96.0, 92.0, 90.0], 114));

    // 4. Metals, copper, prismarine, crystals & Nether
    add_cube_all(&mut pack, "oxidized_copper", brick_tex([82.0, 164.0, 146.0], [56.0, 118.0, 106.0], false, 120));
    add_cube_all(&mut pack, "weathered_copper", brick_tex([106.0, 152.0, 118.0], [74.0, 112.0, 86.0], false, 121));
    add_cube_all(&mut pack, "exposed_copper", brick_tex([164.0, 124.0, 102.0], [112.0, 86.0, 70.0], false, 119));
    add_cube_all(&mut pack, "cut_copper", brick_tex([192.0, 108.0, 78.0], [134.0, 72.0, 50.0], false, 122));
    add_cube_all(&mut pack, "copper_block", brick_tex([196.0, 112.0, 82.0], [142.0, 76.0, 54.0], false, 123));
    add_cube_all(&mut pack, "prismarine", cobble_tex([104.0, 158.0, 150.0], [66.0, 112.0, 108.0], 123));
    add_cube_all(&mut pack, "prismarine_bricks", brick_tex([98.0, 174.0, 162.0], [64.0, 124.0, 116.0], false, 124));
    add_cube_all(&mut pack, "dark_prismarine", brick_tex([52.0, 98.0, 88.0], [32.0, 64.0, 58.0], false, 125));
    add_cube_all(
        &mut pack,
        "amethyst_block",
        make_tex(|x, y| {
            let facet = if (x + y) % 6 == 0 || (x + 16 - y) % 8 == 0 {
                28.0
            } else {
                -8.0
            };
            let n = (tex_hash(x, y, 126) - 0.5) * 14.0;
            [148.0 + facet + n, 102.0 + facet * 0.8 + n, 206.0 + facet + n, 255.0]
        }),
    );
    add_cube_all(
        &mut pack,
        "obsidian",
        make_tex(|x, y| {
            let glint = if tex_hash(x, y, 127) > 0.84 { 32.0 } else { 0.0 };
            let n = (tex_hash(x, y, 128) - 0.5) * 12.0;
            [24.0 + glint + n, 18.0 + glint * 0.5 + n, 40.0 + glint * 1.4 + n, 255.0]
        }),
    );
    add_cube_all(
        &mut pack,
        "sea_lantern",
        make_tex(|x, y| {
            let rim = x <= 1 || y <= 1 || x >= 14 || y >= 14;
            let n = (tex_hash(x, y, 129) - 0.5) * 10.0;
            if rim {
                [138.0 + n, 178.0 + n, 174.0 + n, 255.0]
            } else {
                [206.0 + n, 240.0 + n, 234.0 + n, 255.0]
            }
        }),
    );
    add_cube_all(
        &mut pack,
        "glowstone",
        make_tex(|x, y| {
            let b = if tex_hash(x >> 1, y >> 1, 130) > 0.45 { 24.0 } else { -24.0 };
            [232.0 + b * 0.6, 192.0 + b, 96.0 + b, 255.0]
        }),
    );
    add_cube_all(&mut pack, "lapis_block", solid_speckle([42.0, 82.0, 168.0], 20.0, 131));
    add_cube_all(
        &mut pack,
        "gold_block",
        make_tex(|x, y| {
            let rim = x == 0 || y == 0 || x == 15 || y == 15;
            let hi = (x == 1 || y == 1) && x < 15 && y < 15;
            let lo = (x == 14 || y == 14) && x > 0 && y > 0;
            let n = (tex_hash(x, y, 132) - 0.5) * 10.0;
            if rim {
                [214.0 + n, 162.0 + n, 36.0 + n, 255.0]
            } else if hi {
                [255.0, 238.0, 128.0, 255.0]
            } else if lo {
                [226.0 + n, 178.0 + n, 44.0 + n, 255.0]
            } else {
                [248.0 + n, 212.0 + n, 68.0 + n, 255.0]
            }
        }),
    );
    add_cube_all(
        &mut pack,
        "iron_block",
        make_tex(|x, y| {
            let rim = x == 0 || y == 0 || x == 15 || y == 15;
            let hi = (x == 1 || y == 1) && x < 15 && y < 15;
            let n = (tex_hash(x, y, 135) - 0.5) * 8.0;
            if rim {
                [178.0 + n, 182.0 + n, 186.0 + n, 255.0]
            } else if hi {
                [242.0, 244.0, 246.0, 255.0]
            } else {
                [222.0 + n, 224.0 + n, 228.0 + n, 255.0]
            }
        }),
    );
    add_cube_all(&mut pack, "diamond_block", solid_speckle([98.0, 236.0, 228.0], 14.0, 136));
    add_cube_all(&mut pack, "emerald_block", solid_speckle([52.0, 206.0, 102.0], 16.0, 137));
    add_cube_all(
        &mut pack,
        "redstone_block",
        make_tex(|x, y| {
            let rim = x == 0 || y == 0 || x == 15 || y == 15;
            let facet = ((x + y) % 5 == 0) || ((x + 16 - y) % 7 == 0);
            let n = (tex_hash(x, y, 138) - 0.5) * 14.0;
            if rim {
                [142.0 + n, 18.0 + n * 0.4, 14.0 + n * 0.4, 255.0]
            } else if facet {
                [238.0 + n, 46.0 + n * 0.5, 26.0 + n * 0.5, 255.0]
            } else {
                [198.0 + n, 26.0 + n * 0.4, 18.0 + n * 0.4, 255.0]
            }
        }),
    );
    add_cube_all(
        &mut pack,
        "slime_block",
        make_tex(|x, y| {
            let core = (3..=12).contains(&x) && (3..=12).contains(&y);
            let n = (tex_hash(x, y, 139) - 0.5) * 12.0;
            if core {
                [112.0 + n, 198.0 + n, 88.0 + n, 232.0]
            } else {
                [136.0 + n, 218.0 + n, 108.0 + n, 175.0]
            }
        }),
    );
    add_cube_all(&mut pack, "honey_block", solid_speckle([242.0, 182.0, 48.0], 14.0, 141));
    add_cube_all(&mut pack, "netherrack", cobble_tex([112.0, 46.0, 46.0], [76.0, 28.0, 28.0], 133));
    add_cube_all(&mut pack, "blackstone", cobble_tex([44.0, 40.0, 48.0], [26.0, 24.0, 30.0], 134));
    add_cube_all(
        &mut pack,
        "magma_block",
        make_tex(|x, y| {
            let crack = (x + y * 2) % 7 == 0 || (x * 2 + 32 - y) % 9 == 0;
            if crack {
                [246.0, 146.0, 36.0, 255.0]
            } else {
                [58.0, 30.0, 32.0, 255.0]
            }
        }),
    );

    // 5. Authentic multi-element 3D Redstone components (for tick-sim & redstone viewers)
    // 5a. Redstone lamp (dark lattice frame + warm amber / radiant golden glass panes)
    let lamp_off = make_tex(|x, y| {
        let border = x == 0 || y == 0 || x == 15 || y == 15;
        let lattice = (x + y) == 7 || (x + y) == 8 || (x + y) == 23 || (x + y) == 22
            || x.abs_diff(y) == 8 || (x == 7 && y == 7) || (x == 8 && y == 8);
        let n = (tex_hash(x, y, 145) - 0.5) * 10.0;
        if border {
            [56.0 + n * 0.5, 34.0 + n * 0.4, 20.0 + n * 0.3, 255.0]
        } else if lattice {
            [72.0 + n * 0.5, 44.0 + n * 0.4, 26.0 + n * 0.3, 255.0]
        } else {
            let cx = (x as f32 - 7.5).abs();
            let cy = (y as f32 - 7.5).abs();
            let glow = (1.0 - (cx.max(cy) / 8.0)) * 22.0;
            [116.0 + glow + n, 76.0 + glow * 0.7 + n, 44.0 + glow * 0.4 + n, 255.0]
        }
    });
    let lamp_on = make_tex(|x, y| {
        let border = x == 0 || y == 0 || x == 15 || y == 15;
        let lattice = (x + y) == 7 || (x + y) == 8 || (x + y) == 23 || (x + y) == 22
            || x.abs_diff(y) == 8;
        let n = (tex_hash(x, y, 146) - 0.5) * 8.0;
        if border {
            [156.0 + n, 96.0 + n * 0.7, 42.0 + n * 0.4, 255.0]
        } else if lattice {
            [194.0 + n, 128.0 + n * 0.7, 58.0 + n * 0.5, 255.0]
        } else {
            let cx = (x as f32 - 7.5).abs();
            let cy = (y as f32 - 7.5).abs();
            let bloom = (1.0 - (cx * cx + cy * cy).sqrt() / 11.0).max(0.0) * 32.0;
            [248.0 + bloom * 0.3, 212.0 + bloom * 0.9, 124.0 + bloom * 1.4, 255.0]
        }
    });
    let _ = pack.add_model_json(
        "minecraft:block/redstone_lamp",
        r##"{"textures":{"all":"minecraft:block/redstone_lamp"},"elements":[{"from":[0,0,0],"to":[16,16,16],"faces":{"down":{"texture":"#all","cullface":"down"},"up":{"texture":"#all","cullface":"up"},"north":{"texture":"#all","cullface":"north"},"south":{"texture":"#all","cullface":"south"},"west":{"texture":"#all","cullface":"west"},"east":{"texture":"#all","cullface":"east"}}}]}"##,
    );
    let _ = pack.add_model_json(
        "minecraft:block/redstone_lamp_on",
        r##"{"textures":{"all":"minecraft:block/redstone_lamp_on"},"elements":[{"from":[0,0,0],"to":[16,16,16],"faces":{"down":{"texture":"#all","cullface":"down"},"up":{"texture":"#all","cullface":"up"},"north":{"texture":"#all","cullface":"north"},"south":{"texture":"#all","cullface":"south"},"west":{"texture":"#all","cullface":"west"},"east":{"texture":"#all","cullface":"east"}}}]}"##,
    );
    let _ = pack.add_texture("minecraft:block/redstone_lamp", 16, 16, lamp_off);
    let _ = pack.add_texture("minecraft:block/redstone_lamp_on", 16, 16, lamp_on);
    let _ = pack.add_blockstate_json(
        "minecraft:redstone_lamp",
        r##"{"variants":{"lit=false":{"model":"minecraft:block/redstone_lamp"},"lit=true":{"model":"minecraft:block/redstone_lamp_on"}}}"##,
    );

    // Shared redstone torch head & stick textures
    let rs_wood = solid_speckle([132.0, 98.0, 62.0], 10.0, 160);
    let rs_head_off = solid_speckle([98.0, 22.0, 18.0], 8.0, 161);
    let rs_head_on = make_tex(|x, y| {
        let hot = (5..=10).contains(&x) && (5..=10).contains(&y);
        if hot { [255.0, 210.0, 96.0, 255.0] } else { [252.0, 48.0, 28.0, 255.0] }
    });
    let _ = pack.add_texture("minecraft:block/rs_wood", 16, 16, rs_wood);
    let _ = pack.add_texture("minecraft:block/rs_head_off", 16, 16, rs_head_off);
    let _ = pack.add_texture("minecraft:block/rs_head_on", 16, 16, rs_head_on);

    // 5b. Redstone wire: true Minecraft multipart directional dust (straight lines stay straight!)
    let dust_tiers: [(&str, [f32; 3], bool); 4] = [
        ("t0", [84.0, 12.0, 12.0], false),
        ("t1", [158.0, 18.0, 14.0], false),
        ("t2", [218.0, 26.0, 18.0], false),
        ("t3", [255.0, 42.0, 24.0], true),
    ];
    for (tag, rgb, sparks) in dust_tiers {
        let tex_name = format!("minecraft:block/rs_dust_{tag}");
        let px = make_tex(|x, y| {
            let n = (tex_hash(x, y, 170) - 0.5) * 26.0;
            if sparks && tex_hash(x, y, 171) > 0.86 {
                return [255.0, 168.0, 64.0, 255.0];
            }
            [rgb[0] + n, rgb[1] + n * 0.4, rgb[2] + n * 0.4, 255.0]
        });
        let _ = pack.add_texture(&tex_name, 16, 16, px);
        let parts: [(&str, [f32; 3], [f32; 3]); 5] = [
            ("dot", [6.0, 0.0, 6.0], [10.0, 0.6, 10.0]),
            ("n", [6.5, 0.0, 0.0], [9.5, 0.6, 6.5]),
            ("s", [6.5, 0.0, 9.5], [9.5, 0.6, 16.0]),
            ("w", [0.0, 0.0, 6.5], [6.5, 0.6, 9.5]),
            ("e", [9.5, 0.0, 6.5], [16.0, 0.6, 9.5]),
        ];
        for (pname, f, t) in parts {
            let mname = format!("minecraft:block/rs_wire_{pname}_{tag}");
            let mjson = format!(
                r##"{{"textures":{{"d":"{tex_name}"}},"elements":[{{"from":[{},{},{}],"to":[{},{},{}],"shade":false,"faces":{{"up":{{"texture":"#d"}},"north":{{"texture":"#d"}},"south":{{"texture":"#d"}},"west":{{"texture":"#d"}},"east":{{"texture":"#d"}}}}}}]}}"##,
                f[0], f[1], f[2], t[0], t[1], t[2]
            );
            let _ = pack.add_model_json(&mname, &mjson);
        }
    }
    {
        let mut mp = Vec::new();
        let tier_powers: [(&str, &str); 4] = [
            ("t0", "0"),
            ("t1", "1|2|3|4|5"),
            ("t2", "6|7|8|9|10|11"),
            ("t3", "12|13|14|15"),
        ];
        for (tag, pvals) in tier_powers {
            mp.push(format!(
                r##"{{"when":{{"power":"{pvals}"}},"apply":{{"model":"minecraft:block/rs_wire_dot_{tag}"}}}}"##
            ));
            for (dir, pshort) in [("north", "n"), ("south", "s"), ("west", "w"), ("east", "e")] {
                mp.push(format!(
                    r##"{{"when":{{"{dir}":"side|up","power":"{pvals}"}},"apply":{{"model":"minecraft:block/rs_wire_{pshort}_{tag}"}}}}"##
                ));
            }
        }
        let bs = format!(r##"{{"multipart":[{}]}}"##, mp.join(","));
        let _ = pack.add_blockstate_json("minecraft:redstone_wire", &bs);
    }

    // 5c. Repeater: 3D stone slab with directional arrow + 2 genuine 3D redstone torch pegs!
    let rep_side = make_tex(|x, y| {
        let n = (tex_hash(x, y, 175) - 0.5) * 10.0;
        let b = if y < 2 { 134.0 } else { 118.0 };
        [b + n, b + 1.0 + n, b + 3.0 + n, 255.0]
    });
    let _ = pack.add_texture("minecraft:block/repeater_side", 16, 16, rep_side);
    for (suffix, lit) in [("off", false), ("on", true)] {
        let top_px = make_tex(|x, y| {
            let n = (tex_hash(x, y, 176) - 0.5) * 8.0;
            let border = x == 0 || y == 0 || x == 15 || y == 15;
            let track = (x == 7 || x == 8) && (5..=13).contains(&y);
            // Directional arrow pointing North (toward y = 1)
            let dx = (x as i32 - 7).abs().min((x as i32 - 8).abs());
            let arrow = (2..=6).contains(&y) && dx <= (y as i32 - 2);
            if border {
                [108.0 + n, 110.0 + n, 114.0 + n, 255.0]
            } else if track {
                if lit { [212.0, 38.0, 26.0, 255.0] } else { [82.0, 74.0, 74.0, 255.0] }
            } else if arrow {
                if lit { [228.0, 56.0, 36.0, 255.0] } else { [122.0, 124.0, 128.0, 255.0] }
            } else {
                [162.0 + n, 164.0 + n, 167.0 + n, 255.0]
            }
        });
        let top_tex = format!("minecraft:block/repeater_top_{suffix}");
        let _ = pack.add_texture(&top_tex, 16, 16, top_px);
        let head_tex = if lit {
            "minecraft:block/rs_head_on"
        } else {
            "minecraft:block/rs_head_off"
        };
        for delay in 1..=4u32 {
            let z2 = 4 + delay * 2; // 6, 8, 10, 12
            let model_name = format!("minecraft:block/repeater_{delay}tick_{suffix}");
            let model_json = format!(
                r##"{{"textures":{{"top":"{top_tex}","side":"minecraft:block/repeater_side","wood":"minecraft:block/rs_wood","head":"{head_tex}"}},"elements":[{{"from":[0,0,0],"to":[16,2,16],"faces":{{"down":{{"texture":"#side","cullface":"down"}},"up":{{"texture":"#top"}},"north":{{"texture":"#side"}},"south":{{"texture":"#side"}},"west":{{"texture":"#side"}},"east":{{"texture":"#side"}}}}}},{{"from":[7,2,2],"to":[9,5,4],"faces":{{"north":{{"texture":"#wood"}},"south":{{"texture":"#wood"}},"west":{{"texture":"#wood"}},"east":{{"texture":"#wood"}}}}}},{{"from":[7,5,2],"to":[9,7,4],"faces":{{"up":{{"texture":"#head"}},"north":{{"texture":"#head"}},"south":{{"texture":"#head"}},"west":{{"texture":"#head"}},"east":{{"texture":"#head"}}}}}},{{"from":[7,2,{z2}],"to":[9,5,{}],"faces":{{"north":{{"texture":"#wood"}},"south":{{"texture":"#wood"}},"west":{{"texture":"#wood"}},"east":{{"texture":"#wood"}}}}}},{{"from":[7,5,{z2}],"to":[9,7,{}],"faces":{{"up":{{"texture":"#head"}},"north":{{"texture":"#head"}},"south":{{"texture":"#head"}},"west":{{"texture":"#head"}},"east":{{"texture":"#head"}}}}}}]}}"##,
                z2 + 2,
                z2 + 2
            );
            let _ = pack.add_model_json(&model_name, &model_json);
        }
    }
    {
        let mut variants = Vec::new();
        for (facing, rot_y) in [("south", 0), ("west", 90), ("north", 180), ("east", 270)] {
            for delay in 1..=4 {
                for (powered, suf) in [("false", "off"), ("true", "on")] {
                    variants.push(format!(
                        r##""delay={delay},facing={facing},powered={powered}":{{"model":"minecraft:block/repeater_{delay}tick_{suf}","y":{rot_y}}}"##
                    ));
                }
            }
        }
        let bs = format!(r##"{{"variants":{{{}}}}}"##, variants.join(","));
        let _ = pack.add_blockstate_json("minecraft:repeater", &bs);
    }

    // 5d. Lever: cobblestone base + angled 3D oak stick handle (-45° off / +45° on)!
    for (suf, angle, tip_tex) in [
        ("off", -45.0f32, "minecraft:block/cobblestone"),
        ("on", 45.0f32, "minecraft:block/rs_head_on"),
    ] {
        let model_name = format!("minecraft:block/lever_{suf}");
        let model_json = format!(
            r##"{{"textures":{{"base":"minecraft:block/cobblestone","wood":"minecraft:block/rs_wood","tip":"{tip_tex}"}},"elements":[{{"from":[5,0,4],"to":[11,3,12],"faces":{{"down":{{"texture":"#base"}},"up":{{"texture":"#base"}},"north":{{"texture":"#base"}},"south":{{"texture":"#base"}},"west":{{"texture":"#base"}},"east":{{"texture":"#base"}}}}}},{{"from":[7,1,7],"to":[9,10,9],"rotation":{{"origin":[8,1,8],"axis":"x","angle":{angle}}},"faces":{{"up":{{"texture":"#wood"}},"down":{{"texture":"#wood"}},"north":{{"texture":"#wood"}},"south":{{"texture":"#wood"}},"west":{{"texture":"#wood"}},"east":{{"texture":"#wood"}}}}}},{{"from":[6.8,9.8,6.8],"to":[9.2,11.6,9.2],"rotation":{{"origin":[8,1,8],"axis":"x","angle":{angle}}},"faces":{{"up":{{"texture":"#tip"}},"down":{{"texture":"#tip"}},"north":{{"texture":"#tip"}},"south":{{"texture":"#tip"}},"west":{{"texture":"#tip"}},"east":{{"texture":"#tip"}}}}}}]}}"##
        );
        let _ = pack.add_model_json(&model_name, &model_json);
    }
    {
        let mut variants = Vec::new();
        for (facing, rot_y) in [("north", 0), ("east", 90), ("south", 180), ("west", 270)] {
            for (powered, suf) in [("false", "off"), ("true", "on")] {
                variants.push(format!(
                    r##""facing={facing},powered={powered}":{{"model":"minecraft:block/lever_{suf}","y":{rot_y}}}"##
                ));
            }
        }
        let bs = format!(r##"{{"variants":{{{}}}}}"##, variants.join(","));
        let _ = pack.add_blockstate_json("minecraft:lever", &bs);
    }

    // 5e. Redstone torch (floor + 22.5°-angled wall torch!)
    for (suf, head_tex) in [
        ("off", "minecraft:block/rs_head_off"),
        ("on", "minecraft:block/rs_head_on"),
    ] {
        let floor_m = format!(
            r##"{{"textures":{{"wood":"minecraft:block/rs_wood","head":"{head_tex}"}},"elements":[{{"from":[7,0,7],"to":[9,8,9],"faces":{{"down":{{"texture":"#wood"}},"north":{{"texture":"#wood"}},"south":{{"texture":"#wood"}},"west":{{"texture":"#wood"}},"east":{{"texture":"#wood"}}}}}},{{"from":[7,8,7],"to":[9,10,9],"faces":{{"up":{{"texture":"#head"}},"down":{{"texture":"#head"}},"north":{{"texture":"#head"}},"south":{{"texture":"#head"}},"west":{{"texture":"#head"}},"east":{{"texture":"#head"}}}}}}]}}"##
        );
        let _ = pack.add_model_json(&format!("minecraft:block/redstone_torch_{suf}"), &floor_m);

        let wall_m = format!(
            r##"{{"textures":{{"wood":"minecraft:block/rs_wood","head":"{head_tex}"}},"elements":[{{"from":[7,3,12],"to":[9,11,14],"rotation":{{"origin":[8,3,13],"axis":"x","angle":-22.5}},"faces":{{"down":{{"texture":"#wood"}},"north":{{"texture":"#wood"}},"south":{{"texture":"#wood"}},"west":{{"texture":"#wood"}},"east":{{"texture":"#wood"}}}}}},{{"from":[7,11,12],"to":[9,13,14],"rotation":{{"origin":[8,3,13],"axis":"x","angle":-22.5}},"faces":{{"up":{{"texture":"#head"}},"down":{{"texture":"#head"}},"north":{{"texture":"#head"}},"south":{{"texture":"#head"}},"west":{{"texture":"#head"}},"east":{{"texture":"#head"}}}}}}]}}"##
        );
        let _ = pack.add_model_json(&format!("minecraft:block/redstone_wall_torch_{suf}"), &wall_m);
    }
    let _ = pack.add_blockstate_json(
        "minecraft:redstone_torch",
        r##"{"variants":{"lit=false":{"model":"minecraft:block/redstone_torch_off"},"lit=true":{"model":"minecraft:block/redstone_torch_on"}}}"##,
    );
    {
        let mut variants = Vec::new();
        for (facing, rot_y) in [("north", 0), ("east", 90), ("south", 180), ("west", 270)] {
            for (lit, suf) in [("false", "off"), ("true", "on")] {
                variants.push(format!(
                    r##""facing={facing},lit={lit}":{{"model":"minecraft:block/redstone_wall_torch_{suf}","y":{rot_y}}}"##
                ));
            }
        }
        let bs = format!(r##"{{"variants":{{{}}}}}"##, variants.join(","));
        let _ = pack.add_blockstate_json("minecraft:redstone_wall_torch", &bs);
    }

    // 5f. Stone & Oak buttons (crisp 6x2x4 button, depresses to 1px when pressed!)
    for (bname, btex) in [
        ("stone_button", "minecraft:block/stone"),
        ("oak_button", "minecraft:block/oak_planks"),
    ] {
        let _ = pack.add_model_json(
            &format!("minecraft:block/{bname}_off"),
            &format!(
                r##"{{"textures":{{"all":"{btex}"}},"elements":[{{"from":[5,0,6],"to":[11,2,10],"faces":{{"down":{{"texture":"#all"}},"up":{{"texture":"#all"}},"north":{{"texture":"#all"}},"south":{{"texture":"#all"}},"west":{{"texture":"#all"}},"east":{{"texture":"#all"}}}}}}]}}"##
            ),
        );
        let _ = pack.add_model_json(
            &format!("minecraft:block/{bname}_on"),
            &format!(
                r##"{{"textures":{{"all":"{btex}"}},"elements":[{{"from":[5,0,6],"to":[11,1,10],"faces":{{"down":{{"texture":"#all"}},"up":{{"texture":"#all"}},"north":{{"texture":"#all"}},"south":{{"texture":"#all"}},"west":{{"texture":"#all"}},"east":{{"texture":"#all"}}}}}}]}}"##
            ),
        );
        let mut variants = Vec::new();
        for (face, rx) in [("floor", 0), ("wall", 90), ("ceiling", 180)] {
            for (facing, ry) in [("north", 0), ("east", 90), ("south", 180), ("west", 270)] {
                for (powered, suf) in [("false", "off"), ("true", "on")] {
                    variants.push(format!(
                        r##""face={face},facing={facing},powered={powered}":{{"model":"minecraft:block/{bname}_{suf}","x":{rx},"y":{ry}}}"##
                    ));
                }
            }
        }
        let bs = format!(r##"{{"variants":{{{}}}}}"##, variants.join(","));
        let _ = pack.add_blockstate_json(&format!("minecraft:{bname}"), &bs);
    }

    // 5f2. Observer (iconic stone face on front, arrow on top, glowing redstone dot on back!)
    let obs_front = make_tex(|x, y| {
        let border = x == 0 || y == 0 || x == 15 || y == 15;
        let brow = y == 4 && (2..=13).contains(&x);
        let eye = (y == 6 || y == 7) && ((3..=5).contains(&x) || (10..=12).contains(&x));
        let pupil = y == 7 && (x == 4 || x == 11);
        let mouth = (10..=12).contains(&y) && (5..=10).contains(&x);
        let n = (tex_hash(x, y, 185) - 0.5) * 10.0;
        if pupil {
            [18.0, 18.0, 20.0, 255.0]
        } else if eye {
            [218.0, 222.0, 226.0, 255.0]
        } else if brow || mouth {
            [42.0 + n * 0.5, 44.0 + n * 0.5, 48.0 + n * 0.5, 255.0]
        } else if border {
            [68.0 + n, 70.0 + n, 74.0 + n, 255.0]
        } else {
            [114.0 + n, 116.0 + n, 120.0 + n, 255.0]
        }
    });
    let obs_top = make_tex(|x, y| {
        let border = x == 0 || y == 0 || x == 15 || y == 15;
        let dx = (x as i32 - 7).abs().min((x as i32 - 8).abs());
        let arrow = (3..=9).contains(&y) && dx <= (y as i32 - 3).min(4);
        let stem = (10..=12).contains(&y) && (x == 7 || x == 8);
        let n = (tex_hash(x, y, 186) - 0.5) * 10.0;
        if arrow || stem {
            [62.0 + n, 64.0 + n, 68.0 + n, 255.0]
        } else if border {
            [72.0 + n, 74.0 + n, 78.0 + n, 255.0]
        } else {
            [122.0 + n, 124.0 + n, 128.0 + n, 255.0]
        }
    });
    let obs_side = cobble_tex([96.0, 98.0, 102.0], [64.0, 66.0, 70.0], 187);
    let obs_back_off = make_tex(|x, y| {
        let dot = (6..=9).contains(&x) && (6..=9).contains(&y);
        let n = (tex_hash(x, y, 188) - 0.5) * 10.0;
        if dot {
            [116.0, 26.0, 22.0, 255.0]
        } else {
            [86.0 + n, 88.0 + n, 92.0 + n, 255.0]
        }
    });
    let obs_back_on = make_tex(|x, y| {
        let dot = (5..=10).contains(&x) && (5..=10).contains(&y);
        let hot = (6..=9).contains(&x) && (6..=9).contains(&y);
        let n = (tex_hash(x, y, 189) - 0.5) * 10.0;
        if hot {
            [255.0, 72.0, 42.0, 255.0]
        } else if dot {
            [216.0, 34.0, 24.0, 255.0]
        } else {
            [86.0 + n, 88.0 + n, 92.0 + n, 255.0]
        }
    });
    let _ = pack.add_texture("minecraft:block/observer_front", 16, 16, obs_front);
    let _ = pack.add_texture("minecraft:block/observer_top", 16, 16, obs_top);
    let _ = pack.add_texture("minecraft:block/observer_side", 16, 16, obs_side);
    let _ = pack.add_texture("minecraft:block/observer_back", 16, 16, obs_back_off);
    let _ = pack.add_texture("minecraft:block/observer_back_on", 16, 16, obs_back_on);
    for (suf, back_tex) in [
        ("off", "minecraft:block/observer_back"),
        ("on", "minecraft:block/observer_back_on"),
    ] {
        let mjson = format!(
            r##"{{"textures":{{"front":"minecraft:block/observer_front","back":"{back_tex}","top":"minecraft:block/observer_top","side":"minecraft:block/observer_side"}},"elements":[{{"from":[0,0,0],"to":[16,16,16],"faces":{{"north":{{"texture":"#front","cullface":"north"}},"south":{{"texture":"#back","cullface":"south"}},"up":{{"texture":"#top","cullface":"up"}},"down":{{"texture":"#top","cullface":"down"}},"west":{{"texture":"#side","cullface":"west"}},"east":{{"texture":"#side","cullface":"east"}}}}}}]}}"##
        );
        let _ = pack.add_model_json(&format!("minecraft:block/observer_{suf}"), &mjson);
    }
    {
        let mut variants = Vec::new();
        for (facing, rx, ry) in [
            ("north", 0, 0),
            ("east", 0, 90),
            ("south", 0, 180),
            ("west", 0, 270),
            ("up", 270, 0),
            ("down", 90, 0),
        ] {
            for (powered, suf) in [("false", "off"), ("true", "on")] {
                variants.push(format!(
                    r##""facing={facing},powered={powered}":{{"model":"minecraft:block/observer_{suf}","x":{rx},"y":{ry}}}"##
                ));
            }
        }
        let bs = format!(r##"{{"variants":{{{}}}}}"##, variants.join(","));
        let _ = pack.add_blockstate_json("minecraft:observer", &bs);
    }

    // 5g. Sticky piston, regular piston, and 3D T-shaped piston_head!
    let piston_side = make_tex(|x, y| {
        let n = (tex_hash(x, y, 180) - 0.5) * 14.0;
        if y < 4 {
            // Oak wooden rim near piston face
            [168.0 + n, 134.0 + n, 84.0 + n, 255.0]
        } else if y == 4 || y == 5 {
            // Dark iron band
            [74.0 + n * 0.5, 76.0 + n * 0.5, 80.0 + n * 0.5, 255.0]
        } else {
            // Cobblestone housing
            [118.0 + n, 120.0 + n, 122.0 + n, 255.0]
        }
    });
    let piston_face_regular = planks_tex([174.0, 140.0, 88.0], [122.0, 92.0, 54.0], 181);
    let piston_face_sticky = make_tex(|x, y| {
        let slime = (3..=12).contains(&x) && (3..=12).contains(&y) && tex_hash(x >> 1, y >> 1, 182) > 0.22;
        let n = (tex_hash(x, y, 183) - 0.5) * 16.0;
        if slime {
            [112.0 + n, 194.0 + n, 84.0 + n, 255.0]
        } else {
            [168.0 + n, 134.0 + n, 84.0 + n, 255.0]
        }
    });
    let _ = pack.add_texture("minecraft:block/piston_side", 16, 16, piston_side);
    let _ = pack.add_texture("minecraft:block/piston_top", 16, 16, piston_face_regular);
    let _ = pack.add_texture("minecraft:block/piston_top_sticky", 16, 16, piston_face_sticky);

    for (kind, face_tex) in [
        ("piston", "minecraft:block/piston_top"),
        ("sticky_piston", "minecraft:block/piston_top_sticky"),
    ] {
        // Unrotated faces North (-Z)
        let retracted = format!(
            r##"{{"textures":{{"face":"{face_tex}","side":"minecraft:block/piston_side","back":"minecraft:block/cobblestone"}},"elements":[{{"from":[0,0,0],"to":[16,16,16],"faces":{{"north":{{"texture":"#face"}},"south":{{"texture":"#back","cullface":"south"}},"up":{{"texture":"#side","cullface":"up"}},"down":{{"texture":"#side","cullface":"down"}},"west":{{"texture":"#side","cullface":"west"}},"east":{{"texture":"#side","cullface":"east"}}}}}}]}}"##
        );
        let extended = format!(
            r##"{{"textures":{{"side":"minecraft:block/piston_side","back":"minecraft:block/cobblestone"}},"elements":[{{"from":[0,0,4],"to":[16,16,16],"faces":{{"north":{{"texture":"#back"}},"south":{{"texture":"#back","cullface":"south"}},"up":{{"texture":"#side"}},"down":{{"texture":"#side"}},"west":{{"texture":"#side"}},"east":{{"texture":"#side"}}}}}}]}}"##
        );
        let _ = pack.add_model_json(&format!("minecraft:block/{kind}_retracted"), &retracted);
        let _ = pack.add_model_json(&format!("minecraft:block/{kind}_extended"), &extended);

        let mut variants = Vec::new();
        for (facing, rx, ry) in [
            ("north", 0, 0),
            ("east", 0, 90),
            ("south", 0, 180),
            ("west", 0, 270),
            ("up", 270, 0),
            ("down", 90, 0),
        ] {
            variants.push(format!(
                r##""extended=false,facing={facing}":{{"model":"minecraft:block/{kind}_retracted","x":{rx},"y":{ry}}}"##
            ));
            variants.push(format!(
                r##""extended=true,facing={facing}":{{"model":"minecraft:block/{kind}_extended","x":{rx},"y":{ry}}}"##
            ));
        }
        let bs = format!(r##"{{"variants":{{{}}}}}"##, variants.join(","));
        let _ = pack.add_blockstate_json(&format!("minecraft:{kind}"), &bs);
    }

    for (kind, face_tex) in [
        ("normal", "minecraft:block/piston_top"),
        ("sticky", "minecraft:block/piston_top_sticky"),
    ] {
        let head = format!(
            r##"{{"textures":{{"face":"{face_tex}","wood":"minecraft:block/oak_planks"}},"elements":[{{"from":[0,0,0],"to":[16,16,4],"faces":{{"north":{{"texture":"#face"}},"south":{{"texture":"#wood"}},"up":{{"texture":"#wood"}},"down":{{"texture":"#wood"}},"west":{{"texture":"#wood"}},"east":{{"texture":"#wood"}}}}}},{{"from":[6,6,4],"to":[10,10,20],"faces":{{"up":{{"texture":"#wood"}},"down":{{"texture":"#wood"}},"west":{{"texture":"#wood"}},"east":{{"texture":"#wood"}}}}}}]}}"##
        );
        let head_short = format!(
            r##"{{"textures":{{"face":"{face_tex}","wood":"minecraft:block/oak_planks"}},"elements":[{{"from":[0,0,0],"to":[16,16,4],"faces":{{"north":{{"texture":"#face"}},"south":{{"texture":"#wood"}},"up":{{"texture":"#wood"}},"down":{{"texture":"#wood"}},"west":{{"texture":"#wood"}},"east":{{"texture":"#wood"}}}}}},{{"from":[6,6,4],"to":[10,10,16],"faces":{{"up":{{"texture":"#wood"}},"down":{{"texture":"#wood"}},"west":{{"texture":"#wood"}},"east":{{"texture":"#wood"}}}}}}]}}"##
        );
        let _ = pack.add_model_json(&format!("minecraft:block/piston_head_{kind}"), &head);
        let _ = pack.add_model_json(&format!("minecraft:block/piston_head_{kind}_short"), &head_short);
    }
    {
        let mut variants = Vec::new();
        for (facing, rx, ry) in [
            ("north", 0, 0),
            ("east", 0, 90),
            ("south", 0, 180),
            ("west", 0, 270),
            ("up", 270, 0),
            ("down", 90, 0),
        ] {
            for kind in ["normal", "sticky"] {
                variants.push(format!(
                    r##""facing={facing},short=false,type={kind}":{{"model":"minecraft:block/piston_head_{kind}","x":{rx},"y":{ry}}}"##
                ));
                variants.push(format!(
                    r##""facing={facing},short=true,type={kind}":{{"model":"minecraft:block/piston_head_{kind}_short","x":{rx},"y":{ry}}}"##
                ));
            }
        }
        let bs = format!(r##"{{"variants":{{{}}}}}"##, variants.join(","));
        let _ = pack.add_blockstate_json("minecraft:piston_head", &bs);
    }

    // 5h. Comparator (3D slab + triangle + 2 rear torch pegs + front compare/subtract peg)
    for (mode, front_lit) in [("compare", false), ("subtract", true)] {
        for (psuf, powered) in [("off", false), ("on", true)] {
            let top_px = make_tex(|x, y| {
                let n = (tex_hash(x, y, 190) - 0.5) * 8.0;
                let border = x == 0 || y == 0 || x == 15 || y == 15;
                // Triangle pointing North (toward y = 2)
                let dx = (x as i32 - 7).abs().min((x as i32 - 8).abs());
                let tri = (3..=11).contains(&y) && dx <= ((y as i32 - 3) / 2);
                if border {
                    [108.0 + n, 110.0 + n, 114.0 + n, 255.0]
                } else if tri {
                    if powered { [224.0, 48.0, 32.0, 255.0] } else { [126.0, 128.0, 132.0, 255.0] }
                } else {
                    [162.0 + n, 164.0 + n, 167.0 + n, 255.0]
                }
            });
            let top_tex = format!("minecraft:block/comparator_top_{mode}_{psuf}");
            let _ = pack.add_texture(&top_tex, 16, 16, top_px);
            let rear_head = if powered { "minecraft:block/rs_head_on" } else { "minecraft:block/rs_head_off" };
            let front_head = if front_lit { "minecraft:block/rs_head_on" } else { "minecraft:block/rs_head_off" };
            let front_h = if front_lit { 7 } else { 5 };
            let mjson = format!(
                r##"{{"textures":{{"top":"{top_tex}","side":"minecraft:block/repeater_side","wood":"minecraft:block/rs_wood","rhead":"{rear_head}","fhead":"{front_head}"}},"elements":[{{"from":[0,0,0],"to":[16,2,16],"faces":{{"down":{{"texture":"#side","cullface":"down"}},"up":{{"texture":"#top"}},"north":{{"texture":"#side"}},"south":{{"texture":"#side"}},"west":{{"texture":"#side"}},"east":{{"texture":"#side"}}}}}},{{"from":[4,2,11],"to":[6,5,13],"faces":{{"north":{{"texture":"#wood"}},"south":{{"texture":"#wood"}},"west":{{"texture":"#wood"}},"east":{{"texture":"#wood"}}}}}},{{"from":[4,5,11],"to":[6,7,13],"faces":{{"up":{{"texture":"#rhead"}},"north":{{"texture":"#rhead"}},"south":{{"texture":"#rhead"}},"west":{{"texture":"#rhead"}},"east":{{"texture":"#rhead"}}}}}},{{"from":[10,2,11],"to":[12,5,13],"faces":{{"north":{{"texture":"#wood"}},"south":{{"texture":"#wood"}},"west":{{"texture":"#wood"}},"east":{{"texture":"#wood"}}}}}},{{"from":[10,5,11],"to":[12,7,13],"faces":{{"up":{{"texture":"#rhead"}},"north":{{"texture":"#rhead"}},"south":{{"texture":"#rhead"}},"west":{{"texture":"#rhead"}},"east":{{"texture":"#rhead"}}}}}},{{"from":[7,2,2],"to":[9,{},4],"faces":{{"north":{{"texture":"#wood"}},"south":{{"texture":"#wood"}},"west":{{"texture":"#wood"}},"east":{{"texture":"#wood"}}}}}},{{"from":[7,{},2],"to":[9,{front_h},4],"faces":{{"up":{{"texture":"#fhead"}},"north":{{"texture":"#fhead"}},"south":{{"texture":"#fhead"}},"west":{{"texture":"#fhead"}},"east":{{"texture":"#fhead"}}}}}}]}}"##,
                front_h - 2,
                front_h - 2
            );
            let _ = pack.add_model_json(&format!("minecraft:block/comparator_{mode}_{psuf}"), &mjson);
        }
    }
    {
        let mut variants = Vec::new();
        for (facing, rot_y) in [("south", 0), ("west", 90), ("north", 180), ("east", 270)] {
            for mode in ["compare", "subtract"] {
                for (powered, psuf) in [("false", "off"), ("true", "on")] {
                    variants.push(format!(
                        r##""facing={facing},mode={mode},powered={powered}":{{"model":"minecraft:block/comparator_{mode}_{psuf}","y":{rot_y}}}"##
                    ));
                }
            }
        }
        let bs = format!(r##"{{"variants":{{{}}}}}"##, variants.join(","));
        let _ = pack.add_blockstate_json("minecraft:comparator", &bs);
    }

    // 5i. Target block (crisp red & white bullseye!)
    add_cube_all(
        &mut pack,
        "target",
        make_tex(|x, y| {
            let d = (x as i32 - 7).abs().max((x as i32 - 8).abs()).max((y as i32 - 7).abs().max((y as i32 - 8).abs()));
            let n = (tex_hash(x, y, 192) - 0.5) * 10.0;
            if d <= 1 || d == 4 || d == 5 {
                [214.0 + n, 44.0 + n * 0.4, 38.0 + n * 0.4, 255.0]
            } else {
                [238.0 + n, 224.0 + n, 206.0 + n, 255.0]
            }
        }),
    );

    // 5j. Note block (dark oak acoustic chamber with musical note icon)
    add_cube_all(
        &mut pack,
        "note_block",
        make_tex(|x, y| {
            let border = x <= 1 || y <= 1 || x >= 14 || y >= 14;
            let stem = x == 9 && (4..=10).contains(&y);
            let flag = y == 4 && (9..=11).contains(&x);
            let note = (x == 7 || x == 8) && (9..=11).contains(&y);
            let n = (tex_hash(x, y, 193) - 0.5) * 10.0;
            if stem || flag || note {
                [38.0, 22.0, 14.0, 255.0]
            } else if border {
                [82.0 + n, 52.0 + n, 34.0 + n, 255.0]
            } else {
                [118.0 + n, 76.0 + n, 48.0 + n, 255.0]
            }
        }),
    );

    // 5k. Dropper & Dispenser (stone body + distinctive face across all 6 facings)
    let furnace_side = cobble_tex([112.0, 114.0, 116.0], [78.0, 80.0, 84.0], 194);
    let _ = pack.add_texture("minecraft:block/furnace_side", 16, 16, furnace_side);
    for (kind, is_dispenser) in [("dropper", false), ("dispenser", true)] {
        let front_px = make_tex(|x, y| {
            let border = x == 0 || y == 0 || x == 15 || y == 15;
            let hole = if is_dispenser {
                ((5..=10).contains(&x) && (4..=8).contains(&y))
                    || ((7..=8).contains(&x) && (10..=12).contains(&y))
            } else {
                ((6..=9).contains(&x) && (5..=9).contains(&y))
                    || ((5..=10).contains(&x) && y == 11)
            };
            let n = (tex_hash(x, y, 195) - 0.5) * 10.0;
            if hole {
                [26.0, 26.0, 30.0, 255.0]
            } else if border {
                [84.0 + n, 86.0 + n, 90.0 + n, 255.0]
            } else {
                [122.0 + n, 124.0 + n, 128.0 + n, 255.0]
            }
        });
        let ftex = format!("minecraft:block/{kind}_front");
        let _ = pack.add_texture(&ftex, 16, 16, front_px);
        let mjson = format!(
            r##"{{"textures":{{"front":"{ftex}","side":"minecraft:block/furnace_side"}},"elements":[{{"from":[0,0,0],"to":[16,16,16],"faces":{{"north":{{"texture":"#front","cullface":"north"}},"south":{{"texture":"#side","cullface":"south"}},"up":{{"texture":"#side","cullface":"up"}},"down":{{"texture":"#side","cullface":"down"}},"west":{{"texture":"#side","cullface":"west"}},"east":{{"texture":"#side","cullface":"east"}}}}}}]}}"##
        );
        let _ = pack.add_model_json(&format!("minecraft:block/{kind}"), &mjson);
        let mut variants = Vec::new();
        for (facing, rx, ry) in [
            ("north", 0, 0),
            ("east", 0, 90),
            ("south", 0, 180),
            ("west", 0, 270),
            ("up", 270, 0),
            ("down", 90, 0),
        ] {
            variants.push(format!(
                r##""facing={facing}":{{"model":"minecraft:block/{kind}","x":{rx},"y":{ry}}}"##
            ));
        }
        let bs = format!(r##"{{"variants":{{{}}}}}"##, variants.join(","));
        let _ = pack.add_blockstate_json(&format!("minecraft:{kind}"), &bs);
    }

    // 5l. Hopper (authentic 3D metal funnel bowl + neck + directional spout)
    let hopper_px = solid_speckle([76.0, 78.0, 84.0], 10.0, 196);
    let _ = pack.add_texture("minecraft:block/hopper_metal", 16, 16, hopper_px);
    let hopper_down = r##"{"textures":{"m":"minecraft:block/hopper_metal"},"elements":[{"from":[0,10,0],"to":[16,16,2],"faces":{"up":{"texture":"#m"},"down":{"texture":"#m"},"north":{"texture":"#m"},"south":{"texture":"#m"},"west":{"texture":"#m"},"east":{"texture":"#m"}}},{"from":[0,10,14],"to":[16,16,16],"faces":{"up":{"texture":"#m"},"down":{"texture":"#m"},"north":{"texture":"#m"},"south":{"texture":"#m"},"west":{"texture":"#m"},"east":{"texture":"#m"}}},{"from":[0,10,2],"to":[2,16,14],"faces":{"up":{"texture":"#m"},"down":{"texture":"#m"},"west":{"texture":"#m"},"east":{"texture":"#m"}}},{"from":[14,10,2],"to":[16,16,14],"faces":{"up":{"texture":"#m"},"down":{"texture":"#m"},"west":{"texture":"#m"},"east":{"texture":"#m"}}},{"from":[2,10,2],"to":[14,11,14],"faces":{"up":{"texture":"#m"},"down":{"texture":"#m"}}},{"from":[4,4,4],"to":[12,10,12],"faces":{"down":{"texture":"#m"},"north":{"texture":"#m"},"south":{"texture":"#m"},"west":{"texture":"#m"},"east":{"texture":"#m"}}},{"from":[6,0,6],"to":[10,4,10],"faces":{"down":{"texture":"#m"},"north":{"texture":"#m"},"south":{"texture":"#m"},"west":{"texture":"#m"},"east":{"texture":"#m"}}}]}"##;
    let hopper_side = r##"{"textures":{"m":"minecraft:block/hopper_metal"},"elements":[{"from":[0,10,0],"to":[16,16,2],"faces":{"up":{"texture":"#m"},"down":{"texture":"#m"},"north":{"texture":"#m"},"south":{"texture":"#m"},"west":{"texture":"#m"},"east":{"texture":"#m"}}},{"from":[0,10,14],"to":[16,16,16],"faces":{"up":{"texture":"#m"},"down":{"texture":"#m"},"north":{"texture":"#m"},"south":{"texture":"#m"},"west":{"texture":"#m"},"east":{"texture":"#m"}}},{"from":[0,10,2],"to":[2,16,14],"faces":{"up":{"texture":"#m"},"down":{"texture":"#m"},"west":{"texture":"#m"},"east":{"texture":"#m"}}},{"from":[14,10,2],"to":[16,16,14],"faces":{"up":{"texture":"#m"},"down":{"texture":"#m"},"west":{"texture":"#m"},"east":{"texture":"#m"}}},{"from":[2,10,2],"to":[14,11,14],"faces":{"up":{"texture":"#m"},"down":{"texture":"#m"}}},{"from":[4,4,4],"to":[12,10,12],"faces":{"down":{"texture":"#m"},"north":{"texture":"#m"},"south":{"texture":"#m"},"west":{"texture":"#m"},"east":{"texture":"#m"}}},{"from":[6,4,0],"to":[10,8,4],"faces":{"up":{"texture":"#m"},"down":{"texture":"#m"},"north":{"texture":"#m"},"west":{"texture":"#m"},"east":{"texture":"#m"}}}]}"##;
    let _ = pack.add_model_json("minecraft:block/hopper_down", hopper_down);
    let _ = pack.add_model_json("minecraft:block/hopper_side", hopper_side);
    let _ = pack.add_blockstate_json(
        "minecraft:hopper",
        r##"{"variants":{"facing=down":{"model":"minecraft:block/hopper_down"},"facing=north":{"model":"minecraft:block/hopper_side","y":0},"facing=east":{"model":"minecraft:block/hopper_side","y":90},"facing=south":{"model":"minecraft:block/hopper_side","y":180},"facing=west":{"model":"minecraft:block/hopper_side","y":270}}}"##,
    );

    // 5m. Trapdoors (oak + iron, bottom / top / open across 4 facings)
    for (td, tex) in [
        ("oak_trapdoor", "minecraft:block/oak_planks"),
        ("iron_trapdoor", "minecraft:block/iron_block"),
    ] {
        let _ = pack.add_model_json(
            &format!("minecraft:block/{td}_bottom"),
            &format!(r##"{{"textures":{{"t":"{tex}"}},"elements":[{{"from":[0,0,0],"to":[16,3,16],"faces":{{"up":{{"texture":"#t"}},"down":{{"texture":"#t"}},"north":{{"texture":"#t"}},"south":{{"texture":"#t"}},"west":{{"texture":"#t"}},"east":{{"texture":"#t"}}}}}}]}}"##),
        );
        let _ = pack.add_model_json(
            &format!("minecraft:block/{td}_top"),
            &format!(r##"{{"textures":{{"t":"{tex}"}},"elements":[{{"from":[0,13,0],"to":[16,16,16],"faces":{{"up":{{"texture":"#t"}},"down":{{"texture":"#t"}},"north":{{"texture":"#t"}},"south":{{"texture":"#t"}},"west":{{"texture":"#t"}},"east":{{"texture":"#t"}}}}}}]}}"##),
        );
        let _ = pack.add_model_json(
            &format!("minecraft:block/{td}_open"),
            &format!(r##"{{"textures":{{"t":"{tex}"}},"elements":[{{"from":[0,0,13],"to":[16,16,16],"faces":{{"up":{{"texture":"#t"}},"down":{{"texture":"#t"}},"north":{{"texture":"#t"}},"south":{{"texture":"#t"}},"west":{{"texture":"#t"}},"east":{{"texture":"#t"}}}}}}]}}"##),
        );
        let mut variants = Vec::new();
        for (facing, ry) in [("north", 0), ("east", 90), ("south", 180), ("west", 270)] {
            variants.push(format!(r##""facing={facing},half=bottom,open=false":{{"model":"minecraft:block/{td}_bottom","y":{ry}}}"##));
            variants.push(format!(r##""facing={facing},half=top,open=false":{{"model":"minecraft:block/{td}_top","y":{ry}}}"##));
            variants.push(format!(r##""facing={facing},open=true":{{"model":"minecraft:block/{td}_open","y":{ry}}}"##));
        }
        let bs = format!(r##"{{"variants":{{{}}}}}"##, variants.join(","));
        let _ = pack.add_blockstate_json(&format!("minecraft:{td}"), &bs);
    }

    // 5n. Powered rail & regular rail
    for (rname, has_power) in [("rail", false), ("powered_rail", true)] {
        for (suf, lit) in if has_power { vec![("off", false), ("on", true)] } else { vec![("off", false)] } {
            let px = make_tex(|x, y| {
                let rail = x == 2 || x == 3 || x == 12 || x == 13;
                let tie = y % 4 == 1 || y % 4 == 2;
                let core = has_power && (x == 7 || x == 8);
                if rail {
                    if has_power { [226.0, 186.0, 62.0, 255.0] } else { [164.0, 168.0, 174.0, 255.0] }
                } else if core {
                    if lit { [252.0, 46.0, 26.0, 255.0] } else { [112.0, 24.0, 20.0, 255.0] }
                } else if tie && (1..=14).contains(&x) {
                    [136.0, 102.0, 64.0, 255.0]
                } else {
                    [0.0; 4]
                }
            });
            let tname = format!("minecraft:block/{rname}_{suf}");
            let _ = pack.add_texture(&tname, 16, 16, px);
            let mjson = format!(
                r##"{{"textures":{{"r":"{tname}"}},"elements":[{{"from":[0,0.4,0],"to":[16,0.4,16],"shade":false,"faces":{{"up":{{"texture":"#r"}},"down":{{"texture":"#r"}}}}}}]}}"##
            );
            let _ = pack.add_model_json(&format!("minecraft:block/{rname}_flat_{suf}"), &mjson);
        }
        let bs = if has_power {
            r##"{"variants":{"powered=false,shape=north_south":{"model":"minecraft:block/powered_rail_flat_off"},"powered=false,shape=east_west":{"model":"minecraft:block/powered_rail_flat_off","y":90},"powered=true,shape=north_south":{"model":"minecraft:block/powered_rail_flat_on"},"powered=true,shape=east_west":{"model":"minecraft:block/powered_rail_flat_on","y":90}}}"##
        } else {
            r##"{"variants":{"shape=north_south":{"model":"minecraft:block/rail_flat_off"},"shape=east_west":{"model":"minecraft:block/rail_flat_off","y":90}}}"##
        };
        let _ = pack.add_blockstate_json(&format!("minecraft:{rname}"), bs);
    }

    // 6. Fluids (water & lava)
    let water_px = make_tex(|x, y| {
        let wave = if (x + y * 2) % 6 < 2 { 18.0 } else { -8.0 };
        [52.0 + wave, 122.0 + wave, 212.0 + wave, 190.0]
    });
    let _ = pack.add_texture("minecraft:block/water_still", 16, 16, water_px.clone());
    let _ = pack.add_texture("minecraft:block/water_flow", 16, 16, water_px.clone());
    add_cube_all(&mut pack, "water", water_px);

    let lava_px = make_tex(|x, y| {
        let hot = tex_hash(x >> 1, y >> 1, 140) > 0.55;
        if hot { [250.0, 186.0, 48.0, 255.0] } else { [224.0, 92.0, 24.0, 255.0] }
    });
    let _ = pack.add_texture("minecraft:block/lava_still", 16, 16, lava_px.clone());
    let _ = pack.add_texture("minecraft:block/lava_flow", 16, 16, lava_px.clone());
    add_cube_all(&mut pack, "lava", lava_px);

    // 7. Flora
    add_cross_plant(
        &mut pack,
        "short_grass",
        make_tex(|x, y| {
            let blade = (x.abs_diff(4) <= 1 && y >= 6)
                || (x.abs_diff(8) <= 1 && y >= 3)
                || (x.abs_diff(12) <= 1 && y >= 7)
                || (y >= 12 && (3..=13).contains(&x));
            if blade {
                let hi = if y < 8 { 24.0 } else { 0.0 };
                [96.0 + hi, 162.0 + hi, 58.0, 255.0]
            } else {
                [0.0; 4]
            }
        }),
    );
    add_cross_plant(
        &mut pack,
        "poppy",
        make_tex(|x, y| {
            let petal = ((x as f32 - 8.0).powi(2) + (y as f32 - 5.0).powi(2)).sqrt() <= 2.6;
            let stem = x == 8 && (7..=15).contains(&y);
            let leaf = (x == 6 || x == 7) && (y == 11 || y == 12);
            if petal {
                [220.0, 44.0, 42.0, 255.0]
            } else if stem || leaf {
                [64.0, 134.0, 48.0, 255.0]
            } else {
                [0.0; 4]
            }
        }),
    );
    add_cross_plant(
        &mut pack,
        "dandelion",
        make_tex(|x, y| {
            let petal = ((x as f32 - 8.0).powi(2) + (y as f32 - 6.0).powi(2)).sqrt() <= 2.4;
            let stem = x == 8 && (8..=15).contains(&y);
            if petal {
                [248.0, 212.0, 48.0, 255.0]
            } else if stem {
                [68.0, 138.0, 48.0, 255.0]
            } else {
                [0.0; 4]
            }
        }),
    );

    // 8. 16-color Concrete, Wool & Terracotta spectra (great for Oklab palette mapping!)
    let colors: [(&str, [f32; 3]); 16] = [
        ("white", [214.0, 218.0, 220.0]),
        ("light_gray", [138.0, 140.0, 136.0]),
        ("gray", [64.0, 68.0, 72.0]),
        ("black", [22.0, 24.0, 30.0]),
        ("blue", [44.0, 52.0, 148.0]),
        ("cyan", [28.0, 136.0, 148.0]),
        ("light_blue", [52.0, 164.0, 208.0]),
        ("lime", [108.0, 178.0, 42.0]),
        ("green", [78.0, 102.0, 42.0]),
        ("yellow", [244.0, 194.0, 48.0]),
        ("orange", [226.0, 114.0, 32.0]),
        ("red", [164.0, 42.0, 40.0]),
        ("magenta", [178.0, 62.0, 168.0]),
        ("purple", [112.0, 44.0, 166.0]),
        ("pink", [218.0, 114.0, 152.0]),
        ("brown", [102.0, 66.0, 44.0]),
    ];
    for (idx, (color, rgb)) in colors.iter().enumerate() {
        let salt = 200 + idx as u32 * 7;
        add_cube_all(&mut pack, &format!("{color}_concrete"), solid_speckle(*rgb, 5.0, salt));
        add_cube_all(&mut pack, &format!("{color}_wool"), solid_speckle(*rgb, 14.0, salt + 1));
        let terra = [
            rgb[0] * 0.72 + 44.0,
            rgb[1] * 0.65 + 32.0,
            rgb[2] * 0.58 + 24.0,
        ];
        add_cube_all(&mut pack, &format!("{color}_terracotta"), solid_speckle(terra, 8.0, salt + 2));
    }
    add_cube_all(&mut pack, "terracotta", solid_speckle([152.0, 94.0, 68.0], 10.0, 330));

    pack
}
