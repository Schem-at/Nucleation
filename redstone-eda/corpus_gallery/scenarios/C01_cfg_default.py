"""CONFIG 1/4 -- the default. Baseline for the triptych: no rule, default
materials, router's own cost weights (BusCost::default: length 1, delay 4,
skew 8, coherence 6, footprint 0.5)."""
import _shared

SCENARIO = _shared.serpentine(
    "C01_cfg_default",
    "Config A: defaults",
    "The serpentine with nothing configured. Every other C entry changes "
    "exactly one thing against this baseline.",
    notes="Compare `cells` / `delay_rt` / `footprint` against C02-C04.",
)
