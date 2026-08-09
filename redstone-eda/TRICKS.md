# TRICKS — techniques mined from community computational schematics

Source corpus: `computational_schematics/` (8 community `.schem` files, surveyed
2026-08-09). Each trick names the schematic + coordinates, and explains WHY it
works in terms of our probed material laws (`notes-material-model.md`).
Contradictions/extensions of the probed laws are flagged `PROBE:` and get
mc-tick verdicts appended as they are run.

Status: LIVE DOCUMENT for the overnight run — appended as analysis proceeds.
Corpus palette overview (counts from full block scans):

| schematic | size (blocks) | notable |
|---|---|---|
| ADD005_8bit_cle | 1217 | torch-heavy (141 wall torches), 32 repeaters, 18 targets, 17 levers, 26 lamps, 17 signs |
| ADD007_8bit_cca_matt | 691 | comparator+repeater balanced (41+41), 17 levers, 26 lamps — carry-cancel adder |
| BINTOBCD001 | 4140 | combinational double-dabble |
| DIVIDE009 | 9686 | 840 repeaters, 520 comparators, 64 hoppers+72 barrels (!), 16 redstone_block, clocked 5tpb |
| MULTIPY003 | 9839 | 920 comparators, 472 barrels, slabs; wool colour-coded lanes |
| NUMDISPLAY001 | 1270 | 87 repeaters, 49 lamps (7-seg), 9 targets, 4 levers |
| REGISTER001 | 202 | tiny: 26 repeaters, 10 comparators, 8 levers, 2 buttons, 16 lamps |
| REGISTERFILE001 | 9388 | 883 repeaters, 510 barrels, 493 comparators, buttons |

## Trick index (appended below as mined)

### T1. Diagonal-staircase operand interleave (ADD005_8bit_cle)
Bit i of BOTH operands lives at `y = 4+2i, z = 19-2i` (A at z+1, B at z-1 of
the sign column x=0); each bit-slice is a full-adder cell on its own diagonal
terrace, so the carry chain climbs 2y per bit while inputs stay reachable from
one wall. WHY it works: the 1-y step conducts up always (transparent-diode law
— uphill is unconditional), so a diagonal carry path needs no torch ladder;
2-block vertical pitch keeps the two dust lines non-adjacent (no diagonal in
use ⇒ solid separators legal). Sign convention: bit-weight signs (1..128) at
the lever wall AND at the output wall — machine-readable labels we parsed to
type the ports.

### T2. Signal-strength adder in 4-block depth (ADD007_8bit_cca_matt)
17x19x4 footprint for a full 8-bit adder + cin/cout, VERIFIED 10/10 in mc-tick
(walking ones, alternating patterns, 255+255+1). Vertical bit pitch y+2, A bank
at z=0, B bank at z=3, sum lamps at x=16, all on a 4-deep slab. 41 comparators
+ 41 repeaters and almost no torches (17): the per-bit cell is
comparator-arithmetic (carry-cancel), not dual-rail logic — comparators do
add/subtract in analog signal strength, halving cell volume vs our
torch-NOR cells. Coordinates: A=(0,3+2i,0), B=(0,3+2i,3), CIN=(6,0,3),
SUM=(16,3+2i,1), COUT lamp=(12,18,3).
PROBE: our cell library has no SS-arithmetic adder cell; mc-tick handles the
comparator math correctly (verdict: PASS, this probe).

### T3. Input echo lamps as a UI convention (ADD005, ADD007, MULTIPY003, DIVIDE009, REGISTERFILE001)
Every community build places a lamp IMMEDIATELY beside each input lever
(ADD005: lever (0,y,z) → lamp (1,y,z)). The lamp is powered by the lever
directly, not by circuit output — it is operator feedback only. For contract
extraction this means: a lamp adjacent to a lever is NOT an output port;
output lamp banks sit across the build. Autodetection heuristic adopted in the
enhanced copies: exclude lamps within 1 block of a lever.

### T4. LSB-first x-pitch-2 BCD bus, digit-packed (BINTOBCD001)
VERIFIED: 8 levers (2i,5,0) → 10 lamps (2i,4,47); output is 3 BCD digits
packed LSB-first: ones=(x0..x6), tens=(x8..x14), hundreds=(x16,x18).
Tested 0,1,9,10,99,100,255,137 — all exact. The double-dabble array is
purely combinational in a 19x19x49 volume; the whole datapath runs flat at
two y-levels (no vertical transport at all), trading z-length for simplicity.

