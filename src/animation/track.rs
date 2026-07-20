//! Property tracks and clips.
//!
//! A [`Track`] animates one scalar property through keyframes. A [`Clip`]
//! bundles tracks with timing (duration, delay, repeat, ping-pong) and samples
//! them into a [`Pose`].
//!
//! Tracks are per-property rather than keyframing whole poses because different
//! properties usually want different easing — position sliding out while
//! rotation eases in is the common case, not the exception.

use serde::{Deserialize, Serialize};

use super::easing::Easing;
use super::pose::Pose;

/// A single animatable channel of a [`Pose`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Property {
    X,
    Y,
    Z,
    RotX,
    RotY,
    RotZ,
    ScaleX,
    ScaleY,
    ScaleZ,
    /// Writes all three scale axes at once.
    ScaleUniform,
    Opacity,
    TintR,
    TintG,
    TintB,
    TintA,
    EmissiveR,
    EmissiveG,
    EmissiveB,
}

impl Property {
    fn write(self, pose: &mut Pose, v: f32) {
        match self {
            Property::X => pose.translate[0] = v,
            Property::Y => pose.translate[1] = v,
            Property::Z => pose.translate[2] = v,
            Property::RotX => pose.rotate_deg[0] = v,
            Property::RotY => pose.rotate_deg[1] = v,
            Property::RotZ => pose.rotate_deg[2] = v,
            Property::ScaleX => pose.scale[0] = v,
            Property::ScaleY => pose.scale[1] = v,
            Property::ScaleZ => pose.scale[2] = v,
            Property::ScaleUniform => pose.scale = [v; 3],
            Property::Opacity => pose.opacity = v,
            Property::TintR => pose.tint[0] = v,
            Property::TintG => pose.tint[1] = v,
            Property::TintB => pose.tint[2] = v,
            Property::TintA => pose.tint[3] = v,
            Property::EmissiveR => pose.emissive[0] = v,
            Property::EmissiveG => pose.emissive[1] = v,
            Property::EmissiveB => pose.emissive[2] = v,
        }
    }
}

/// A named post-interpolation transform applied to a track's value.
///
/// This exists because closures cannot cross the Diplomat boundary, so the
/// useful shapes are enumerated instead. Rust callers wanting arbitrary maths
/// should pre-bake keyframes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Modifier {
    Abs,
    AbsSin,
    AbsCos,
    /// `0.5 * (|sin v| + |cos v|)` — a bouncing arc when driven over `0..4π`.
    SinCosBounce,
    Fract,
    Round,
    Clamp01,
    Negate,
}

impl Modifier {
    pub fn apply(self, v: f32) -> f32 {
        match self {
            Modifier::Abs => v.abs(),
            Modifier::AbsSin => v.sin().abs(),
            Modifier::AbsCos => v.cos().abs(),
            Modifier::SinCosBounce => 0.5 * (v.sin().abs() + v.cos().abs()),
            Modifier::Fract => v.fract(),
            Modifier::Round => v.round(),
            Modifier::Clamp01 => v.clamp(0.0, 1.0),
            Modifier::Negate => -v,
        }
    }
}

/// One keyframe. `at` is normalised position within the clip, `0..=1`.
///
/// `ease` describes the transition **into** this keyframe, matching the
/// convention of the JS animation libraries this mirrors.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Keyframe {
    pub at: f32,
    pub value: f32,
    pub ease: Easing,
}

impl Keyframe {
    pub fn new(at: f32, value: f32) -> Self {
        Keyframe {
            at,
            value,
            ease: Easing::Linear,
        }
    }

    pub fn eased(at: f32, value: f32, ease: Easing) -> Self {
        Keyframe { at, value, ease }
    }
}

/// One property animated through keyframes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Track {
    pub property: Property,
    pub keys: Vec<Keyframe>,
    pub modifier: Option<Modifier>,
}

