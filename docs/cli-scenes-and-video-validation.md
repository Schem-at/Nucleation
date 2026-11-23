# CLI scenes and video validation

Fresh checks on 18 November 2026, on `harrison-work` (Linux x86-64), using
Rust 1.98.0 (`88d9e12ae`, 2026-08-18), locked dependencies, release optimization
and the repository's LTO settings. FFmpeg was `n9.0.1`. The visual resource pack
was the existing Minecraft 1.21.11 model pack. No Rust compilation ran on the Mac.

## Automated checks

| Command | Result |
|---|---|
| `cargo test --locked --release -p nucleation-cli --bin nucleation --test cli` | 20 unit and 10 integration tests passed |
| `cargo test --locked --release -p nucleation --features rendering,mc-tick --lib rendering::` | 32 passed |
| `cargo check --locked --release -p nucleation-cli --no-default-features` | Passed |
| `cargo build --locked --release -p nucleation-cli` | Passed |
| `sh -n examples/cli/piston-camera.sh` and `git diff --check` | Passed |

New coverage checks authored-scene file/pipe round trips, metadata, editing
without modifying the source, invalid coordinates, full piston extension and
return at both zero and negative input origins, camera interpolation and pan,
ordered same-tick controls, invalid rendering options, stdin/stdout distinction,
and camera bounds derived from actual mesh vertices and movement endpoints.
The renderer suite also checks encoder failure handling and preservation of an
existing output file.

## Real command-line exports

The documented `examples/cli/piston-camera.sh` created a Litematic and rendered
its 80-tick cycle using the ordinary `nucleation scene` and `nucleation render`
commands. The lever is used at ticks 10 and 50. The camera changes yaw, pitch and
zoom. The final state is quiescent: lever off, piston retracted, gold block back
at `(1,1,0)`.

- MP4: H.264, 1280×720, 60 FPS, 240 decoded frames, 4.000 seconds.
- WebM: VP9, 320×180, 20 FPS, four decoded frames; decoded alpha includes both
  0 and 255, using the `libvpx-vp9` decoder.
- MOV: ProRes 4444, same short run; decoded alpha includes both 0 and 255.
- The short alpha runs also exercise a timed interaction and camera pan.
- `scene ... --to litematic -o - | render - -o -` produces a valid PNG stream.
- Extracted images at the initial, moving, extended, retracting and returned
  states were visually inspected. The complete geometry stays inside the frame
  throughout the inspected orbit; the piston head and payload move together.

The standard executable was installed at `/home/harrison/.local/bin/nucleation`.
Its SHA-256 is
`b538ae6ef0534fb957b3149dd75c44a5d49ffc7cbb7ae9d80a1eb1db8f7fe3d6`.

## Reproduction and artifacts

Follow [the CLI guide](cli-scenes-and-video.md), selecting a local resource pack
and FFmpeg. The example shell script invokes only the standard CLI. Compilation
used `CARGO_BUILD_JOBS=4` and `RUST_TEST_THREADS=4`.

Raw test logs, source schematic, videos, final-state JSON, FFprobe output,
decoded inspection frames and the export-check script are retained outside Git:

- Laptop: `/home/harrison/nucleation-perf/cli-scene-video/`
- Mac mirror: `/Users/harrison/RustroverProjects/Nucleation-schematic-archive/validation/cli-scene-video/`

These checks validate the CLI, shared simulation engine and visual export.
They do not add a fresh vanilla server capture or establish a simulation
speedup. Simulation mechanics and dependency versions are unchanged. Rendering
now belongs to the default CLI build; `--no-default-features` retains the smaller
build. Video encoding still requires external FFmpeg.

## Aperture hints and diagnostic video

On the SpaceWalker and SoulBanished hipster door, a 3×3 YZ filter and a nearby
point at (5,9,8) selected the passage from (5,8,7) to (5,10,9). Selecting
candidate 1, anchoring the centre at (5,9,8), and anchoring the minimum corner
at (5,8,7) independently produced the same bounds. The input was the lever at
(12,7,13). The first use filled all nine cells after seven ticks, and the
second cleared them after 39 ticks.

FFprobe counted 320 H.264 frames at 20 fps. For every decoded frame, the
number of orange cell centres in the side view matched the corresponding JSONL
occupancy count. Both phase reports matched the earlier CLI measurements and
the entire ordered trace matched byte for byte (SHA-256
`4ad1182c5305239112bbd311a2153b6f4c087ed40d0bf4c0675344979986729d`).
The PNG, MP4, JSONL and comparison script are in the laptop's
`cli-scene-video/aperture-review/` validation directory. The Mac archive copy
is under `validation/cli-door-stats/aperture/`.

## Hipster repeat-frequency reference

An independent check used the released 0.12.3 Python wheel with the user's
SpaceWalker and SoulBanished Litematic, seed zero, in-world settling, origin
(0,0,0) and ten warmup ticks. The slow 320-tick run matched all saved aperture
occupancy counts. The frequency search covered integer dwell times from 1 to
80 ticks after either input. Each candidate started at the same checkpoint.
A moving-piston cell kept the aperture in transit.

| Input | First aperture response | Completed endpoint | Motion |
|---|---:|---:|---:|
| Close | 1 tick | 9 ticks | 8 ticks |
| Open | 1 tick | 39 ticks | 38 ticks |

| Periodic schedule | Ticks after close | Ticks after open | Period | Result |
|---|---:|---:|---:|---|
| Use the isolated completion times | 9 | 39 | 48 | Failed on cycle 2 |
| Fastest verified | 9 | 44 | 53 | Passed 128 cycles, 0.3774 Hz |
| Fastest evenly spaced | 44 | 44 | 88 | Passed 128 cycles, 0.2273 Hz |

These frequencies describe the simulated aperture. The existing vanilla
capture covers the slower reference inputs. The high-frequency schedules were
measured with the released simulator.

### Hipster aperture and mechanism settling

Using the stored Minecraft 1.21.11 tick-end trace and a replay with the 0.12.3
wheel, the landed 3×3 closes at +9 game ticks and is fully clear at +39 after
the opening input. The last observed vanilla block change and the start of
continuous simulator queue quiescence occur at +16 on closing and +47 on
opening. Offsets count completed ticks since each input.

Watching only the lower two rows (`--corner 5,8,7 --size 3x2 --plane yz`)
gives an earlier first clearance on opening, +15. Those rows are obstructed at
+24 and stay clear from +39. This is cell occupancy in the selected plane.
