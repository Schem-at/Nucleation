//! Aperture clocks and bounded, exhaustive search of two-input schedules.
use super::{control, state, step, Pos, Simulation};
use mc_test::mc_tick::{Checkpoint, StateId};
use serde_json::{json, Value};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Endpoint {
    Open,
    Closed,
}
impl Endpoint {
    pub fn opposite(self) -> Self {
        match self {
            Self::Open => Self::Closed,
            Self::Closed => Self::Open,
        }
    }
    fn action(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Closed => "close",
        }
    }
}

// A moving-piston placeholder occupies a cell before its payload lands.
#[derive(Clone, Copy)]
pub(super) struct Reading {
    pub occupied: usize,
    pub(super) moving: bool,
}
impl Reading {
    pub fn sample(sim: &Simulation, cells: &[Pos]) -> Self {
        let mut count = 0;
        let mut moving = false;
        for &p in cells {
            let id = sim.world().get(p);
            if id == StateId::AIR {
                continue;
            }
            let descriptor = state(sim, id);
            match descriptor.split('[').next().unwrap_or(descriptor) {
                "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air" => {}
                "minecraft:moving_piston" => {
                    count += 1;
                    moving = true;
                }
                _ => count += 1,
            }
        }
        Self {
            occupied: count,
            moving,
        }
    }
    pub fn endpoint(self, n: usize) -> Option<Endpoint> {
        if n == 0 || self.moving {
            None
        } else if self.occupied == 0 {
            Some(Endpoint::Open)
        } else if self.occupied == n {
            Some(Endpoint::Closed)
        } else {
            None
        }
    }
}

// A scheduled pause is still busy. An empty queue can also coexist with a
// moving entity or hopper activity. Track the last observed change as well.
#[derive(Default, Clone, Copy)]
struct Settling {
    counts: [usize; 3],
    quiet_since: Option<u64>,
    confirmed: Option<u64>,
}
impl Settling {
    fn observe(&mut self, elapsed: u64, quiet: bool, counts: [usize; 3]) {
        if !quiet {
            self.quiet_since = None;
        } else if self.counts != counts || self.quiet_since.is_none() {
            self.quiet_since = Some(elapsed);
        }
        self.counts = counts;
        self.confirmed = self.quiet_since.filter(|&start| start < elapsed);
    }
}

#[derive(Default, Clone, Copy)]
pub(super) struct Clock {
    delay: Option<u64>,
    complete: Option<u64>,
    first_arrival: Option<u64>,
    settling: Settling,
    flickered: bool,
}
impl Clock {
    pub fn sample(&mut self, elapsed: u64, at: Option<Endpoint>, target: Endpoint) {
        if at != Some(target.opposite()) {
            self.delay.get_or_insert(elapsed);
        }
        if at == Some(target) {
            self.first_arrival.get_or_insert(elapsed);
            self.complete.get_or_insert(elapsed);
        } else {
            self.flickered |= self.first_arrival.is_some();
            self.complete = None;
        }
    }
    pub fn observe_mechanism(&mut self, elapsed: u64, sim: &Simulation) {
        self.settling.observe(
            elapsed,
            sim.is_quiescent(),
            [
                sim.recorded().len(),
                sim.recorded_inventory().len(),
                sim.recorded_entities().len(),
            ],
        );
    }
    pub fn mechanism_settled(self) -> Option<u64> {
        self.settling.confirmed
    }
    fn clean(self) -> bool {
        self.complete.is_some() && !self.flickered
    }
    pub fn report(self, target: Endpoint) -> Value {
        json!({"action":target.action(),"delay_ticks":self.delay,"completion_ticks":self.complete,
        "aperture_first_ticks":self.first_arrival,"aperture_stable_ticks":self.complete,
        "mechanism_settled_ticks":self.mechanism_settled(),
        "settle_tail_ticks":self.mechanism_settled().zip(self.complete).and_then(|(settled,aperture)|settled.checked_sub(aperture)),
        "motion_ticks":self.complete.zip(self.delay).map(|(end,start)|end-start),"lost_target_after_first_arrival":self.flickered})
    }
}

#[derive(Debug, PartialEq)]
enum Trial {
    Passed(Value),
    FailedAt(usize),
}

