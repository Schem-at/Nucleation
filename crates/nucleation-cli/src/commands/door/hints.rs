use super::*;

#[derive(Default)]
pub(super) struct Hints {
    pub size: Option<[i32; 2]>,
    pub plane: Option<[usize; 2]>,
    pub corner: Option<Point>,
    pub center: Option<Point>,
    pub near: Option<Point>,
    pub candidate: Option<usize>,
}

pub(super) fn size(s: &str) -> Result<[i32; 2], String> {
    let (a, b) = s
        .split_once('x')
        .ok_or("size needs WIDTHxHEIGHT, e.g. 3x3")?;
    let w: i32 = a.parse().map_err(|_| "invalid width")?;
    let h: i32 = b.parse().map_err(|_| "invalid height")?;
    if w < 1 || h < 1 || i64::from(w) * i64::from(h) > 65536 {
        return Err("aperture size must be positive and at most 65536 cells".into());
    }
    Ok([w, h])
}
pub(super) fn plane(raw: &str) -> Result<[usize; 2], String> {
    match raw {
        "xy" => Ok([0, 1]),
        "yz" => Ok([2, 1]),
        "xz" => Ok([0, 2]),
        _ => Err("plane must be xy, yz or xz".into()),
    }
}
impl Hints {
    pub fn validate(&self, bounds: bool) -> Result<(), String> {
        let anchored = self.corner.is_some() || self.center.is_some();
        if self.corner.is_some() && self.center.is_some() {
            return Err("choose one of --corner and --center".into());
        }
        if anchored && self.size.is_none() {
            return Err("--corner and --center need --size WIDTHxHEIGHT".into());
        }
        if (bounds
            && (anchored
                || self.size.is_some()
                || self.plane.is_some()
                || self.near.is_some()
                || self.candidate.is_some()))
            || (anchored && (self.near.is_some() || self.candidate.is_some()))
            || (self.candidate.is_some() && self.near.is_some())
        {
            return Err("use one placement method: explicit bounds, anchored rectangle, nearest candidate, or candidate number".into());
        }
        Ok(())
    }
    pub fn rectangle(&self, offset: Point) -> Result<Option<Vec<Pos>>, String> {
        let Some(anchor) = self.corner.or(self.center) else {
            return Ok(None);
        };
        let [u, v] = self.plane.unwrap_or([0, 1]);
        let [w, h] = self.size.ok_or("missing rectangle dimensions")?;
        let mut lo = pt(local(anchor, offset)?);
        if self.center.is_some() {
            for (i, n) in [(u, w), (v, h)] {
                lo[i] = lo[i]
                    .checked_sub((n - 1) / 2)
                    .ok_or("rectangle coordinate overflow")?;
            }
        }
        let mut cells = Vec::new();
        for row in 0..h {
            for col in 0..w {
                let mut at = lo;
                at[u] = lo[u]
                    .checked_add(col)
                    .ok_or("rectangle coordinate overflow")?;
                at[v] = lo[v]
                    .checked_add(row)
                    .ok_or("rectangle coordinate overflow")?;
                cells.push(pos(at));
            }
        }
        cells.sort_unstable();
        Ok(Some(cells))
    }
    pub fn matches(&self, panel: &[Pos]) -> bool {
        let lo = pt(panel[0]);
        let hi = pt(*panel.last().unwrap());
        [[0, 1], [2, 1], [0, 2]].into_iter().any(|axes| {
            let [u, v] = axes;
            let normal = 3 - u - v;
            lo[normal] == hi[normal]
                && self.plane.is_none_or(|p| p == axes)
                && self.size.is_none_or(|[w, h]| {
                    i64::from(hi[u]) - i64::from(lo[u]) + 1 == i64::from(w)
                        && i64::from(hi[v]) - i64::from(lo[v]) + 1 == i64::from(h)
                })
        })
    }
    pub fn pick(
        &self,
        panels: &[Vec<Pos>],
        offset: Point,
    ) -> Result<(Option<usize>, &'static str), String> {
        if let Some(n) = self.candidate {
            if n == 0 || n > panels.len() {
                return Err(format!(
                    "candidate {n} unavailable; {} match these hints",
                    panels.len()
                ));
            }
            return Ok((Some(n - 1), "candidate"));
        }
        if let Some(near) = self.near {
            let target = pt(local(near, offset)?);
            let mut distances: Vec<_> = panels
                .iter()
                .enumerate()
                .map(|(i, panel)| {
                    let lo = pt(panel[0]);
                    let hi = pt(*panel.last().unwrap());
                    let d = (0..3)
                        .map(|axis| {
                            let v = i128::from(target[axis])
                                - i128::from(target[axis].clamp(lo[axis], hi[axis]));
                            v * v
                        })
                        .sum::<i128>();
                    (d, i)
                })
                .collect();
            distances.sort_unstable();
            if let Some(&(d, i)) = distances.first() {
                if distances.get(1).is_none_or(|next| next.0 != d) {
                    return Ok((Some(i), "nearest_panel"));
                }
            }
        }
        Ok(match panels.len() {
            0 => (None, "undetermined"),
            1 => (Some(0), "inferred_rectangular_panel"),
            _ => (None, "ambiguous"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn anchored_rectangles_use_source_axes_and_lower_middle() {
        let a = Hints {
            size: Some([4, 2]),
            plane: Some(plane("yz").unwrap()),
            center: Some([-8, 5, -2]),
            ..Hints::default()
        };
        let r = a.rectangle([-10, 4, -9]).unwrap().unwrap();
        assert_eq!(r.len(), 8);
        assert_eq!(r[0], Pos::new(2, 1, 6));
        assert_eq!(*r.last().unwrap(), Pos::new(2, 2, 9));
        assert!(a.matches(&r));
        assert!(!Hints {
            plane: Some([0, 1]),
            ..Hints::default()
        }
        .matches(&r));
        assert!(size("65536x1").is_ok());
        for raw in ["1x65537", "-1x2", "2x0", "100"] {
            assert!(size(raw).is_err())
        }
        let b = Hints {
            size: Some([2, 1]),
            corner: Some([i32::MAX, 0, 0]),
            ..Hints::default()
        };
        assert!(b.rectangle([0; 3]).is_err());
    }
    #[test]
    fn nearest_uses_distance_to_panel_and_preserves_ties() {
        let panels = vec![
            vec![pos([-2, 0, 0]), pos([-2, 4, 4])],
            vec![pos([2, 0, 0]), pos([2, 4, 4])],
        ];
        let mut h = Hints {
            near: Some([-10, 4, 4]),
            ..Hints::default()
        };
        assert_eq!(h.pick(&panels, [-10, 0, 0]).unwrap(), (None, "ambiguous"));
        h.near = Some([-11, 4, 4]);
        assert_eq!(
            h.pick(&panels, [-10, 0, 0]).unwrap(),
            (Some(0), "nearest_panel")
        );
        h.candidate = Some(2);
        assert!(h.validate(false).is_err());
        h.near = None;
        assert_eq!(h.pick(&panels, [0; 3]).unwrap().0, Some(1));
        h.candidate = Some(0);
        assert!(h.pick(&panels, [0; 3]).is_err());
    }
}