impl Track {
    /// Keys are sorted on construction, so callers may supply them in any order.
    pub fn new(property: Property, mut keys: Vec<Keyframe>) -> Self {
        keys.sort_by(|a, b| {
            a.at.partial_cmp(&b.at)
                .unwrap_or(core::cmp::Ordering::Equal)
        });
        Track {
            property,
            keys,
            modifier: None,
        }
    }

    /// Evenly spaced keyframes from a list of values, sharing one easing.
    ///
    /// `Track::from_values(Property::X, &[-4.0, 0.0, 4.0], Easing::Linear)`
    /// places keys at 0.0, 0.5 and 1.0.
    pub fn from_values(property: Property, values: &[f32], ease: Easing) -> Self {
        let n = values.len();
        let keys = values
            .iter()
            .enumerate()
            .map(|(i, &v)| {
                let at = if n <= 1 {
                    0.0
                } else {
                    i as f32 / (n - 1) as f32
                };
                Keyframe::eased(at, v, ease)
            })
            .collect();
        Track::new(property, keys)
    }

    /// A two-key track from `from` to `to`.
    pub fn tween(property: Property, from: f32, to: f32, ease: Easing) -> Self {
        Track::new(
            property,
            vec![Keyframe::new(0.0, from), Keyframe::eased(1.0, to, ease)],
        )
    }

    pub fn with_modifier(mut self, m: Modifier) -> Self {
        self.modifier = Some(m);
        self
    }

    /// Sample at normalised clip `progress`. `None` when the track has no keys.
    pub fn sample(&self, progress: f32) -> Option<f32> {
        let raw = self.sample_raw(progress)?;
        Some(match self.modifier {
            Some(m) => m.apply(raw),
            None => raw,
        })
    }

    fn sample_raw(&self, progress: f32) -> Option<f32> {
        match self.keys.len() {
            0 => None,
            1 => Some(self.keys[0].value),
            _ => {
                let p = progress.clamp(0.0, 1.0);
                if p <= self.keys[0].at {
                    return Some(self.keys[0].value);
                }
                let last = self.keys.last().unwrap();
                if p >= last.at {
                    return Some(last.value);
                }
                for w in self.keys.windows(2) {
                    let (a, b) = (&w[0], &w[1]);
                    if p >= a.at && p <= b.at {
                        let span = b.at - a.at;
                        // Coincident keys act as a hard cut to the later value.
                        if span <= f32::EPSILON {
                            return Some(b.value);
                        }
                        let local = (p - a.at) / span;
                        let eased = b.ease.eval(local);
                        return Some(a.value + (b.value - a.value) * eased);
                    }
                }
                Some(last.value)
            }
        }
    }
}

/// How many times a clip plays.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Repeat {
    #[default]
    Once,
    Times(u32),
    Forever,
}

/// Tracks plus timing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Clip {
    pub duration_ms: f32,
    pub delay_ms: f32,
    pub tracks: Vec<Track>,
    /// Play every other iteration backwards (ping-pong).
    pub alternate: bool,
    pub repeat: Repeat,
}

impl Clip {
    pub fn new(duration_ms: f32) -> Self {
        Clip {
            duration_ms: duration_ms.max(0.0),
            delay_ms: 0.0,
            tracks: Vec::new(),
            alternate: false,
            repeat: Repeat::Once,
        }
    }

    pub fn track(mut self, t: Track) -> Self {
        self.tracks.push(t);
        self
    }

    pub fn delay(mut self, ms: f32) -> Self {
        self.delay_ms = ms;
        self
    }

    pub fn alternate(mut self, yes: bool) -> Self {
        self.alternate = yes;
        self
    }

    pub fn repeat(mut self, r: Repeat) -> Self {
        self.repeat = r;
        self
    }

