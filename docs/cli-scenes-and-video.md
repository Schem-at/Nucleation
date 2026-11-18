# Scenes and video from the standard CLI

`nucleation scene` creates or edits a schematic. `nucleation render` renders a
still image or runs the shared tick engine and streams a video to external
FFmpeg. No custom executable, browser or intermediate PNG sequence is needed.
The default CLI build includes rendering:

```sh
cargo install --locked --path crates/nucleation-cli
```

FFmpeg must be on `PATH`, or selected with `render --ffmpeg /path/to/ffmpeg`.
Select a Minecraft resource pack with `--pack`, `NUCLEATION_PACK`, or the existing
installed-client discovery. The pack supplies visual models and textures; it does
not select the simulation's physics version. A lightweight CLI build remains
available with `--no-default-features`.

## Place a piston and lever, then film them

```sh
export NUCLEATION_PACK=/path/to/minecraft-models.zip

nucleation scene --name 'Piston and lever' --data-version 4671 \
  --set '0,0,0=minecraft:stone' \
  --set '0,0,1=minecraft:stone' \
  --set '0,1,0=minecraft:sticky_piston[facing=east,extended=false]' \
  --set '0,1,1=minecraft:lever[face=floor,facing=north,powered=false]' \
  --set '1,1,0=minecraft:gold_block' \
  -o piston.litematic

nucleation render piston.litematic -o piston.mp4 \
  --ticks 80 --use 10:0,1,1 --use 50:0,1,1 \
  --camera 0:45,30,1 --camera 80:225,40,1.2 \
  --width 1280 --height 720 --fps 60 --report result.json
```

This makes a four-second video: the lever powers the piston at 0.5 seconds,
then releases it at 2.5 seconds, while the camera orbits and zooms. The gold
block is pushed out and pulled back. The shell script
[`examples/cli/piston-camera.sh`](../examples/cli/piston-camera.sh) runs these same
standard commands and keeps its output in a chosen directory.

## Authoring

Repeat `--set 'X,Y,Z=BLOCK_STATE'` to place blocks. Changes apply in command-line
order; `minecraft:air` removes a block. Supply an input schematic to edit a copy:

```sh
nucleation scene piston.litematic --set '1,1,0=minecraft:diamond_block' -o diamond.schem
nucleation scene --set '0,0,0=minecraft:stone' --to litematic -o - |
  nucleation render - -o stone.png
```

Imports use content detection, including Litematic, Sponge and SNBT. Output
extensions choose the exporter, or use `--to FORMAT`. `--data-version` stamps
the authored block states; it does not translate state names to another version.
Imported save metadata is preserved unless explicitly changed.

## Shape brushes and formulas

```sh
nucleation scene --cuboid '0,0,0:20,0,20' --block minecraft:stone -o base.schem
nucleation scene base.schem --sphere '10,5,10' --radius 4 --block minecraft:glass -o dome.schem
nucleation scene --curve '[12*cos(t), 4*t, 12*sin(t)]' \
  --block minecraft:oxidized_copper --radius 1.5 --steps 128 -o helix.litematic
```

Shapes and single-block edits apply in argument order. The shape brush settings
apply to all shapes in the invocation. The solid brush defaults to stone, the
radius to 1.5 blocks, and curves to 128 intervals. The formula receives a jq
input from zero through one. The shorthand `t` spans zero through 2π and the
formula returns three coordinates in schematic space (Y up). Curve sampling
requires jq on PATH. The building library connects the samples with straight
segments and fills the resulting tube with the selected block.

## Select records from a schematic

```sh
nucleation query entities build.litematic --limit 10
nucleation query block-entities build.litematic --id minecraft:chest --nbt |
  jq -c '{pos, items: .nbt.Items}'
```

Each matching record occupies one line, with its region, ID and position.
`tile-entities` is an alias for `block-entities`. Add `--nbt` for decoded
payload data. ID filtering and the optional result limit apply before JSON
serialization. Input decoding still loads the schematic into memory.

## Measure a door

```sh
nucleation door-stats door.litematic
nucleation door-stats door.litematic --list-inputs
nucleation door-stats door.litematic --input 12,7,13 --ticks 160 --json
```

The simulator loads in-world, seeds its RNG with zero, warms up for ten game ticks,
then takes a checkpoint. It observes two equal windows with no input, restores
the checkpoint, and observes the same windows with one interaction at the start
of each. Each window defaults to 160 game ticks. Use `--warmup` and `--ticks`
to change these durations. Inputs use saved schematic coordinates. The default
redstone hash origin is the schematic minimum and can be set with `--origin`.

