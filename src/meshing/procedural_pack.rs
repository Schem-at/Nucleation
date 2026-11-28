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

fn add_flat_plate(
    pack: &mut ResourcePackSource,
    model_name: &str,
    height: u32,
    inset: u32,
    pixels: Vec<u8>,
) {
    let tex = format!("minecraft:block/{model_name}");
    let hi = 16 - inset;
    let model = format!(
        r##"{{"textures":{{"all":"{tex}"}},"elements":[{{"from":[{inset},0,{inset}],"to":[{hi},{height},{hi}],"faces":{{"down":{{"texture":"#all"}},"up":{{"texture":"#all"}},"north":{{"texture":"#all"}},"south":{{"texture":"#all"}},"west":{{"texture":"#all"}},"east":{{"texture":"#all"}}}}}}]}}"##
    );
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
    add_cube_all(&mut pack, "smooth_stone", solid_speckle([162.0, 164.0, 166.0], 8.0, 100));
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
    add_cube_all(&mut pack, "gold_block", solid_speckle([246.0, 208.0, 68.0], 16.0, 132));
    add_cube_all(&mut pack, "iron_block", solid_speckle([220.0, 222.0, 224.0], 10.0, 135));
    add_cube_all(&mut pack, "diamond_block", solid_speckle([98.0, 236.0, 228.0], 14.0, 136));
    add_cube_all(&mut pack, "emerald_block", solid_speckle([52.0, 206.0, 102.0], 16.0, 137));
    add_cube_all(&mut pack, "redstone_block", solid_speckle([196.0, 26.0, 18.0], 18.0, 138));
    add_cube_all(&mut pack, "slime_block", solid_speckle([114.0, 196.0, 92.0], 14.0, 139));
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

    // 5. Redstone components with powered/lit state variants (for tick-sim / redstone viewers!)
    let lamp_off = make_tex(|x, y| {
        let grid = x == 0 || y == 0 || x == 15 || y == 15 || (x + y) % 8 == 0 || (x + 16 - y) % 8 == 0;
        if grid {
            [62.0, 38.0, 24.0, 255.0]
        } else {
            [118.0, 74.0, 46.0, 255.0]
        }
    });
    let lamp_on = make_tex(|x, y| {
        let grid = x == 0 || y == 0 || x == 15 || y == 15 || (x + y) % 8 == 0 || (x + 16 - y) % 8 == 0;
        if grid {
            [196.0, 128.0, 54.0, 255.0]
        } else {
            [255.0, 224.0, 126.0, 255.0]
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
        r#"{"variants":{"lit=false":{"model":"minecraft:block/redstone_lamp"},"lit=true":{"model":"minecraft:block/redstone_lamp_on"}}}"#,
    );

    // Redstone wire (flat cross strip, dark crimson when power=0, bright glowing red when powered)
    add_flat_plate(
        &mut pack,
        "redstone_wire_off",
        1,
        2,
        make_tex(|x, y| {
            let line = (6..=9).contains(&x) || (6..=9).contains(&y);
            if line { [88.0, 14.0, 14.0, 255.0] } else { [0.0, 0.0, 0.0, 0.0] }
        }),
    );
    add_flat_plate(
        &mut pack,
        "redstone_wire_on",
        1,
        2,
        make_tex(|x, y| {
            let line = (6..=9).contains(&x) || (6..=9).contains(&y);
            if line { [252.0, 36.0, 24.0, 255.0] } else { [0.0, 0.0, 0.0, 0.0] }
        }),
    );
    {
        let mut variants = Vec::with_capacity(16);
        for p in 0..=15 {
            let m = if p == 0 {
                "minecraft:block/redstone_wire_off"
            } else {
                "minecraft:block/redstone_wire_on"
            };
            variants.push(format!(r#""power={p}":{{"model":"{m}"}}"#));
        }
        let bs = format!(r#"{{"variants":{{{}}}}}"#, variants.join(","));
        let _ = pack.add_blockstate_json("minecraft:redstone_wire", &bs);
    }

    // Repeater (flat slab with directional arrow + lit indicator)
    add_flat_plate(
        &mut pack,
        "repeater_off",
        2,
        0,
        make_tex(|x, y| {
            let torch = (x >= 6 && x <= 9) && (y == 4 || y == 5 || y == 11 || y == 12);
            if torch { [96.0, 20.0, 20.0, 255.0] } else { [156.0, 158.0, 160.0, 255.0] }
        }),
    );
    add_flat_plate(
        &mut pack,
        "repeater_on",
        2,
        0,
        make_tex(|x, y| {
            let torch = (x >= 6 && x <= 9) && (y == 4 || y == 5 || y == 11 || y == 12);
            if torch { [255.0, 54.0, 34.0, 255.0] } else { [164.0, 166.0, 168.0, 255.0] }
        }),
    );
    let _ = pack.add_blockstate_json(
        "minecraft:repeater",
        r#"{"variants":{"powered=false":{"model":"minecraft:block/repeater_off"},"powered=true":{"model":"minecraft:block/repeater_on"}}}"#,
    );

    // Lever (base + handle block, green/red accent for on/off)
    add_flat_plate(&mut pack, "lever_off", 5, 4, solid_speckle([110.0, 112.0, 116.0], 10.0, 151));
    add_flat_plate(&mut pack, "lever_on", 5, 4, solid_speckle([238.0, 68.0, 44.0], 10.0, 152));
    let _ = pack.add_blockstate_json(
        "minecraft:lever",
        r#"{"variants":{"powered=false":{"model":"minecraft:block/lever_off"},"powered=true":{"model":"minecraft:block/lever_on"}}}"#,
    );

    // Redstone torch (compact pillar, lit vs unlit)
    add_flat_plate(&mut pack, "redstone_torch_off", 10, 6, solid_speckle([98.0, 64.0, 44.0], 8.0, 153));
    add_flat_plate(&mut pack, "redstone_torch_on", 10, 6, solid_speckle([255.0, 62.0, 32.0], 12.0, 154));
    let _ = pack.add_blockstate_json(
        "minecraft:redstone_torch",
        r#"{"variants":{"lit=false":{"model":"minecraft:block/redstone_torch_off"},"lit=true":{"model":"minecraft:block/redstone_torch_on"}}}"#,
    );
    let _ = pack.add_blockstate_json(
        "minecraft:redstone_wall_torch",
        r#"{"variants":{"lit=false":{"model":"minecraft:block/redstone_torch_off"},"lit=true":{"model":"minecraft:block/redstone_torch_on"}}}"#,
    );

    // Stone button
    add_flat_plate(&mut pack, "stone_button_off", 3, 5, solid_speckle([144.0, 146.0, 150.0], 8.0, 155));
    add_flat_plate(&mut pack, "stone_button_on", 2, 5, solid_speckle([232.0, 96.0, 48.0], 8.0, 156));
    let _ = pack.add_blockstate_json(
        "minecraft:stone_button",
        r#"{"variants":{"powered=false":{"model":"minecraft:block/stone_button_off"},"powered=true":{"model":"minecraft:block/stone_button_on"}}}"#,
    );

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