    /// Animate the tint through a list of colours, expanding to one track per
    /// channel.
    ///
    /// Colours are `#rgb`, `#rrggbb`, or `#rrggbbaa`; an unparseable entry is
    /// treated as opaque white rather than failing, so a typo dulls a colour
    /// instead of killing the animation.
    ///
    /// One name writing a compound value is the ergonomic worth copying from
    /// the JS animation libraries — three separate channel tracks at the call
    /// site is noise.
    pub fn tint(self, colors: &[&str], ease: Easing) -> Self {
        self.color_tracks(
            colors,
            ease,
            [
                Property::TintR,
                Property::TintG,
                Property::TintB,
                Property::TintA,
            ],
        )
    }

    /// As [`Clip::tint`], but drives the emissive channels (alpha is ignored).
    pub fn emissive(self, colors: &[&str], ease: Easing) -> Self {
        self.color_tracks(
            colors,
            ease,
            [
                Property::EmissiveR,
                Property::EmissiveG,
                Property::EmissiveB,
                // Emissive alpha is unused; parking it on TintA would corrupt
                // opacity, so the 4th channel is dropped below.
                Property::EmissiveB,
            ],
        )
    }

    fn color_tracks(mut self, colors: &[&str], ease: Easing, props: [Property; 4]) -> Self {
        let parsed: Vec<[f32; 4]> = colors.iter().map(|c| parse_color(c)).collect();
        if parsed.is_empty() {
            return self;
        }
        let emissive = matches!(props[0], Property::EmissiveR);
        let channels = if emissive { 3 } else { 4 };
        for (ch, prop) in props.iter().enumerate().take(channels) {
            let values: Vec<f32> = parsed.iter().map(|c| c[ch]).collect();
            self.tracks.push(Track::from_values(*prop, &values, ease));
        }
        self
    }
}

/// Parse `#rgb`, `#rrggbb` or `#rrggbbaa` into linear-ish 0..1 RGBA.
///
/// Values are passed through as sRGB-encoded 0..1 without a gamma conversion,
/// matching how the renderer treats vertex colours.
pub fn parse_color(s: &str) -> [f32; 4] {
    let h = s.trim().trim_start_matches('#');
    let n = |i: usize, len: usize| -> Option<f32> {
        let slice = if len == 1 {
            let c = h.get(i..i + 1)?;
            u8::from_str_radix(&format!("{c}{c}"), 16).ok()?
        } else {
            u8::from_str_radix(h.get(i * 2..i * 2 + 2)?, 16).ok()?
        };
        Some(slice as f32 / 255.0)
    };
    let parsed = match h.len() {
        3 => (0..3).map(|i| n(i, 1)).collect::<Option<Vec<_>>>(),
        6 => (0..3).map(|i| n(i, 2)).collect::<Option<Vec<_>>>(),
        8 => (0..4).map(|i| n(i, 2)).collect::<Option<Vec<_>>>(),
        _ => None,
    };
    match parsed {
        Some(v) if v.len() == 3 => [v[0], v[1], v[2], 1.0],
        Some(v) if v.len() == 4 => [v[0], v[1], v[2], v[3]],
        // Unparseable: neutral white, so a typo dulls rather than breaks.
        _ => [1.0, 1.0, 1.0, 1.0],
    }
}

impl Clip {
    /// Wall-clock length including delay and repeats. `None` when it never ends.
    pub fn total_ms(&self) -> Option<f32> {
        match self.repeat {
            Repeat::Once => Some(self.delay_ms + self.duration_ms),
            Repeat::Times(n) => Some(self.delay_ms + self.duration_ms * n.max(1) as f32),
            Repeat::Forever => None,
        }
    }

