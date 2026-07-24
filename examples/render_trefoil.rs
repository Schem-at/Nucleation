//! A trefoil knot with a pulse of light running around it, forever.
//!
//!     cargo run --release --example render_trefoil --features rendering -- <pack.zip> [out_dir]
//!
//! Two things loop here, and they loop for the same reason: a trefoil is a
//! *closed* curve. The colour is a cyclic hue sweep that meets itself with no
//! seam, and the light pulse travels the curve and wraps. Sample exactly one
//! period and the GIF is seamless — the last frame flows back into the first.
//!
//! Blocks are grouped into curve segments rather than individually: ~8k
//! separate meshes would be one draw call each, and at this scale the segment
//! boundary is invisible.

use nucleation::animation::{
    presets, BuildAnimator, Clip, Easing, Grouping, Keyframe, Order, Power, Property, Repeat,
    Stagger, Target, Track,
};
use nucleation::blockpedia::color::ExtendedColorData;
use nucleation::building::BlockPalette;
use nucleation::meshing::{MeshConfig, ResourcePackSource};
use nucleation::rendering::{render_animation_to_files, RenderConfig};
use nucleation::UniversalSchematic;

/// Curve samples. Higher = smoother tube, more blocks, slower render.
const STEPS: usize = 520;
/// Tube radius in blocks.
const TUBE: f64 = 2.6;
/// Overall knot scale.
const SCALE: f64 = 11.0;
/// Curve segments = animation groups. Fewer is faster; more is a finer pulse.
const SEGMENTS: usize = 90;
/// One full loop: build, hold, dissolve, and one 360° camera turn.
const PERIOD_MS: f32 = 6000.0;
/// How long the wave takes to travel the knot, as a fraction of the period.
/// The tail remains visible across the period boundary, so the travelling wave
/// wraps directly into the head without an empty pause at the GIF seam.
const SPREAD_MS: f32 = PERIOD_MS * 0.40;
/// Phase boundaries within a segment's own clip, as fractions of the period.
const APPEAR_END: f32 = 0.10;
const VANISH_START: f32 = 0.58;
const VANISH_END: f32 = 0.68;
/// How far above its final position a block starts (and ends).
const FLY_HEIGHT: f32 = 7.0;
const FPS: f64 = 20.0;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let pack_path = args.next().unwrap_or_else(|| {
        eprintln!("usage: render_trefoil <pack.zip> [out_dir]");
        std::process::exit(2);
    });
    let out_dir = args
        .next()
        .unwrap_or_else(|| "render_work/trefoil".to_string());
    std::fs::create_dir_all(&out_dir)?;

    // The widest palette a solid form can use: every full-cube block, patterned
    // ones included for their extra hues. Only see-through blocks are dropped,
    // since they would punch holes in the tube. Dithered, so each voxel picks
    // between the two nearest blocks and the ramp reads smooth up close.
    //
    // Dithering costs real bytes in a GIF: alternating neighbouring pixels is
    // exactly what LZW cannot compress, and it can double the file. With a
    // palette this wide the plain snap is already smooth, so `--no-dither` is
    // the better trade for a README.
    let base = BlockPalette::builder()
        .full_blocks_only()
        .exclude_transparent()
        .exclude_tile_entities()
        .build();
    let dither = !std::env::args().any(|a| a == "--no-dither");
    let palette = if dither { base.dithered() } else { base };
    println!("palette: {} blocks (dither: {dither})", palette.len());

    // 1. Build the knot. A cell is claimed by the first segment that fills it,
    //    so overlapping spheres cannot make the pulse jump backwards.
    let mut schem = UniversalSchematic::new("trefoil".to_string());
    let mut segment_of: std::collections::HashMap<(i32, i32, i32), usize> = Default::default();

    for i in 0..STEPS {
        let f = i as f64 / STEPS as f64;
        let a = f * std::f64::consts::TAU;
        let cx = (SCALE * (a.sin() + 2.0 * (2.0 * a).sin())).round() as i32;
        let cy = (SCALE * (a.cos() - 2.0 * (2.0 * a).cos())).round() as i32;
        let cz = (SCALE * -(3.0 * a).sin()).round() as i32;
        let seg = ((f * SEGMENTS as f64) as usize).min(SEGMENTS - 1);

        // Hue swept once around the loop so the ramp closes on itself. Saturation
        // is eased below full on purpose: at max saturation the targets are more
        // vivid than any block, and whole hue arcs collapse onto the single most
        // saturated block. Pulling it back lands targets inside the block gamut,
        // giving the dither real neighbours to blend between.
        let (r, g, b) = hsv_to_rgb((1.0 - f) % 1.0, 0.82, 1.0);
        let target = ExtendedColorData::from_rgb(r, g, b);

        let rad = TUBE.ceil() as i32;
        for dx in -rad..=rad {
            for dy in -rad..=rad {
                for dz in -rad..=rad {
                    if (dx * dx + dy * dy + dz * dz) as f64 > TUBE * TUBE {
                        continue;
                    }
                    let p = (cx + dx, cy + dy, cz + dz);
                    if segment_of.contains_key(&p) {
                        continue;
                    }
                    // Dithering is position-aware, so neighbouring voxels
                    // alternate between the two nearest blocks.
                    if let Some(id) = palette.snap(&target, p.0, p.1, p.2) {
                        segment_of.insert(p, seg);
                        schem.set_block_from_string(p.0, p.1, p.2, &id).ok();
                    }
                }
            }
        }
    }
    println!("blocks: {}", segment_of.len());

    // 2. One group per curve segment; group index runs along the curve.
    let mut buckets: Vec<Vec<(i32, i32, i32)>> = vec![Vec::new(); SEGMENTS];
    let mut sorted: Vec<_> = segment_of.into_iter().collect();
    sorted.sort_unstable(); // deterministic group contents
    for (pos, seg) in sorted {
        buckets[seg].push(pos);
    }
    let positions: Vec<_> = buckets.iter().flatten().copied().collect();
    let mut anim = BuildAnimator::from_positions(&positions, Grouping::Custom(buckets));
    println!("segments: {}", anim.groups().len());

    // 3. Every segment runs the same endlessly repeating clip, offset by where
    //    it sits on the curve — which is what turns a per-segment move into a
    //    wave travelling around the knot.
    //
    //    One period is: fly in (staggered along the curve) → hold the finished
    //    knot → fly back out in the same direction. The tail crosses the period
    //    boundary and meets the head on the closed curve, avoiding an empty hold.
    //
    //    The last segment still lands before the first leaves, so the knot is
    //    genuinely whole for a beat even though the travelling wave wraps.
    let flash = arrival_flash();
    let flash_refs: Vec<&str> = flash.iter().map(String::as_str).collect();
    let clip = Clip::new(PERIOD_MS)
        .repeat(Repeat::Forever)
        // Scale: pop in with overshoot, hold, shrink away.
        .track(Track::new(
            Property::ScaleUniform,
            vec![
                Keyframe::new(0.0, 0.0),
                Keyframe::eased(APPEAR_END, 1.0, Easing::out_back()),
                Keyframe::new(VANISH_START, 1.0),
                Keyframe::eased(VANISH_END, 0.0, Easing::In(Power::Cubic)),
                Keyframe::new(1.0, 0.0),
            ],
        ))
        // Height: drop in from above, and rise away on the way out, so blocks
        // visibly travel rather than just fading.
        .track(Track::new(
            Property::Y,
            vec![
                Keyframe::new(0.0, FLY_HEIGHT),
                Keyframe::eased(APPEAR_END, 0.0, Easing::Out(Power::Cubic)),
                Keyframe::new(VANISH_START, 0.0),
                Keyframe::eased(VANISH_END, FLY_HEIGHT, Easing::In(Power::Cubic)),
                Keyframe::new(1.0, FLY_HEIGHT),
            ],
        ))
        // A warm flash on arrival. Emissive *adds* to the block colour, so a
        // bright wide pulse clips to white and erases the gradient underneath —
        // keep it brief and modest.
        .emissive(&flash_refs, Easing::Linear);

    // Starting one full period early means every segment is already mid-cycle
    // at t=0, so frame 0 and frame N of a single period match exactly.
    anim.timeline_mut()
        .add_staggered(clip, &Stagger::total(Order::Index, SPREAD_MS), -PERIOD_MS);

    // 4. Exactly one turn per period, so the camera loops with the pulse.
    let spin = presets::turntable(PERIOD_MS);
    anim.timeline_mut().add(spin, Target::Camera, 0.0);

    // 5. Sample one period, excluding the endpoint — frame N would duplicate
    //    frame 0 and stutter the loop.
    let count = (PERIOD_MS as f64 / 1000.0 * FPS).round() as usize;
    let frames: Vec<_> = (0..count)
        .map(|i| anim.frame_at((i as f64 * 1000.0 / FPS) as f32))
        .collect();
    println!(
        "{PERIOD_MS:.0}ms -> {} frames @{FPS}fps (seamless)",
        frames.len()
    );

    let pack = ResourcePackSource::from_file(&pack_path)?;
    let meshes = schem.mesh_groups(&pack, &MeshConfig::default(), anim.groups())?;

    let mut rc = RenderConfig::isometric();
    rc.width = 480;
    rc.height = 480;
    rc.sphere_fit = true;
    rc.background = Some([0.0, 0.0, 0.0, 0.0]);

    let paths = render_animation_to_files(&meshes, &frames, &rc, None, &format!("{out_dir}/f"))?;
    println!("wrote {} frames to {out_dir}/", paths.len());
    Ok(())
}

