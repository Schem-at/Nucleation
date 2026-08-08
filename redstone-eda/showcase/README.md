# Redstone-EDA showcase gallery

Six `.schem` pieces demonstrating the `feat/redstone-eda` stack end to end,
all generated through the Python bindings (see `../demos/README.md` for the
wheel setup — `NUCLEATION_FEATURES=bridge-full,routing`).  Every piece was
sim-verified by its generator BEFORE saving and is saved BAKED at rest (all
levers off, circuit quiescent, settled block states written back), so what
you paste is exactly what the simulator proved.  Builds are colour-coded via
`rs.PALETTE` (gray rails, magenta routed wire + supports, orange gates,
purple inverters, ...).

Regenerate any piece with
`~/eda-venv/bin/python showcase_<name>.py` from this directory (each script
exits non-zero unless its verification is perfect).

| Piece | Script | Size (blocks, WxHxL) | What it demonstrates | Verification evidence |
|---|---|---|---|---|
| `router_gallery.schem` | `showcase_router.py` | 239, 13x10x45 | The NEW bridge routing API, four feats side by side: (A) obstacle maze — the route dives below grade under two offset walls; (B) vertical via — a torch-ladder climb to a platform 6 up; (C) three-net braid threading individual slots in a shared wall at 2-block clearance; (D) shared-trunk fork — a branch routed off an existing mid-trunk cell of the same net | 7/7 destinations conduct in-sim (power 0 -> >0 on lever flip); braid isolation proven (only the flipped net's dst powers); whole-build `Routing.drc` = 0 violations; baked at rest via `bake_to` |
| `adder4_cells.schem` | `showcase_adder4.py` | 988, 24x6x54 | Dense comparator-cell 4-bit ripple-carry adder: truth-tabled FA cells stamped at pitch, carry chain connected by abutment (zero routing) | EXHAUSTIVE 512/512 (16x16x2 input combinations) plus structural audit and net-short check = 0 |
| `mult4x4_stacked.schem` | `showcase_mult.py` | 27 222, 135x44x129 | 4x4 multiplier as four stacked planes (partial products + three Kogge-Stone accumulator rows) with 3D maze-routed inter-plane nets (dust, stairs, torch-ladder climbs) — the chip flow in miniature | EXHAUSTIVE 256/256 products; audit + net check clean |
| `kogge_stone_32bit.schem` | `showcase_ks32.py` | 154 152, 560x8x357 | The flagship: 32-bit Kogge-Stone prefix adder from the PLA compiler with channel-routed rails | 54/54 cases (47 seeded randoms + 7 corners) on the 65-lever bank; audit + net check clean |
| `alu8.schem` | `showcase_alu.py` | 30 052, 272x8x208 | 8-bit 4-op ALU (ADD/SUB/AND/XOR): the adder's internal g/p terms double as AND/XOR, plus B-select stage and a 4-term output mux | 144/144 cases (per-op corners + seeded randoms); audit + net check clean |
| `bus_riser8.schem` | `showcase_bus_riser.py` | 168, 9x8x15 | NEW bridge routing end-to-end: an 8-bit bus routed up two levels through a torch-ladder bank, one stateless `route_net` call per bit, skew-matched by identical lane geometry | 8/8 bits conduct and are isolated from neighbours; measured first-arrival = 4 ticks on EVERY bit (single-stepped); `Routing.drc` = 0 |
| `dff.schem` | `seq_cells.py` | 70, 13x4x7 | SEQUENTIAL: master-slave rising-edge DFF from repeater locks (master lock <- CLK, slave lock <- NOT CLK), Q bridged over its own clock column; baked at Q=0, wakes InWorld-quiescent in 0 gt | 11-point fixed-tick clocked protocol + 24-step random D/CLK vs model; baked Q=0 AND Q=1 reload + clock-after-reload; characterized setup 0 / hold 3 / min pulse 3 / clk->Q 10 / min period 20 gt |
| `register4.schem` | `seq_register4.py` | 280, 13x4x28 | 4-bit register: DFF stamps at pitch 7, clock chained by abutment (boundary rows hold only the clock column), 2 gt skew/bit; baked at Q=0 | 16 random write rounds + 16 hold-scramble rounds, all fixed-tick; baked reload InWorld quiescent in 0 gt |
| `counter4.schem` | `seq_counter.py` | 1 973, 50x6x57 | 4-bit synchronous counter -- the sequential loop closed: register + the FA cell column as increment (cin tied high), Q->a and sum->D feedback corridors with diagonal-guard capping, alias-aware net check clean | 24 clocked steps count mod 16; baked Q=0 reloads InWorld quiescent in 0 gt and counts 5 more; measured min period 100 gt (90 fails) |
| `accumulator4.schem` | `compositor/accumulator.py` | 2 059, 56x6x56 | 4-bit clocked ACCUMULATOR composed with the Compositor MVP: register4 + FA cell column, Q->a and sum->D corridors, external B bus west (flyovers over the feedback columns), shared clock; carry chain connected by `connect()` abutment; reset-by-bake at Q=0 | alias-aware nets.check clean; settles Q=0/D=0; baked reload InWorld quiescent in 0 gt; 24 random-B clocked steps match the running sum mod 16; measured min B-settle 80 gt (loop min period 100 gt); bridge DRC hard-clean, LVS opens=0; functional sim cross-check 24/24 (`functional_sim.py`) |
| `bus8_run.schem` | `bus8_probe.py` | 628, 41x16x1 | DENSE VERTICAL BUS: 8 bits stacked in y at 2y pitch, ONE block wide (dust / slab-top / dust per the probed material table; solid only under repeaters) — a 40-block run with TWO refresh stages per bit, station cells staggered diagonally (bit n at x = stage+n) so every repeater floor lands on the bit below's station blocks, never on live dust; bit 7's entry arrives at the probed ss1 minimum (15 dust cells) | 96/96 output checks (walking-ones + all-on + 0xAA/0x55 + all-off, 8 bits x 12 patterns), zero crosstalk; baked at rest |
| `bus_cross8.schem` | `bus8_cross.py` | 514, 17x17x17 | 8x8 90-DEGREE BUS CROSSING of two such vertical buses (A bits at odd y, B bits at even y): at the shared column every level is a block-sandwich station ENTRY block — a 16-block tower where the station blocks double as ALL 64 bit-pair isolations (y-parity technique generalized). The whole crossing is a 3x3-ground, 17-tall core with 16 repeaters (1/bit, doubling as each bus's refresh) | 432/432 output checks (walking-ones per bus 8+8, both-all-on, alternating pairs both ways, 8 seeded random joint patterns — 27 patterns x all 16 outputs), zero crosstalk; baked at rest |
| `hexanalog_trunk.schem` | `compositor/hexanalog.py` | 556, 54x2x57 | HexAnalog: 4 binary bits -> ONE wire's signal strength (comparator subtraction chain with wall-torch-inverted, exact-strength decay lanes), a 12-block one-wire trunk of alternating comparator/dust (lossless regeneration), and a 3-stage threshold + gated-subtract + comparator-merge decoder back to 4 bits | 9 analog primitive probes PASS (subtract-by-side-dust exact, comp pass-through/chaining exact, two-comps-into-one-dust = max, repeater-gated side); encoder 16/16 exhaustive; trunk entry strength == v AND decode 16/16 exhaustive |

Total: 13 pieces, 218 901 blocks, 1 676 verification cases/checks
(7 + 512 + 256 + 54 + 144 + 8 + 35 + 32 + 29 + 30 + 96 + 432 + 41
conduction/correctness checks).

## Notes

- Pieces 2–5 are regenerated by the existing self-verifying builders
  (`rca_cells.py`, `mult4.py`, `build_ppa.py`, `build_alu.py`); the
  `showcase_*.py` wrappers pin their arguments, parse the verification line
  and refuse to accept anything short of a perfect score.
- Pieces 1 and 6 are new and exercise the bridge `Routing.route_net` /
  `Routing.drc` API directly; conduction is proven through mc-tick
  (`TickSimulation`) and the settled state written back with `bake_to`.
- Bridge `route_net` is stateless per call (the negotiated multi-net
  `Workspace` is a native-crate API), so multi-net pieces keep nets apart by
  2-block clearance discipline — and prove electrical isolation in-sim,
  which is the check that actually matters.
