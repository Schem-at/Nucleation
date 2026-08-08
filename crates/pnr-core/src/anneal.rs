//! Seeded simulated-annealing engine over Move/Cost/Feasibility traits.
//!
//! The RNG is a SplitMix64 implemented here so results are reproducible from
//! a seed with no `rand` dependency (wasm-safe, deterministic per platform;
//! the only float op is `exp`, whose ULP differences across platforms can in
//! principle flip an acceptance — seeds are reproducible on a given target).

/// Deterministic seeded RNG (SplitMix64).
#[derive(Clone, Debug)]
pub struct SplitMix64(u64);

impl SplitMix64 {
    /// Seed the generator.
    pub fn new(seed: u64) -> Self {
        SplitMix64(seed)
    }

    /// Next raw 64-bit value.
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }

    /// Uniform float in `[0, 1)`.
    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Uniform integer in `[0, n)`. `n` must be nonzero.
    pub fn gen_range(&mut self, n: usize) -> usize {
        (self.next_u64() % n as u64) as usize
    }
}

/// A problem the annealer can optimize: propose moves, apply them, score
/// states, and gate feasibility.
pub trait AnnealProblem {
    /// Solution state.
    type State: Clone;
    /// A proposed perturbation.
    type Move;

    /// Propose a move from `state` (None = no move available).
    fn propose(&self, state: &Self::State, rng: &mut SplitMix64) -> Option<Self::Move>;
    /// Apply a move, producing the successor state.
    fn apply(&self, state: &Self::State, mv: &Self::Move) -> Self::State;
    /// Cost to minimize.
    fn cost(&self, state: &Self::State) -> f64;
    /// Hard feasibility gate: infeasible successors are always rejected.
    fn feasible(&self, state: &Self::State) -> bool {
        let _ = state;
        true
    }
}

/// Geometric cooling schedule.
#[derive(Clone, Debug)]
pub struct Schedule {
    /// Initial temperature.
    pub t0: f64,
    /// Multiplicative cooling factor per temperature step (e.g. 0.95).
    pub cooling: f64,
    /// Proposals evaluated at each temperature.
    pub steps_per_temp: usize,
    /// Stop when the temperature falls below this.
    pub t_min: f64,
}

impl Default for Schedule {
    fn default() -> Self {
        Schedule {
            t0: 10.0,
            cooling: 0.9,
            steps_per_temp: 200,
            t_min: 1e-3,
        }
    }
}

/// Outcome of an annealing run.
#[derive(Clone, Debug)]
pub struct AnnealResult<S> {
    /// Best feasible state observed.
    pub best: S,
    /// Its cost.
    pub best_cost: f64,
    /// Number of accepted moves.
    pub accepted: usize,
    /// Number of proposals evaluated.
    pub proposed: usize,
}

/// Run simulated annealing from `init` (which must be feasible) under the
/// schedule, seeded for reproducibility.
pub fn anneal<P: AnnealProblem>(
    problem: &P,
    init: P::State,
    schedule: &Schedule,
    seed: u64,
) -> AnnealResult<P::State> {
    let mut rng = SplitMix64::new(seed);
    let mut cur = init.clone();
    let mut cur_cost = problem.cost(&cur);
    let mut best = init;
    let mut best_cost = cur_cost;
    let mut accepted = 0usize;
    let mut proposed = 0usize;

    let mut t = schedule.t0;
    while t > schedule.t_min {
        for _ in 0..schedule.steps_per_temp {
            let Some(mv) = problem.propose(&cur, &mut rng) else {
                continue;
            };
            proposed += 1;
            let next = problem.apply(&cur, &mv);
            if !problem.feasible(&next) {
                continue;
            }
            let next_cost = problem.cost(&next);
            let d = next_cost - cur_cost;
            let accept = d <= 0.0 || rng.next_f64() < (-d / t).exp();
            if accept {
                cur = next;
                cur_cost = next_cost;
                accepted += 1;
                if cur_cost < best_cost {
                    best = cur.clone();
                    best_cost = cur_cost;
                }
            }
        }
        t *= schedule.cooling;
    }
    AnnealResult {
        best,
        best_cost,
        accepted,
        proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 1-D placement toy: put n cells on integer slots minimizing total
    /// wirelength of given (i, j) nets, cells must not overlap (feasibility).
    struct Toy {
        nets: Vec<(usize, usize)>,
    }
    impl AnnealProblem for Toy {
        type State = Vec<i64>;
        type Move = (usize, i64);
        fn propose(&self, s: &Vec<i64>, rng: &mut SplitMix64) -> Option<(usize, i64)> {
            let i = rng.gen_range(s.len());
            let d = [-2i64, -1, 1, 2][rng.gen_range(4)];
            Some((i, d))
        }
        fn apply(&self, s: &Vec<i64>, mv: &(usize, i64)) -> Vec<i64> {
            let mut n = s.clone();
            n[mv.0] += mv.1;
            n
        }
        fn cost(&self, s: &Vec<i64>) -> f64 {
            self.nets
                .iter()
                .map(|(a, b)| (s[*a] - s[*b]).abs() as f64)
                .sum()
        }
        fn feasible(&self, s: &Vec<i64>) -> bool {
            let mut v = s.clone();
            v.sort_unstable();
            v.windows(2).all(|w| w[0] != w[1])
        }
    }

    #[test]
    fn converges_and_respects_feasibility() {
        let p = Toy {
            nets: vec![(0, 1), (1, 2), (2, 3)],
        };
        let init = vec![0, 20, -15, 40];
        let start_cost = p.cost(&init);
        let res = anneal(&p, init, &Schedule::default(), 42);
        assert!(res.best_cost < start_cost, "no improvement");
        assert!(p.feasible(&res.best), "best state infeasible");
        // Chain of 4 distinct integer slots: optimal wirelength is 3.
        assert!(res.best_cost <= 6.0, "far from optimum: {}", res.best_cost);
    }

    #[test]
    fn seeded_and_reproducible() {
        let p = Toy {
            nets: vec![(0, 1), (1, 2)],
        };
        let a = anneal(&p, vec![0, 9, -7], &Schedule::default(), 7);
        let b = anneal(&p, vec![0, 9, -7], &Schedule::default(), 7);
        assert_eq!(a.best, b.best);
        assert_eq!(a.accepted, b.accepted);
        let c = anneal(&p, vec![0, 9, -7], &Schedule::default(), 8);
        // Different seed may find a different (equally good) layout; only
        // require determinism per seed, so just touch the value.
        let _ = c.best_cost;
    }
}
