use super::*;
use font8x8::{UnicodeFonts, BASIC_FONTS};
use image::{Rgb, RgbImage};
use std::path::Path;

const W: u32 = 1032;
const H: u32 = 468;
type Color = [u8; 3];
const BG: Color = [21, 28, 34];
const PANEL: Color = [31, 40, 47];
const INK: Color = [237, 239, 234];
const MUTED: Color = [175, 193, 197];
const BLOCK: Color = [80, 99, 109];
const CLEAR: Color = [87, 226, 201];
const FILLED: Color = [255, 166, 85];
const INPUT: Color = [251, 225, 101];
const CANDIDATE: Color = [180, 145, 246];

fn rect(im: &mut RgbImage, x: i32, y: i32, w: i32, h: i32, c: Color) {
    for yy in y.max(0)..(y + h).min(H as i32) {
        for xx in x.max(0)..(x + w).min(W as i32) {
            im.put_pixel(xx as u32, yy as u32, Rgb(c));
        }
    }
}
fn label(im: &mut RgbImage, x: i32, y: i32, text: &str, scale: i32, c: Color) {
    for (i, ch) in text.chars().enumerate() {
        if let Some(bitmap) = BASIC_FONTS.get(ch) {
            for (r, bits) in bitmap.iter().enumerate() {
                for col in 0..8 {
                    if bits & (1 << col) != 0 {
                        rect(
                            im,
                            x + (i as i32 * 8 + col) * scale,
                            y + r as i32 * scale,
                            scale,
                            scale,
                            c,
                        )
                    }
                }
            }
        }
    }
}
fn outline(im: &mut RgbImage, x: i32, y: i32, w: i32, h: i32, n: i32, c: Color) {
    rect(im, x, y, w, n, c);
    rect(im, x, y + h - n, w, n, c);
    rect(im, x, y, n, h, c);
    rect(im, x + w - n, y, n, h, c);
}