    /// Normalised progress at `t_ms`, accounting for delay, repeat and alternate.
    ///
    /// Before the delay elapses the clip holds its first frame; after it
    /// finishes it holds its last.
    pub fn progress_at(&self, t_ms: f32) -> f32 {
        if self.duration_ms <= 0.0 {
            return 1.0;
        }
        let local = t_ms - self.delay_ms;
        if local <= 0.0 {
            return 0.0;
        }
        let iteration = (local / self.duration_ms).floor();
        let mut p = (local / self.duration_ms) - iteration;

        let iters_allowed = match self.repeat {
            Repeat::Once => Some(1.0),
            Repeat::Times(n) => Some(n.max(1) as f32),
            Repeat::Forever => None,
        };
        if let Some(max) = iters_allowed {
            if iteration >= max {
                // Finished: hold the final state, respecting ping-pong parity.
                let last_iter = max - 1.0;
                return if self.alternate && (last_iter as i64) % 2 == 1 {
                    0.0
                } else {
                    1.0
                };
            }
        }
        if self.alternate && (iteration as i64) % 2 == 1 {
            p = 1.0 - p;
        }
        p
    }

    /// Sample every track into `pose`. Tracks with no keys leave their channel
    /// untouched, so a clip only overrides what it actually animates.
    pub fn sample_into(&self, t_ms: f32, pose: &mut Pose) {
        let p = self.progress_at(t_ms);
        for track in &self.tracks {
            if let Some(v) = track.sample(p) {
                track.property.write(pose, v);
            }
        }
    }

