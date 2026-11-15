# DECODE004 original programmed rows

Pinned original: `Logical-Redstone-Archive/DECODE004_decode_h2v.schem`, SHA-256 `09d6768f157d67845fa0615393077d610ed06c7fc9e57eb3721bb9bb3586267b`, save DataVersion 4189. No source edits, programmed-row rewrites or inert block substitutions are used.

The four real input levers are `(6,1,z)` for `z=4,6,8,10`, in LSB-first order. Their support lamps at `(5,1,z)` expose the input echo. Four output lamps are `(0,y,0)` for `y=2,4,6,8`, bottom to top.

Each source row has east-facing repeaters at `(1,y,4)` and `(1,y,8)`, west-facing wall torches at `(1,y,6)` and `(1,y,10)`, and a north-facing output-inversion wall torch at `(0,y,1)`. Thus each row computes `!(bit0 | !bit1 | bit2 | !bit3)`. All four rows are identically programmed: output is `0b1111` for input `0b1010`, otherwise zero. Claiming a one-hot or full 4-to-16 decoder for this original would be incorrect.

These positions and the equation follow the static inspection in `tools/prepare_tick_logic_archive.py` and the existing `check_logic_candidates.rs` port checks. The source hash and fresh NBT inspection fix the evidence independently of simulator outputs. The generated oracle is ordinary integer/Boolean logic with no simulation calls.

The 608-vector suite exhausts all 16 input values and all 256 ordered source/destination transitions, including unchanged values. Ascending, descending, Gray-code and reverse-Gray traversals additionally exercise different input sequences, followed by repeated transitions into/out of the programmed word. Actual within-boundary bit writes always follow the profile's declared LSB-first positions; this suite does not claim to test alternative intra-tick write order or transient glitch freedom.

Evaluation uses the prior native checker's explicit placement origin `(0,0,0)`, 128 setup ticks, and 32 response game ticks per vector. The saved Sponge Offset `[-2,-11,-7]` is retained in the unchanged file; this profile does not claim to reconstruct original server coordinates. The window is conservative, not a minimum latency measurement. Full checkpoint/output replay is checked by the shared carrier. A passing receipt proves these fixed native endpoints, not fresh vanilla parity.