Detection considers levers, buttons and note blocks. It selects the only
candidate, or prefers a single lever among other candidates. Multiple equally
plausible inputs require `--input`. Use `--list-inputs` for their coordinates
and states after warmup.

Each use reports block writes, distinct positions written, successful piston
extensions and retractions (including drops), recorded slot and entity events,
the last write, and whether engine queues are quiet at the endpoint. The JSON
includes endpoint block differences from both the initial layout and idle
replay. Layout restoration compares block states and positions. A changed
note pitch, for example, counts as a difference. If queues are still active at
the end, extend the observation window.

When the first use settles, the CLI looks for a single complete rectangular
sheet (at least 2×2) that changed between air and one propertyless block state.
It groups face-connected cells with the same transition and excludes cells
whose idle endpoint differed from their initial state. A unique sheet is
reported as an **inferred** panel. Mixed materials, several panels, or a moving
endpoint can leave the aperture undetermined. An explicit box overrides inference.

Use `--aperture x1,y1,z1:x2,y2,z2` to watch the inclusive box containing
the actual passage. The result gives its endpoint occupancy (clear, filled,
or partial) and when that occupied-cell count last changed. A separate timestamp reports when the exact endpoint block states last changed. This samples air
versus non-air at tick ends. Collision shapes, traversability and mechanical
settling have different meanings. Without an aperture the two phases are
reported as uses 1 and 2.

### Benchmark opening, closing and repeat frequency

```sh
nucleation door-stats door.litematic --frequency --cycles 16 --max-dwell 80 --json
```

Each use reports separate elapsed game ticks from the input:

| JSON field | What it measures |
|---|---|
| `delay_ticks` | First departure from the starting aperture state |
| `aperture_first_ticks` | First tick the entire aperture is clear on opening or fully filled with landed blocks on closing |
| `aperture_stable_ticks` | First tick of the final uninterrupted run at that target through the observation window |
| `mechanism_settled_ticks` | Start of the final quiet interval for the simulated build, confirmed by a following quiet tick |
| `settle_tail_ticks` | Additional ticks from the stable aperture to mechanism settlement |

The aperture is sampled at tick ends, starting at +1. Mechanism settlement
requires the scheduled tick, fluid, block event and piston move queues to be
empty, with no subsequent recorded block, inventory or entity changes during
the window. For a continuously active build or a short dwell, settlement can
be null even when the passage is clear. An unobserved aperture milestone is
also null. `completion_ticks` retains its original meaning of stable aperture
time, and `motion_ticks` measures from initial response to that time.

`--frequency` searches periodic schedules with one use to open and one to close.
Each dwell ranges from one through `--max-dwell` ticks (default 80).
The result gives both the fastest passing schedule and the fastest with equally
spaced inputs. One hertz means one full open/close pair per second.
`--cycles` (default 8, minimum 2) sets the number of uninterrupted cycles a
schedule must pass. Either numeric flag enables the search.

Each schedule starts from the same warm checkpoint and RNG seed. Within a
schedule, inventory, queues and the entire mechanism carry forward between uses.
Every half-cycle must reach its full target by the next input and hold from its
first arrival to that input. Reaching the target on the final tick qualifies.
Intermediate partial motion may reverse direction. A different block material
in the doorway or a shuffled mechanism elsewhere is allowed. The separate
full-layout restoration statistic is informational. Results apply to the
tested starting state, dwell bounds and cycle count. The JSON includes the
per-cycle opening and closing times for each winning schedule. Preview, video
and JSONL timeline continue to describe the original two-use measurement.

### Locate and preview a passage

Start with the dimensions you know. Add a plane or a point near the passage to
narrow the matching panels. A distance tie stays ambiguous. The JSON report
lists matching bounds and one-based candidate numbers in schematic coordinates.

```sh
nucleation door-stats door.litematic --size 3x3 --preview door.png --preview-only
nucleation door-stats door.litematic --size 3x3 --near 5,9,8 --preview door.png --json
```

With several matches, keep the same size and plane filters and add
`--candidate 2` to select the second. If detection misses the panel entirely,
supply one corner or centre plus its size. These placements also work on mixed
materials and one-cell-wide passages. An explicit preview works before choosing
an input, even on a schematic with several levers.

```sh
nucleation door-stats door.litematic --plane yz --center 5,9,8 --size 3x3 --preview door.png --preview-only
nucleation door-stats door.litematic --plane yz --corner 5,8,7 --size 3x3 --record cycle.mp4 --timeline cycle.jsonl --json
```

