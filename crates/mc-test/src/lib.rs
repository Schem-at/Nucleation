//! The scenario descriptor and its evaluator — one vocabulary, many carriers.
//!
//! A scenario is a structure plus a list of player actions plus end-state
//! checks at named ticks. By default it is deliberately blind to *how* the
//! engine got there (no traces, no event order): a faster redstone backend
//! that still opens and resets the door passes unchanged. Cases that need to
//! pin ordering opt in with `events`, and every failure report carries the
//! recorded change log as diagnostics regardless.
//!
//! Carriers of the same descriptor:
//!
//! - a `*.test.json` beside an `.snbt` structure (mc-tick's `cases` test),
//! - a `.litematic` or `.schem` with the descriptor embedded in its root
//!   `NucleationTest` tag (nucleation's `litematic_cases` test and the
//!   `nucleation-test` CLI).
//!
//! See `crates/mc-tick/tests/cases/README.md` for the descriptor format.

pub mod block_based;
pub mod eval;
pub mod spec;

pub use block_based::synthesize_block_based;
pub use eval::{build_sim, report, run, run_with, try_build_sim, CaseResult, RunOptions};
pub use spec::{
    parse_suite, state_matches, Action, Case, Check, ContentExpect, EntityExpect, EventExpect,
    Expect, SettleMode, Snapshot, MARGIN,
};

/// The engine, re-exported so a carrier needs only this crate.
pub use mc_tick;