    /// Sample into a fresh identity pose.
    pub fn sample(&self, t_ms: f32) -> Pose {
        let mut pose = Pose::IDENTITY;
        self.sample_into(t_ms, &mut pose);
        pose
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::animation::easing::Power;

    #[test]
    fn tween_interpolates_linearly() {
        let t = Track::tween(Property::X, 0.0, 10.0, Easing::Linear);
        assert_eq!(t.sample(0.0), Some(0.0));
        assert_eq!(t.sample(0.5), Some(5.0));
        assert_eq!(t.sample(1.0), Some(10.0));
    }

    #[test]
    fn from_values_spaces_keys_evenly() {
        let t = Track::from_values(Property::X, &[-4.0, 0.0, 4.0], Easing::Linear);
        assert_eq!(t.keys.len(), 3);
        assert_eq!(t.keys[1].at, 0.5);
        assert_eq!(t.sample(0.25), Some(-2.0));
        assert_eq!(t.sample(0.75), Some(2.0));
    }

    #[test]
    fn single_key_is_constant_and_empty_is_none() {
        let one = Track::new(Property::Y, vec![Keyframe::new(0.3, 7.0)]);
        assert_eq!(one.sample(0.0), Some(7.0));
        assert_eq!(one.sample(1.0), Some(7.0));
        let none = Track::new(Property::Y, vec![]);
        assert_eq!(none.sample(0.5), None);
    }

    #[test]
    fn keys_are_sorted_on_construction() {
        let t = Track::new(
            Property::X,
            vec![
                Keyframe::new(1.0, 10.0),
                Keyframe::new(0.0, 0.0),
                Keyframe::new(0.5, 5.0),
            ],
        );
        let ats: Vec<f32> = t.keys.iter().map(|k| k.at).collect();
        assert_eq!(ats, vec![0.0, 0.5, 1.0]);
        assert_eq!(t.sample(0.5), Some(5.0));
    }

    #[test]
    fn out_of_range_progress_clamps_to_end_values() {
        let t = Track::tween(Property::X, 2.0, 8.0, Easing::Linear);
        assert_eq!(t.sample(-1.0), Some(2.0));
        assert_eq!(t.sample(2.0), Some(8.0));
    }

    #[test]
    fn easing_belongs_to_the_incoming_segment() {
        let t = Track::new(
            Property::X,
            vec![
                Keyframe::new(0.0, 0.0),
                Keyframe::eased(1.0, 1.0, Easing::In(Power::Quad)),
            ],
        );
        // Quadratic ease-in: halfway through time is a quarter of the distance.
        assert!((t.sample(0.5).unwrap() - 0.25).abs() < 1e-5);
    }

    #[test]
    fn modifier_applies_after_interpolation() {
        let t = Track::tween(Property::Y, -4.0, 4.0, Easing::Linear).with_modifier(Modifier::Abs);
        assert_eq!(t.sample(0.0), Some(4.0));
        assert_eq!(t.sample(0.5), Some(0.0));
        assert_eq!(t.sample(1.0), Some(4.0));
    }

    /// The reference animation's bounce: y driven 0..4π through |sin|+|cos|.
    #[test]
    fn sincos_bounce_stays_in_a_sane_band() {
        let t = Track::tween(
            Property::Y,
            0.0,
            4.0 * core::f32::consts::PI,
            Easing::Linear,
        )
        .with_modifier(Modifier::SinCosBounce);
        for i in 0..=40 {
            let v = t.sample(i as f32 / 40.0).unwrap();
            assert!(
                (0.49..=0.71).contains(&v),
                "bounce left its band at {i}: {v}"
            );
        }
    }

    #[test]
    fn coincident_keys_cut_rather_than_divide_by_zero() {
        let t = Track::new(
            Property::X,
            vec![
                Keyframe::new(0.0, 0.0),
                Keyframe::new(0.5, 1.0),
                Keyframe::new(0.5, 9.0),
                Keyframe::new(1.0, 9.0),
            ],
        );
        let v = t.sample(0.5).unwrap();
        assert!(v.is_finite(), "coincident keys produced {v}");
    }

    #[test]
    fn clip_holds_first_frame_during_delay() {
        let c = Clip::new(100.0).delay(50.0).track(Track::tween(
            Property::X,
            0.0,
            10.0,
            Easing::Linear,
        ));
        assert_eq!(c.sample(0.0).translate[0], 0.0);
        assert_eq!(c.sample(49.0).translate[0], 0.0);
        assert!((c.sample(100.0).translate[0] - 5.0).abs() < 1e-4);
        assert!((c.sample(150.0).translate[0] - 10.0).abs() < 1e-4);
    }

    #[test]
    fn clip_holds_last_frame_after_finishing() {
        let c = Clip::new(100.0).track(Track::tween(Property::X, 0.0, 10.0, Easing::Linear));
        assert!((c.sample(500.0).translate[0] - 10.0).abs() < 1e-4);
    }

    #[test]
    fn alternate_ping_pongs() {
        let c = Clip::new(100.0)
            .alternate(true)
            .repeat(Repeat::Forever)
            .track(Track::tween(Property::X, 0.0, 10.0, Easing::Linear));
        assert!((c.progress_at(50.0) - 0.5).abs() < 1e-5);
        // Second iteration runs backwards.
        assert!((c.progress_at(150.0) - 0.5).abs() < 1e-5);
        assert!((c.progress_at(125.0) - 0.75).abs() < 1e-5);
        assert!((c.progress_at(175.0) - 0.25).abs() < 1e-5);
    }

    #[test]
    fn repeat_times_stops_after_n_iterations() {
        let c = Clip::new(100.0).repeat(Repeat::Times(2));
        assert!((c.progress_at(150.0) - 0.5).abs() < 1e-5);
        assert_eq!(c.progress_at(250.0), 1.0);
        assert_eq!(c.total_ms(), Some(200.0));
        assert_eq!(Clip::new(100.0).repeat(Repeat::Forever).total_ms(), None);
    }

    #[test]
    fn zero_duration_clip_reads_as_complete() {
        let c = Clip::new(0.0).track(Track::tween(Property::X, 0.0, 10.0, Easing::Linear));
        assert_eq!(c.progress_at(0.0), 1.0);
        assert_eq!(c.sample(0.0).translate[0], 10.0);
    }

    #[test]
    fn clip_only_overrides_channels_it_animates() {
        let c = Clip::new(100.0).track(Track::tween(Property::X, 0.0, 10.0, Easing::Linear));
        let mut pose = Pose {
            translate: [0.0, 5.0, 0.0],
            rotate_deg: [0.0, 45.0, 0.0],
            ..Pose::IDENTITY
        };
        c.sample_into(100.0, &mut pose);
        assert_eq!(pose.translate[0], 10.0);
        assert_eq!(pose.translate[1], 5.0, "untouched channel preserved");
        assert_eq!(pose.rotate_deg[1], 45.0, "untouched channel preserved");
    }

    #[test]
    fn scale_uniform_writes_all_axes() {
        let c = Clip::new(10.0).track(Track::tween(
            Property::ScaleUniform,
            0.0,
            2.0,
            Easing::Linear,
        ));
        assert_eq!(c.sample(10.0).scale, [2.0, 2.0, 2.0]);
    }

    #[test]
    fn parse_color_handles_every_supported_form() {
        assert_eq!(parse_color("#ffffff"), [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(parse_color("#000000"), [0.0, 0.0, 0.0, 1.0]);
        assert_eq!(parse_color("#fff"), [1.0, 1.0, 1.0, 1.0]);
        // Shorthand expands each nibble: #f00 == #ff0000.
        assert_eq!(parse_color("#f00"), parse_color("#ff0000"));
        // Explicit alpha.
        let a = parse_color("#ff000080");
        assert_eq!(a[0], 1.0);
        assert!((a[3] - 0.502).abs() < 1e-2);
        // A leading '#' is optional.
        assert_eq!(parse_color("ff0000"), parse_color("#ff0000"));
    }

    #[test]
    fn unparseable_color_falls_back_to_white() {
        for bad in ["", "#12", "not-a-color", "#zzzzzz", "#1234567"] {
            assert_eq!(
                parse_color(bad),
                [1.0, 1.0, 1.0, 1.0],
                "{bad} should be neutral, not broken"
            );
        }
    }

    /// One name writing three channels — the anime.js ergonomic.
    #[test]
    fn tint_expands_to_channel_tracks() {
        let c = Clip::new(100.0).tint(&["#ff0000", "#0000ff"], Easing::Linear);
        assert_eq!(c.tracks.len(), 4, "r, g, b, a");
        let start = c.sample(0.0);
        assert_eq!(start.tint[0], 1.0, "starts red");
        assert_eq!(start.tint[2], 0.0);
        let end = c.sample(100.0);
        assert_eq!(end.tint[0], 0.0, "ends blue");
        assert_eq!(end.tint[2], 1.0);
        // Midway is a genuine blend, not a cut.
        let mid = c.sample(50.0);
        assert!((mid.tint[0] - 0.5).abs() < 1e-3);
        assert!((mid.tint[2] - 0.5).abs() < 1e-3);
    }

    #[test]
    fn tint_accepts_more_than_two_stops() {
        let c = Clip::new(100.0).tint(&["#ff0000", "#00ff00", "#0000ff"], Easing::Linear);
        assert_eq!(c.sample(50.0).tint[1], 1.0, "green at the midpoint");
    }

    #[test]
    fn emissive_leaves_opacity_alone() {
        let c = Clip::new(100.0).emissive(&["#000000", "#ffcc00"], Easing::Linear);
        assert_eq!(c.tracks.len(), 3, "rgb only — no alpha channel");
        let end = c.sample(100.0);
        assert_eq!(end.emissive[0], 1.0);
        assert_eq!(end.tint[3], 1.0, "tint alpha untouched");
        assert_eq!(end.opacity, 1.0, "opacity untouched");
    }

    #[test]
    fn empty_color_list_is_a_noop() {
        let c = Clip::new(100.0).tint(&[], Easing::Linear);
        assert!(c.tracks.is_empty());
    }

    #[test]
    fn sampling_is_deterministic() {
        let c = Clip::new(1000.0)
            .alternate(true)
            .repeat(Repeat::Forever)
            .track(Track::from_values(
                Property::RotZ,
                &[360.0, 0.0, -360.0],
                Easing::out_back(),
            ));
        for i in 0..200 {
            let t = i as f32 * 13.7;
            assert_eq!(c.sample(t), c.sample(t));
        }
    }
}