Width and height follow X/Y on `xy` (the anchored default), Z/Y on `yz`, and
X/Z on `xz`. A corner is the minimum on both axes. For an even dimension, the
centre coordinate selects the lower of the two middle cells on that axis.

The PNG shows front, side and plan views after warmup. Teal outlines mark open
passage cells, orange marks occupied cells, and a yellow cross locates the
selected control. Unresolved candidates are numbered in purple. The diagnostic
MP4 shows these same projections at the end of each measured game tick, with
the use number, time and occupancy. The standard CLI streams frames to FFmpeg
at 20 fps. Warmup and idle replays are excluded. PNG previews also work in the
lightweight build.

Add `--timeline cycle.jsonl` to save one JSON object per observed cycle tick.
Each row includes ordered block changes, piston events with their place in
that row's change order, queue status, and optional aperture occupancy. Positions
are in input coordinates. The first row of each window includes the interaction.
Use a new filename. Ticks in events are absolute simulation ticks. Elapsed
ticks count completed steps since the interaction (20 game ticks per second
at normal playback speed).

## Video controls

| Option | Meaning |
|---|---|
| `--ticks N` | Number of game ticks to execute; default 100 |
| `--tick-ms N` | Playback duration of each tick; default 50, or 20 TPS |
| `--fps N` | Output frame rate, independent of simulation tick rate; default 60 |
| `--use T:X,Y,Z` | Interact with a lever, button or note block |
| `--place T:X,Y,Z=STATE` | Place a block through the engine's update path |
| `--break T:X,Y,Z` | Replace a block with air through that same path |
| `--settle in-world\|quiet\|placement` | Saved-world load, on-place-only, or full placement; default in-world |
| `--seed N` | Simulation RNG seed; default 0 |
| `--origin X,Y,Z` | World hash origin for order-sensitive redstone |
| `--camera T:YAW,PITCH,ZOOM[,PAN_X,PAN_Y,PAN_Z]` | Repeat to specify camera keyframes |
| `--yaw`, `--pitch`, `--zoom` | Initial camera settings |
| `--tight` | Fit the camera to the current viewing angle; otherwise video uses a stable sphere fit |
| `--width`, `--height` | Output resolution; video defaults to 1280×720, PNG to 1024×1024 |
| `--report FILE.json` | Final simulation state, quiescence, frame count and camera endpoints |

Actions use **input schematic coordinates**, including negative coordinates.
Conversion to the simulator's local origin is handled internally and recorded in
the report. Inputs at boundary `T` run before that tick is stepped; tick 0 is the
start, and same-tick actions retain command-line order. Actions at or beyond the
end are rejected. Rendered intervals show the completed tick's state, with moving
blocks interpolated using the engine's own stroke times, carried state, short
head and stationary retraction body.

Camera keyframes interpolate linearly in tick time. Yaw is unwrapped: `45→405`
makes a full revolution. Pitch and zoom are absolute; optional pan coordinates
are offsets from the fitted camera target. The first/last keyframes hold outside
their interval. If the first keyframe follows tick 0, the initial camera settings
supply the starting keyframe. Frames cover `[0, ticks)`; a keyframe at the end
sets the interpolation target without adding an extra duplicate endpoint frame.

Use `.mp4` for opaque H.264, `.webm --transparent` for VP9 with alpha, or
`.mov --transparent` for ProRes 4444. H.264 requires even dimensions. Video needs
an output filename; still PNGs retain stdout piping. Unknown or incompatible
options are errors rather than silently ignored settings.

## Implementation and limits

Simulation uses `mc_test::try_build_sim`, including inventories, block entities,
entity loading and source data-version motion semantics. It does not use the
older example's separate simulation initialization. Piston visualization reads
`Simulation::moving_blocks()` directly rather than guessing from adjacent
changes. Native entity models are used where available; a missing model produces
a warning.

Blocks that stay constant for the entire run are meshed as one static group.
Dynamic position/state meshes are reused, and camera changes only update poses
and camera uniforms. Video framing includes actual mesh vertices and all captured
movement endpoints, so block-centred models and moving payloads share a stable
orbit target. Frame descriptors and pixels stream to the encoder; neither
is retained for the whole movie. Visible state spans, entity tracks and unique
meshes are retained, so a long, highly dynamic circuit can still use substantial
memory. This change does not make a simulation speedup claim.

The existing `nucleation animate` GLB command and the older rendering example
remain available. PNG rendering stays static; scheduled inputs and animated
camera controls require a video output extension.

See the [validation record](cli-scenes-and-video-validation.md) for the tested
toolchain, command results and retained example outputs.