pub(super) struct Sweep<'a> {
    pub cells: &'a [Pos],
    pub input: Pos,
    pub max_dwell: usize,
    pub cycles: usize,
}
impl Sweep<'_> {
    fn window(
        &self,
        sim: &mut Simulation,
        target: Endpoint,
        dwell: usize,
        mut visit: impl FnMut(usize, Clock),
    ) -> Result<Clock, String> {
        if !control(state(sim, sim.world().get(self.input))) {
            return Ok(Clock::default());
        }
        sim.use_block(self.input);
        let mut clock = Clock::default();
        clock.observe_mechanism(0, sim);
        for elapsed in 1..=dwell {
            step(sim)?;
            clock.observe_mechanism(elapsed as u64, sim);
            clock.sample(
                elapsed as u64,
                Reading::sample(sim, self.cells).endpoint(self.cells.len()),
                target,
            );
            visit(elapsed, clock);
            // Once the target flickers no longer dwell in this window can pass.
            if clock.flickered {
                break;
            }
        }
        Ok(clock)
    }
    fn reset(&self, sim: &mut Simulation, base: &Checkpoint) {
        sim.restore(base);
        sim.discard_recording();
        sim.set_rng_seed(0);
    }
    fn prefixes(&self, sim: &mut Simulation, target: Endpoint) -> Result<Vec<bool>, String> {
        let mut allowed = vec![false; self.max_dwell + 1];
        self.window(sim, target, self.max_dwell, |elapsed, c| {
            allowed[elapsed] = c.clean()
        })?;
        Ok(allowed)
    }
    fn repeats(
        &self,
        sim: &mut Simulation,
        initial: Endpoint,
        first: usize,
        second: usize,
    ) -> Result<Trial, String> {
        let mut timings = Vec::new();
        for cycle in 1..=self.cycles {
            let mut row = json!({"cycle":cycle});
            for (target, dwell) in [(initial.opposite(), first), (initial, second)] {
                sim.record();
                let clock = self.window(sim, target, dwell, |_, _| {})?;
                if !clock.clean() {
                    return Ok(Trial::FailedAt(cycle));
                }
                row[target.action()] = clock.report(target);
            }
            timings.push(row);
        }
        let (open, close) = if initial == Endpoint::Closed {
            (first, second)
        } else {
            (second, first)
        };
        Ok(Trial::Passed(
            json!({"period_ticks":first+second,"cycles_per_second":20.0/(first+second) as f64,
        "open_dwell_ticks":open,"close_dwell_ticks":close,"cycles_passed":self.cycles,"cycles":timings}),
        ))
    }
    pub fn measure(&self, sim: &mut Simulation, base: &Checkpoint) -> Result<Value, String> {
        self.reset(sim, base);
        let Some(initial) = Reading::sample(sim, self.cells).endpoint(self.cells.len()) else {
            return Ok(
                json!({"status":"needs_initial_clear_or_filled_aperture","cycles_requested":self.cycles,"max_dwell_ticks":self.max_dwell}),
            );
        };
        let first_ok = self.prefixes(sim, initial.opposite())?;
        let mut second_ok = vec![None; self.max_dwell + 1];
        let mut fastest = None;
        let mut symmetric = None;
        let mut rejected_later = 0;
        let mut examined = 0;
        let mut example = None;
        // Ascending period proves the first pass is fastest inside the stated
        // bounds. No assumption that a longer period will also work.
        for period in 2..=2 * self.max_dwell {
            for first in
                period.saturating_sub(self.max_dwell).max(1)..=self.max_dwell.min(period - 1)
            {
                let second = period - first;
                if fastest.is_some() && first != second {
                    continue;
                }
                examined += 1;
                if !first_ok[first] {
                    continue;
                }
                if second_ok[first].is_none() {
                    self.reset(sim, base);
                    self.window(sim, initial.opposite(), first, |_, _| {})?;
                    second_ok[first] = Some(self.prefixes(sim, initial)?);
                }
                if !second_ok[first].as_ref().unwrap()[second] {
                    continue;
                }
                self.reset(sim, base);
                match self.repeats(sim, initial, first, second)? {
                    Trial::Passed(result) => {
                        fastest.get_or_insert_with(|| result.clone());
                        if first == second {
                            symmetric = Some(result)
                        }
                    }
                    Trial::FailedAt(cycle) => {
                        rejected_later += 1;
                        example.get_or_insert_with(||json!({"first_dwell_ticks":first,"second_dwell_ticks":second,"failed_cycle":cycle}));
                    }
                }
            }
            if symmetric.is_some() {
                break;
            }
        }
        Ok(
            json!({"status":if fastest.is_some(){"measured"}else{"no_passing_schedule"},"initial_aperture":if initial==Endpoint::Open{"clear"}else{"filled"},
            "cycles_requested":self.cycles,"max_dwell_ticks":self.max_dwell,"schedules_examined":examined,
            "fastest":fastest,"fastest_evenly_spaced":symmetric,"examined_schedules_failing_after_cycle_one":rejected_later,"example_later_failure":example}),
        )
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hipster_shortest_single_cycle_fails_when_repeated() {
        let structure = mc_test::mc_tick::Structure::parse(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../mc-tick/tests/corpus/structures/hipster_door.snbt"
        )))
        .unwrap();
        let mut sim = mc_test::build_sim(
            &structure,
            Pos::new(0, 0, 0),
            mc_test::SettleMode::InWorld,
            &[],
            &[],
            Some(2730),
            "cycle frequency",
        );
        sim.set_rng_seed(0);
        sim.run(10);
        let checkpoint = sim.checkpoint();
        let cells: Vec<_> = (8..=10)
            .flat_map(|y| (7..=9).map(move |z| Pos::new(5, y, z)))
            .collect();
        let sweep = Sweep {
            cells: &cells,
            input: Pos::new(12, 7, 13),
            cycles: 3,
            max_dwell: 44,
        };
        sweep.reset(&mut sim, &checkpoint);
        for (target, first, settled) in [(Endpoint::Closed, 9, 16), (Endpoint::Open, 39, 47)] {
            sim.record();
            let clock = sweep.window(&mut sim, target, 160, |_, _| {}).unwrap();
            assert_eq!(clock.first_arrival, Some(first));
            assert_eq!(clock.complete, Some(first));
            assert_eq!(clock.mechanism_settled(), Some(settled));
        }
        // The lower two rows clear, are obstructed again, then stay clear.
        sweep.reset(&mut sim, &checkpoint);
        sweep
            .window(&mut sim, Endpoint::Closed, 160, |_, _| {})
            .unwrap();
        let lower_rows: Vec<_> = cells.iter().copied().filter(|p| p.y <= 9).collect();
        let mut opening = Clock::default();
        sim.use_block(sweep.input);
        for tick in 1..=160 {
            step(&mut sim).unwrap();
            opening.sample(
                tick,
                Reading::sample(&sim, &lower_rows).endpoint(lower_rows.len()),
                Endpoint::Open,
            );
        }
        assert_eq!(
            (opening.first_arrival, opening.complete),
            (Some(15), Some(39))
        );
        assert!(opening.flickered);
        sweep.reset(&mut sim, &checkpoint);
        assert_eq!(
            sweep.repeats(&mut sim, Endpoint::Open, 9, 39).unwrap(),
            Trial::FailedAt(2)
        );
        let result = sweep.measure(&mut sim, &checkpoint).unwrap();
        assert_eq!(result["fastest"]["period_ticks"], 53);
        assert_eq!(result["fastest"]["close_dwell_ticks"], 9);
        assert_eq!(result["fastest_evenly_spaced"]["period_ticks"], 88);
    }

    #[test]
    fn shuffling_and_flicker_are_different() {
        let filled = Reading {
            occupied: 9,
            moving: false,
        }
        .endpoint(9);
        let transit = Reading {
            occupied: 9,
            moving: true,
        }
        .endpoint(9);
        assert_eq!(transit, None);
        let mut clock = Clock::default();
        for (i, v) in [Some(Endpoint::Open), transit, filled, filled]
            .into_iter()
            .enumerate()
        {
            clock.sample((i + 1) as u64, v, Endpoint::Closed);
        }
        assert!(clock.clean());
        assert_eq!((clock.delay, clock.complete), (Some(2), Some(3)));
        clock.sample(5, transit, Endpoint::Closed);
        clock.sample(6, filled, Endpoint::Closed);
        assert!(!clock.clean());
        assert_eq!(clock.complete, Some(6));
        assert_eq!(clock.first_arrival, Some(3));
        let mut settling = Settling::default();
        settling.observe(6, false, [0, 0, 0]);
        settling.observe(7, true, [0, 0, 0]);
        settling.observe(8, true, [0, 0, 0]);
        assert_eq!(settling.confirmed, Some(7));
        settling.observe(9, true, [0, 1, 0]);
        assert_eq!(settling.confirmed, None);
        settling.observe(10, true, [0, 1, 0]);
        assert_eq!(settling.confirmed, Some(9));
    }
}
