# Enhanced community schematics — analysis & contract report

Run: 2026-08-09, mc-tick (`TickSimulation`, vanilla-accurate), branch
`feat/redstone-eda`. Every `*_enhanced.schem` here is a copy of the original
in `computational_schematics/` with a `CellContract` embedded in the `.schem`
metadata (`NucleationCellContract`), autodetected on open — originals
untouched. All contracts round-trip (save -> open -> `cell_contract_json`)
and every declared position was validated against the actual hardware
(levers/buttons for inputs, lamps for outputs). Bit order is LSB-first by
geometric order (bottom-up / anchor-outward) in every port.
Tricks mined from these builds: `redstone-eda/TRICKS.md` (T1-T10).

## ADD005_8bit_cle — 8-bit adder, diagonal-staircase layout
- IO: `a`,`b` uint8 (levers interleaved on the diagonal wall, weight signs
  1..128), `cin` bool (5,1,18), `sum` uint8 lamps (11,1+2i,15-2i), `cout`
  bool lamp (10,18,7). Sign "(2 ticks)" beside COUT.
- Verified: 10/10 vectors in mc-tick (incl. 255+255+1, alternating
  patterns). Status: VERIFIED, contract embedded.

## ADD007_8bit_cca_matt — 8-bit carry-cancel adder (comparator SS-arithmetic)
- IO: `a` (0,3+2i,0), `b` (0,3+2i,3) uint8; `cin` bool (6,0,3); `sum` uint8
  (16,3+2i,1); `cout` bool lamp (12,18,3). Footprint 17x19x4.
- Verified: 10/10 raw vectors + typed CellExecutor spot-check through the
  embedded contract (99+28=127). Status: VERIFIED.

## BINTOBCD001 — 8-bit binary -> 3-digit BCD (double dabble, combinational)
- IO: `bin` uint8 levers (2i,5,0); outputs `bcd_ones` uint4 (0..6,4,47),
  `bcd_tens` uint4 (8..14,4,47), `bcd_hundreds` uint2 (16/18,4,47).
- Verified: 8/8 vectors (0,1,9,10,99,100,255,137) + typed spot-check
  (173 -> 1/7/3). Status: VERIFIED.

## NUMDISPLAY001 — BCD -> 7-segment on a 5x9 lamp matrix
- IO: `bcd` uint4 levers (29,2+2i,4); outputs `seg_a..seg_g` bool, one
  representative lamp per segment of the x=0 matrix.
- Verified: digits 0/1/8 pixel-exact on the full matrix; typed spot-check
  digit 7 -> segments a,b,c only. Status: VERIFIED.

## REGISTER001 — 8-bit register, comparator-loop storage, pulsed read
- IO: `d` uint8 levers (0,3+2i,3); `load_btn` (2,1,6), `read_btn` (4,1,6)
  (BUTTONS — see protocol); `q` uint8 lamps (5,3+2i,3).
- Verified (raw sim): load 0xA5, clear D, press Read -> q shows 0xA5 in a
  window 8-16 gt after the press (~8 gt wide), non-destructive. VERIFIED.
- Caveat: `CellExecutor` cannot drive button ports (backend drive is a lever
  toggle) — typed executor errors on `load_btn`; the contract documents the
  geometry, the protocol needs `use_block` pulses.

## MULTIPY003 — 8x8->16 multiplier, barrel-comparator ROM — NOT VERIFIED
- IO (geometric): `a` (59,6+2i,23), `b` (61,6+2i,23) uint8 levers (echo
  lamps confirm they reach the circuit); `p` uint16 lamp column (0,1+2i,12).
- Honest unknown: settles quiescent but WRONG in all three settle modes
  (e.g. 1*1 -> 83 InWorld; stuck-bit baseline 17722 in Placement/Quiet).
  mc-tick does model container SS (DIVIDE009 works), so suspicion falls on
  non-64-stack item constants or settle-order-sensitive SS loops. Contract
  embedded with mapped geometry; treat function as UNVERIFIED.

## DIVIDE009 — pipelined 8-bit divider (5 gt/bit, 8 stages, pitch 11)
- IO: `dividend` (89,1+2i,2), `divisor` (89,16+2i,7) uint8 levers;
  `remainder` uint8 lamps (0,0+2i,2), `quotient` uint8 lamps (0,18+2i,3).
- Verified: 9/13 vectors (100/7, 37/5, 200/3, 8/9, 144/12, 255/255, 1/1,
  100/32, 100/64, 99/4, 240/16, 255/17 pass; latency ~90 gt), typed
  spot-check 100/7 -> q14 r2 through the embedded contract. Deterministic
  failures (stable over 1800 gt sampling, fresh sim each): 255/16, 250/16,
  100/16, 255/8, 99/2 — small power-of-two divisors with nonzero remainder
  lose the remainder (reads 0) and sometimes quotient bits; design limit vs
  mc-tick divergence UNRESOLVED. Status: VERIFIED-WITH-CAVEATS.

## REGISTERFILE001 — 16x8 register file, 2 display ports — PARTIALLY MAPPED
- IO (geometric): `data_in` uint8 (9,16+2i,33); three 4-bit address banks
  `addr_a` (5,4+2i,34), `addr_b` (11,4+2i,34), `addr_c` (13,4+2i,34);
  `enable` lever (9,1,36) (saved ON); `rw_btn` button (7,5,35); `port_a`
  lamps (1,17+2i,0), `port_b` lamps (17,17+2i,0).
- Verified: with enable ON, pressing the button pulses the CURRENT data_in
  value onto BOTH lamp banks (0x5A -> (90,90), 0x77 -> (119,119)) — a
  write-echo. With enable OFF nothing displays. Storage/readback by address
  could NOT be demonstrated (address changes alone never light the ports;
  barrel-backed cells — same container-SS caveat as MULTIPY003). Contract
  embedded with mapped geometry + `enable:1` initial port value; function
  UNVERIFIED beyond the write-echo.

## Method notes
- Sim: `TickSimulation.from_schematic(s, InWorld|Placement, 0,0,0, EXTRA)`
  where EXTRA interns wall/floor/ceiling lever, button, comparator and
  target states on top of `rs.EXTRA_STATES` (community builds use wall
  levers, which the default set does not intern).
- Lamp adjacency heuristic: a lamp within 1 block of a lever is an input
  echo, never an output (TRICKS.md T3) — kept out of every output port.
- Typed spot-checks used `CellExecutor.for_schematic` on the REOPENED
  enhanced file — i.e. the contract that ships in the artifact, not an
  in-memory one.