pub(super) struct Diagram<'a> {
    cells: &'a [Pos],
    panels: &'a [Vec<Pos>],
    input: Option<Pos>,
    near: Option<Pos>,
    offset: Point,
    lo: Point,
    hi: Point,
}
impl<'a> Diagram<'a> {
    pub fn new(
        initial: &Layout,
        cells: &'a [Pos],
        panels: &'a [Vec<Pos>],
        input: Option<Pos>,
        near: Option<Pos>,
        offset: Point,
    ) -> Self {
        let mut lo = [i32::MAX; 3];
        let mut hi = [i32::MIN; 3];
        for p in initial
            .keys()
            .chain(cells)
            .chain(panels.iter().flatten())
            .copied()
            .chain(input)
            .chain(near)
        {
            let q = pt(p);
            for i in 0..3 {
                lo[i] = lo[i].min(q[i]);
                hi[i] = hi[i].max(q[i]);
            }
        }
        if lo[0] > hi[0] {
            lo = [0; 3];
            hi = [0; 3]
        }
        Self {
            cells,
            panels,
            input,
            near,
            offset,
            lo,
            hi,
        }
    }
    fn frame(&self, sim: &Simulation, status: &str) -> RgbImage {
        let mut im = RgbImage::from_pixel(W, H, Rgb(BG));
        label(&mut im, 20, 17, "Door aperture", 3, INK);
        label(&mut im, 20, 52, status, 2, MUTED);
        for (index, (u, v, name)) in [
            (0, 1, "FRONT   X / Y"),
            (2, 1, "SIDE    Z / Y"),
            (0, 2, "PLAN    X / Z"),
        ]
        .into_iter()
        .enumerate()
        {
            let left = 12 + index as i32 * 340;
            rect(&mut im, left, 86, 328, 294, PANEL);
            label(&mut im, left + 12, 99, name, 2, INK);
            let extent = |a: usize| (i64::from(self.hi[a]) - i64::from(self.lo[a]) + 1) as f64;
            let scale = (298.0 / extent(u)).min(208.0 / extent(v)).min(32.0);
            let x0 = left as f64 + (328.0 - extent(u) * scale) * 0.5;
            let y0 = 132.0 + (208.0 - extent(v) * scale) * 0.5;
            let project = |p: Pos| {
                let p = pt(p);
                (
                    (x0 + (i64::from(p[u]) - i64::from(self.lo[u])) as f64 * scale).floor() as i32,
                    (y0 + (i64::from(self.hi[v]) - i64::from(p[v])) as f64 * scale).floor() as i32,
                )
            };
            let side = (scale.floor() as i32).max(1);
            let block = (side - i32::from(side > 4)).max(1);
            for (p, _) in sim.world().iter_non_air() {
                let (x, y) = project(p);
                if (left..left + 328).contains(&x) && (120..344).contains(&y) {
                    rect(&mut im, x, y, block, block, BLOCK);
                }
            }
            if self.cells.is_empty() {
                for (i, cells) in self.panels.iter().enumerate() {
                    for &p in cells {
                        let (x, y) = project(p);
                        outline(&mut im, x, y, side, side, side.min(2), CANDIDATE);
                    }
                    let (x, y) = project(cells[0]);
                    label(&mut im, x, y, &(i + 1).to_string(), 1, INK);
                }
            }
            // Occupied cells take precedence when multiple depths project onto one pixel.
            for filled in [false, true] {
                for &p in self.cells {
                    if (occupied(sim, std::slice::from_ref(&p)) != 0) == filled {
                        let (x, y) = project(p);
                        if filled {
                            rect(&mut im, x, y, block, block, FILLED);
                        } else {
                            outline(&mut im, x, y, block, block, block.min(2), CLEAR);
                        }
                    }
                }
            }
            if let Some(hint) = self.near {
                let (x, y) = project(hint);
                let cx = x + side / 2;
                let cy = y + side / 2;
                outline(&mut im, cx - 4, cy - 4, 9, 9, 1, [255, 255, 255]);
            }
            if let Some(input) = self.input {
                let (x, y) = project(input);
                let center = side / 2;
                rect(&mut im, x + center - 1, y + center - 4, 3, 9, INPUT);
                rect(&mut im, x + center - 4, y + center - 1, 9, 3, INPUT);
            }
            let axis = |i: usize| {
                format!(
                    "{} {}..{}",
                    ["X", "Y", "Z"][i],
                    i64::from(self.lo[i]) + i64::from(self.offset[i]),
                    i64::from(self.hi[i]) + i64::from(self.offset[i])
                )
            };
            label(
                &mut im,
                left + 12,
                354,
                &format!("{}   {}", axis(u), axis(v)),
                1,
                MUTED,
            );
        }
        for (x, c, s) in [
            (20, BLOCK, "Schematic"),
            (212, CLEAR, "Open cell"),
            (398, FILLED, "Filled cell"),
            (620, INPUT, "Input"),
            (780, CANDIDATE, "Candidate"),
        ] {
            rect(&mut im, x, 400, 10, 10, c);
            label(&mut im, x + 18, 398, s, 2, INK);
        }
        let detail = match self.cells.first().zip(self.cells.last()) {
            Some((&a, &b)) => format!(
                "Passage {:?} to {:?}    {}/{} occupied",
                source_pos(a, self.offset),
                source_pos(b, self.offset),
                occupied(sim, self.cells),
                self.cells.len()
            ),
            None => format!(
                "No passage selected    {} matching candidates",
                self.panels.len()
            ),
        };
        label(&mut im, 20, 438, &detail, 1, MUTED);
        if let Some(near) = self.near {
            outline(&mut im, 744, 437, 9, 9, 1, INK);
            label(
                &mut im,
                760,
                438,
                &format!("Near {:?}", source_pos(near, self.offset)),
                1,
                MUTED,
            );
        }
        im
    }
    pub fn save(&self, sim: &Simulation, path: &Path, status: &str) -> Result<(), String> {
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let temporary = parent.join(format!(".nucleation-preview-{}.tmp", std::process::id()));
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(|e| format!("{}: {e}", temporary.display()))?;
        let result = (|| -> Result<(), String> {
            let mut out = BufWriter::new(file);
            self.frame(sim, status)
                .write_to(&mut out, image::ImageFormat::Png)
                .map_err(|e| e.to_string())?;
            out.flush().map_err(|e| e.to_string())?;
            drop(out);
            std::fs::rename(&temporary, path).map_err(|e| e.to_string())
        })();
        if let Err(error) = result {
            let _ = std::fs::remove_file(&temporary);
            return Err(format!("{}: {error}", path.display()));
        }
        Ok(())
    }
}

#[cfg(feature = "render")]
pub(super) struct Recording(nucleation::rendering::video::VideoEncoder);
#[cfg(not(feature = "render"))]
pub(super) struct Recording;
impl Recording {
    pub fn start(path: &Path) -> Result<Self, String> {
        #[cfg(feature = "render")]
        {
            use nucleation::rendering::video::{VideoConfig, VideoEncoder};
            VideoEncoder::start(&VideoConfig::h264(20.)?, W, H, path)
                .map(Self)
                .map_err(|e| e.to_string())
        }
        #[cfg(not(feature = "render"))]
        {
            let _ = path;
            Err("--record requires the render feature".into())
        }
    }
    pub fn frame(
        &mut self,
        diagram: &Diagram,
        sim: &Simulation,
        phase: u64,
        elapsed: u64,
    ) -> Result<(), String> {
        #[cfg(feature = "render")]
        {
            let im = diagram.frame(
                sim,
                &format!(
                    "Use {}   +{} game ticks   |   tick {}",
                    phase,
                    elapsed,
                    sim.tick_count()
                ),
            );
            self.0
                .write_frame(image::DynamicImage::ImageRgb8(im).into_rgba8().as_raw())
                .map_err(|e| e.to_string())
        }
        #[cfg(not(feature = "render"))]
        {
            let _ = (diagram, sim, phase, elapsed);
            Err("--record requires the render feature".into())
        }
    }
    pub fn finish(self) -> Result<(), String> {
        #[cfg(feature = "render")]
        {
            self.0.finish().map_err(|e| e.to_string())
        }
        #[cfg(not(feature = "render"))]
        {
            Err("--record requires the render feature".into())
        }
    }
}