### T5. 5x9 lamp matrix 7-segment, torch-decoded (NUMDISPLAY001)
VERIFIED: BCD nibble levers (29,2+2i,4) → 45-lamp 5x9 matrix at x=0 renders
digits (0/1/8 spot-checked pixel-exact). 9 targets in 3 columns distribute
segment nets vertically behind the panel: target blocks are dust-connectable
conductors, so one target column fans a segment net to 3 lamp rows without
side-connecting parallel dust lines (targets connect dust like a repeater
would, but omnidirectionally).
PROBE: target-block dust connectivity is not in our material table
(sturdy/conducts/cuts) — extended: target conducts + dust-connects on all
faces. Verdict from this build simming correctly in mc-tick: PASS.

### T6. Comparator-loop register with PULSED read-out (REGISTER001)
VERIFIED end-to-end in mc-tick: drive D=(0,3+2i,3), press Load button
(2,1,6) — value latches into a comparator feedback loop (comparators at
(4,1+2i,1..3) hold SS); clear D; press Read (4,1,6) — the stored byte
appears on the Q lamps (5,3+2i,3) **8 gt after the press, for only ~8 gt**,
then the display goes dark again while the button is still down (the read
path is monostable-pulsed, not level-gated). Read is non-destructive
(value survives). The whole 8-bit register is 6x18x7 = 202 blocks.
WHY: a comparator whose output feeds its own side is a lossless SS latch —
no repeater-lock pair per bit, half the cell size of our seq_cells DFF; the
pulsed display avoids burning lamps into the storage loop (reading taps the
loop through a subtract comparator only while the pulse lasts).
Consequence for contracts: outputs like this need a `read_btn` protocol
note; a plain settle-then-read executor sees ZERO. Buttons are also not
drivable by BackendCircuitExecutor (drive = lever toggle), so button ports
are declared but marked protocol-only.

### T7. Barrel+comparator ROM constants (MULTIPY003, DIVIDE009, REGISTERFILE001)
Hundreds of barrels with carefully-chosen item fills sit behind comparators
as analog constants (MULTIPY003: 472 barrels / 920 comparators; saved
comparator OutputSignals range 0..15). This is how community arithmetic
compresses: an 8x8 multiplier as SS lookup instead of a gate array.
PROBE (mc-tick verdict: PARTIAL): mc-tick DOES model container SS
(crates/mc-tick/src/behaviour.rs: container contents by position, comparator
reads through a conductor). DIVIDE009 (barrel-backed) computes correctly for
most vectors, so container SS is live. MULTIPY003 still settles to stable
WRONG products in every settle mode (P(1,1)=83 InWorld; stuck-bit baselines
in Placement/Quiet) — suspicion: constants built from non-64-stack items
(SS formula depends on max stack size) or a settle-order-sensitive SS loop.
UNRESOLVED — do not trust barrel-ROM builds without per-build verification.

### T8. Pipelined restoring divider, 11-block stage pitch (DIVIDE009)
8 identical compare-subtract stages at x-pitch 11 (target columns at
x=3,14,25,36,47,58,69,80), each climbing y+2 — dividend enters at x=89,
quotient bit i resolves at stage i, remainder emerges at x=0. Named
"5tpb" = 5 ticks per bit; measured answer latency ~90 gt for 144/12.
VERIFIED 9/13 vectors incl. 200/3=66r2, 100/64=1r36. Deterministic FAILS
(sampled stable over 1800 gt, fresh sim each): 255/16, 250/16, 100/16,
255/8, 99/2 — small power-of-two divisors with nonzero remainders lose
remainder (reads 0) and sometimes quotient bits. Either a real design
limit or an mc-tick comparator-timing divergence at the 5 gt/bit corner —
UNRESOLVED, flagged for a vanilla capture cross-check.

### T9. Redstone-block pairs as inter-stage rail injectors (DIVIDE009)
Each divider stage carries a redstone_block PAIR at (x,y,2)/(x+1,y,3)
climbing with the stage diagonal (16 total). They are constant-power rails
feeding the stage's subtract comparators — power that never has to be
routed from a bus. WHY legal: redstone_block is a power source, not a
conductor; it cannot back-feed the neighbouring stage because comparator
sides only read dust/repeater/comparator, and the diagonal offset keeps the
pair out of dust line-of-sight of stage i-1.

### T10. The community IO wall convention (all 8 schematics)
Uniform convention worth adopting for autodetection: input levers on one
build face at y-pitch 2 (wall levers, LSB at the bottom), an echo lamp
DIRECTLY beside each lever, output lamps on the opposite face at the same
pitch, weight signs (1/2/4/...) where the author bothered. Bit order is
ALWAYS geometric bottom-up / LSB-first (verified empirically in ADD005,
ADD007, BINTOBCD, DIVIDE, NUMDISPLAY). This matches our DesignPort
anchor+step model exactly — enhanced contracts were generated as
anchor=(bottom bit), step=(0,2,0) or (2,0,0), LSB-first, no exceptions.