/// Emissive keyframes: a warm flash centred on the moment a block lands, dark
/// for the rest of the period.
///
/// Evenly spaced keys, so the flash window is expressed as a fraction of the
/// period and lines up with `APPEAR_END`.
fn arrival_flash() -> Vec<String> {
    const COUNT: usize = 40;
    let centre = APPEAR_END as f64;
    let half = 0.06;
    let peak = 0.5;
    (0..COUNT)
        .map(|i| {
            let t = i as f64 / (COUNT - 1) as f64;
            let d = (t - centre).abs() / half;
            let v = if d >= 1.0 {
                0.0
            } else {
                let s = 1.0 - d;
                s * s * peak
            };
            // Warm amber: more red than green, almost no blue.
            let r = (255.0 * v).round() as u8;
            let g = (170.0 * v).round() as u8;
            let b = (80.0 * v).round() as u8;
            format!("#{r:02x}{g:02x}{b:02x}")
        })
        .collect()
}

/// HSV -> RGB, `h`/`s`/`v` in 0..1.
fn hsv_to_rgb(h: f64, s: f64, v: f64) -> (u8, u8, u8) {
    let i = (h * 6.0).floor();
    let f = h * 6.0 - i;
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);
    let (r, g, b) = match (i as i32) % 6 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    };
    (
        (r * 255.0).round() as u8,
        (g * 255.0).round() as u8,
        (b * 255.0).round() as u8,
    )
}
