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
