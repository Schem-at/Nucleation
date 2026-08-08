/** Sim Lab: load any schematic, fly through it, click its levers, watch the
 * tick engine run — all in the browser, no backend.
 *
 * The loop is deliberately simple: advance the simulation at the chosen
 * rate, drain its change log, patch the affected chunks once per frame.
 * Everything the user can do to the world (a lever, a button, a note block)
 * goes through `useBlock`, the same call a right-click makes in the game.
 */

import * as THREE from "three";
import React, { useCallback, useEffect, useRef, useState } from "react";
import { World, loader, CHUNK, type SettleName } from "./world";
import { Player } from "./player";
import { addDefaultLights } from "./mesher";
import { TimelineStrip, type Activity } from "./timeline";

type Status = { kind: "idle" | "loading" | "ready" | "error"; message?: string };

const INTERACTIVE = /lever|button|note_block|trapdoor|_door|_gate|repeater|comparator|daylight/;

/** How much of a frame the simulation may have.
 *
 * The rest belongs to meshing and drawing. Ten milliseconds of a 16 ms frame
 * leaves enough to stay interactive while the tab is saturated — the tab going
 * unresponsive is the one failure mode that makes a throttle indicator useless,
 * because you cannot read it.
 */
const TICK_BUDGET_MS = 10;

const SLIDER_MAX = 1000;
/** Where the fine regime ends, in slider travel and in tps. */
const FINE_END = 600;
const FINE_TPS = 20;
/** Top of the coarse regime. Past this the slider asks for everything. */
const COARSE_TPS = 20_000;
/** Slider pinned to the top: run as many ticks as the frame budget allows and
 * report what that turned out to be. */
const UNCAPPED = Infinity;

/** Slider position to ticks per second.
 *
 * Two regimes, because the useful range is not uniform. Below ~20 tps you are
 * watching a machine work and want to land on exactly 3, or 7, or 0.5 — so the
 * bottom 60% of the travel is linear there, about 0.03 tps per step. Above it
 * you are finding out where it breaks and only order of magnitude matters, so
 * the top 40% is exponential. The last notch is uncapped.
 */
function sliderToTps(p: number): number {
  if (p >= SLIDER_MAX) return UNCAPPED;
  if (p <= FINE_END) return Math.max(0.1, Math.round((p / FINE_END) * FINE_TPS * 100) / 100);
  const t = (p - FINE_END) / (SLIDER_MAX - FINE_END);
  return Math.round(FINE_TPS * Math.pow(COARSE_TPS / FINE_TPS, t));
}

/** Ticks per second, written the way a reader wants it at that magnitude. */
function fmtTps(tps: number): string {
  if (tps === UNCAPPED) return "max";
  if (tps >= 10_000) return `${(tps / 1000).toFixed(0)}k`;
  if (tps >= 1000) return `${(tps / 1000).toFixed(1)}k`;
  if (tps >= 100) return tps.toFixed(0);
  if (tps >= 10) return tps.toFixed(1);
  return tps.toFixed(2);
}

export default function App(): JSX.Element {
  const mount = useRef<HTMLDivElement | null>(null);
  const sceneRef = useRef<THREE.Scene | null>(null);
  const worldRef = useRef<World | null>(null);
  const playerRef = useRef<Player | null>(null);
  const outlineRef = useRef<THREE.LineSegments | null>(null);
  const flashRef = useRef<THREE.Mesh | null>(null);
  const flashUntil = useRef(0);
  const lastFile = useRef<File | null>(null);
  const runningRef = useRef(false);
  const rateRef = useRef(10);

  const [status, setStatus] = useState<Status>({ kind: "idle" });
  const [running, setRunning] = useState(false);
  /** Slider travel, not tps — the mapping between them is not linear. */
  const [slider, setSlider] = useState(300);
  const rate = sliderToTps(slider);
  /** Ticks per second actually achieved, and whether that fell short of what
   * was asked. Measured rather than assumed: the whole point of pushing the
   * rate up is to find where the engine stops keeping up, and a readout that
   * echoed the target back would hide exactly that. */
  const [actualTps, setActualTps] = useState(0);
  const [throttled, setThrottled] = useState(false);
  const [stepN, setStepN] = useState(20);
  const [settle, setSettle] = useState<SettleName>("in-world");
  /** Mesh-chunk edge. Changing it reloads the build, because chunking is
   * decided when the scene is built. `0` is one chunk for the whole thing. */
  const [chunkSize, setChunkSize] = useState<number>(CHUNK);
  const [tick, setTick] = useState(0);
  /** Entities on screen. Worth its own readout: a build can be all boats and
   * carts, and until they were drawn at all an empty-looking room that ticked
   * was indistinguishable from a broken load. */
  const [entities, setEntities] = useState(0);
  const [target, setTarget] = useState<string>("");
  const [info, setInfo] = useState<string>("");
  const [locked, setLocked] = useState(false);
  /** Mirrors `World.isRecording()` for the button label — the world is the
   * source of truth (and resets it on load/`startSim`), this just re-renders
   * the bar when it changes. */
  const [recording, setRecording] = useState(false);
  /** The activity strip's data. Polled, not derived every render —
   * `timelineActivityJson()` re-scans the whole accumulated log (see its
   * doc comment in `src/bridge/mc_tick.rs`), so its cost grows with the
   * recording. Calling it from the frame loop would put an O(total log)
   * read on the render path — exactly the unbounded-cost mistake the
   * timeline-prerequisites work removed elsewhere. A 250 ms timer instead
   * bounds how often that cost is paid, independent of frame rate. */
  const [activity, setActivity] = useState<Activity | null>(null);

  useEffect(() => {
    runningRef.current = running;
  }, [running]);
  useEffect(() => {
    rateRef.current = rate;
  }, [rate]);

  /** Read `timelineActivityJson()` once, right now. Shared by the polling
   * timer below and by the stop button's one extra read, so the strip's
   * final frame always reflects the tick the recording actually ended on. */
  const pollActivity = useCallback(() => {
    const world = worldRef.current;
    if (!world?.sim?.timelineActivityJson) return;
    try {
      setActivity(JSON.parse(world.sim.timelineActivityJson()) as Activity);
    } catch {
      /* a malformed read leaves the strip showing its last good frame */
    }
  }, []);

  // Poll on a fixed wall-clock timer while recording — deliberately not in
  // the `requestAnimationFrame` loop above, and deliberately not on every
  // tick. See the comment on `activity` for why a per-frame poll would be
  // the wrong place to pay this call's cost.
  useEffect(() => {
    if (!recording) return;
    pollActivity();
    const id = setInterval(pollActivity, 250);
    return () => clearInterval(id);
  }, [recording, pollActivity]);

  // Scene, camera, render loop — created once.
  useEffect(() => {
    const host = mount.current;
    if (!host) return;
    const scene = new THREE.Scene();
    sceneRef.current = scene;
    scene.background = new THREE.Color(0x0f1115);
    const camera = new THREE.PerspectiveCamera(75, 1, 0.1, 2000);
    const renderer = new THREE.WebGLRenderer({ antialias: true });
    renderer.setPixelRatio(Math.min(devicePixelRatio, 2));
    host.appendChild(renderer.domElement);
    addDefaultLights(scene);
    const player = new Player(camera, renderer.domElement);
    playerRef.current = player;

    // The block outline: the game's own affordance, so you can see what a
    // click will hit before you make it. Slightly larger than the cell so
    // it never z-fights the block's own faces.
    const outline = new THREE.LineSegments(
      new THREE.EdgesGeometry(new THREE.BoxGeometry(1.002, 1.002, 1.002)),
      new THREE.LineBasicMaterial({ color: 0x000000, depthTest: true }),
    );
    outline.visible = false;
    scene.add(outline);
    outlineRef.current = outline;

    // A short-lived green box flashes where a click landed, so an
    // interaction that changes nothing visible still shows it registered.
    const flash = new THREE.Mesh(
      new THREE.BoxGeometry(1.06, 1.06, 1.06),
      new THREE.MeshBasicMaterial({ color: 0x6fd66a, transparent: true, opacity: 0.45 }),
    );
    flash.visible = false;
    scene.add(flash);
    flashRef.current = flash;

    const resize = () => {
      const { clientWidth: w, clientHeight: h } = host;
      renderer.setSize(w, h, false);
      camera.aspect = w / Math.max(h, 1);
      camera.updateProjectionMatrix();
    };
    resize();
    const observer = new ResizeObserver(resize);
    observer.observe(host);

    let raf = 0;
    let last = performance.now();
    let carry = 0;
    let flushing = false;
    // Rolling measurement of what the rate actually came out to.
    let ran = 0;
    let shortfall = false;
    let measuredAt = performance.now();
    const frame = (now: number) => {
      raf = requestAnimationFrame(frame);
      const dt = Math.min((now - last) / 1000, 0.1);
      last = now;
      player.update(dt);
      setLocked(player.pointerLocked);

      const world = worldRef.current;
      if (world?.sim) {
        // How far the pacing has carried toward the next tick. Zero while
        // paused: nothing is advancing, so a stroke caught mid-flight should
        // hold its position rather than drift on a clock the simulation is
        // not running on. Stepping by hand then walks it forward a tick at a
        // time, which is the point of stepping by hand.
        let subTick = 0;
        if (runningRef.current) {
          // What was asked for this frame. Uncapped asks for as much as the
          // budget will take, which is the only honest way to answer "how
          // fast can this go" — any fixed number is a guess about a machine.
          let want: number;
          if (rateRef.current === UNCAPPED) {
            want = Infinity;
            carry = 0;
          } else {
            carry += dt * rateRef.current;
            want = Math.floor(carry);
            carry -= want;
          }

          // Spend ticks until the budget runs out rather than stopping at a
          // fixed count. The old cap of 200 a frame was a ceiling of about
          // 12k tps on any build, however small — it throttled the engine
          // and called it the engine's limit.
          const spendStart = performance.now();
          let steps = 0;
          let outOfBudget = false;
          while (steps < want) {
            world.sim.step();
            steps++;
            // Checking the clock every tick would cost more than a tick on a
            // small build, so amortise it.
            if ((steps & 63) === 0 && performance.now() - spendStart >= TICK_BUDGET_MS) {
              outOfBudget = true;
              break;
            }
          }
          // Uncapped is budget-limited by definition, so say so rather than
          // reporting a clean run: at the top of the slider the number on
          // screen *is* the ceiling, and it should be labelled as one.
          const missed = want === Infinity ? outOfBudget : steps < want;
          // Asked for more than the budget bought: drop the debt instead of
          // letting `carry` grow without bound, which would make the machine
          // sprint to catch up the moment the rate came back down.
          if (missed) carry = 0;

          // Against the wall clock, not against time spent inside `step()`.
          // Ticks per second means ticks per second of the user's time —
          // dividing by stepping time alone would report the engine's
          // throughput and quietly hide every frame it did not get to run.
          ran += steps;
          shortfall = shortfall || missed;
          const window = now - measuredAt;
          // Wait for enough ticks to divide by, not just enough time. A fixed
          // 250 ms window sees one tick or none at 2 tps, so the readout
          // quantises to multiples of four and a correctly-paced 2 tps reads
          // as 4 — the measurement lying about the thing it exists to check.
          // Ten ticks is enough to be accurate; the 2 s ceiling keeps a very
          // slow rate updating at all.
          if (window >= 250 && (ran >= 10 || window >= 2000)) {
            // Kept fractional: rounding to an integer here would report a
            // deliberate 0.5 tps as either 0 or 1.
            setActualTps(ran / (window / 1000));
            setThrottled(shortfall);
            ran = 0;
            shortfall = false;
            measuredAt = now;
          }
          // One drain for the whole batch: the log is cumulative, so
          // reading it per step is quadratic.
          if (steps) {
            world.applyChanges(world.drainChanges());
            setTick(Number(world.sim.tickCount?.() ?? 0));
            setEntities(world.entityCount());
          }
          subTick = Math.min(carry, 1);
        }
        // A fractional tick, not a wall-clock stamp: whole ticks completed plus
        // however far the pacing has carried toward the next. Strokes are
        // interpolated against the same clock the simulation advances on, so
        // one stepped tick and fifty are both drawn correctly and neither can
        // desync — which is what the old `steps === 1` guard was papering over
        // by teleporting everything above it.
        world.animate(Number(world.sim.tickCount?.() ?? 0) + subTick);
        if (!flushing) {
          flushing = true;
          void world.flush().finally(() => {
            flushing = false;
          });
        }
        // What the crosshair is on: show it in the HUD and box it in the
        // world, and colour the box by whether a click would do anything.
        const hit = player.pick((x, y, z) => world.isSolid(x, y, z));
        const box = outlineRef.current;
        if (box) {
          box.visible = !!hit;
          if (hit) {
            // Blocks are centred on integers, so the cell's centre *is* its coordinate.
            box.position.set(hit.pos[0], hit.pos[1], hit.pos[2]);
            const state = world.blockAt(...hit.pos);
            (box.material as THREE.LineBasicMaterial).color.set(
              INTERACTIVE.test(state) ? 0x6fd66a : 0x000000,
            );
          }
        }
        setTarget(hit ? `${world.blockAt(...hit.pos)}  @ ${hit.pos.join(", ")}` : "");
      }

      // Fade the click flash out.
      const flash = flashRef.current;
      if (flash?.visible) {
        const left = flashUntil.current - now;
        if (left <= 0) flash.visible = false;
        else (flash.material as THREE.MeshBasicMaterial).opacity = (left / 350) * 0.45;
      }
      renderer.render(scene, camera);
    };
    raf = requestAnimationFrame(frame);

    // Mouse verbs, as the game assigns them: left breaks, right uses.
    // Breaking is `placeBlock(air)` — the engine's own way to remove a
    // block, so supports drop, observers fire and the machine reacts
    // exactly as it would to a player mining it.
    const act = (event: MouseEvent) => {
      const world = worldRef.current;
      const p = playerRef.current;
      if (!world?.sim || !p || !p.pointerLocked) return;
      event.preventDefault();
      const hit = p.pick((x, y, z) => world.isSolid(x, y, z));
      if (!hit) return;
      const before = world.blockAt(...hit.pos);
      const short = (t: string) => t.replace("minecraft:", "");
      const breaking = event.button === 0;
      try {
        if (breaking) world.sim.placeBlock(hit.pos[0], hit.pos[1], hit.pos[2], "minecraft:air");
        else world.sim.useBlock(...hit.pos);
        const changes = world.drainChanges();
        world.applyChanges(changes);
        const after = world.blockAt(...hit.pos);
        setInfo(
          breaking
            ? `broke ${short(before)} @ ${hit.pos.join(",")}  (${changes.length} change${changes.length === 1 ? "" : "s"})`
            : after !== before
              ? `${short(before)} → ${short(after)}  (${changes.length} change${changes.length === 1 ? "" : "s"})`
              : `${short(before)} — no change (nothing to activate)`,
        );
        const flash = flashRef.current;
        if (flash) {
          flash.position.set(hit.pos[0], hit.pos[1], hit.pos[2]);
          (flash.material as THREE.MeshBasicMaterial).color.set(
            breaking ? 0xff6b6b : after !== before ? 0x6fd66a : 0xe8c14a,
          );
          flash.visible = true;
          flashUntil.current = performance.now() + 350;
        }
      } catch (e) {
        setInfo(`${breaking ? "cannot break" : "cannot use"} that block: ${e}`);
      }
    };
    const noMenu = (e: Event) => {
      if (playerRef.current?.pointerLocked) e.preventDefault();
    };
    renderer.domElement.addEventListener("mousedown", act);
    renderer.domElement.addEventListener("contextmenu", noMenu);

    return () => {
      cancelAnimationFrame(raf);
      observer.disconnect();
      renderer.domElement.removeEventListener("mousedown", act);
      renderer.domElement.removeEventListener("contextmenu", noMenu);
      renderer.dispose();
      host.removeChild(renderer.domElement);
    };
  }, []);

  const open = useCallback(async (file: File, mode: SettleName = settle, chunk: number = chunkSize) => {
    setStatus({ kind: "loading", message: `meshing ${file.name}…` });
    setRunning(false);
    setRecording(false);
    setActivity(null);
    try {
      const bytes = new Uint8Array(await file.arrayBuffer());
      const world = await World.load(bytes, mode, chunk);
      lastFile.current = file;
      // Swap the scene contents: the old build leaves, the new one arrives.
      const previous = worldRef.current;
      if (previous) sceneRef.current?.remove(previous.group);
      worldRef.current = world;
      sceneRef.current?.add(world.group);
      playerRef.current?.frame(world.dims);
      // A handle for the console and for headless tests: the same objects
      // the UI drives, so a scripted check exercises the real path.
      (window as unknown as { simlab: unknown }).simlab = {
        world,
        loader,
        player: playerRef.current,
        scene: sceneRef.current,
      };
      const failure = world.sim ? null : world.startSim();
      setStatus(
        failure
          ? { kind: "error", message: failure }
          : {
              kind: "ready",
              message: `${file.name} — ${world.dims.join("×")}`,
            },
      );
      setTick(0);
      setEntities(world.entityCount());
    } catch (e) {
      setStatus({ kind: "error", message: String(e) });
    }
  }, [settle, chunkSize]);

  const onDrop = useCallback(
    (e: React.DragEvent) => {
      e.preventDefault();
      const file = e.dataTransfer.files?.[0];
      if (file) void open(file);
    },
    [open],
  );

  /** Advance exactly `n` ticks, collecting every change so the re-mesh is
   * one pass however far you jumped. */
  const stepBy = useCallback((n: number) => {
    const world = worldRef.current;
    if (!world?.sim) return;
    const t0 = performance.now();
    for (let i = 0; i < n; i++) world.sim.step();
    const changes = world.drainChanges();
    world.applyChanges(changes);
    const ms = performance.now() - t0;
    setTick(Number(world.sim.tickCount?.() ?? 0));
    setEntities(world.entityCount());
    setInfo(
      `stepped ${n} tick${n === 1 ? "" : "s"} — ${changes.length} block change${
        changes.length === 1 ? "" : "s"
      } in ${ms.toFixed(1)}ms${world.sim.isQuiescent?.() ? " · quiescent" : ""}`,
    );
  }, []);

  /** Toggle the run timeline: `record` when idle, `stop` when recording. The
   * button follows `World`'s own flag rather than driving one of its own, so
   * a world reload (which resets it in `clearFlights`) cannot leave the
   * label out of step with what the engine is actually doing. */
  const toggleRecording = useCallback(() => {
    const world = worldRef.current;
    if (!world?.sim) return;
    if (world.isRecording()) {
      world.stopRecording();
      setRecording(world.isRecording());
      // One more read past the polling timer's last tick, so the strip's
      // final frame reflects everything up to the exact tick recording
      // stopped on rather than whatever the last 250 ms tick caught.
      pollActivity();
    } else {
      world.startRecording();
      setActivity(null);
      setRecording(world.isRecording());
    }
  }, [pollActivity]);

  return (
    <div className="app" onDrop={onDrop} onDragOver={(e) => e.preventDefault()}>
      <div className="viewport" ref={mount} />
      <div className="crosshair" aria-hidden>
        +
      </div>

      <header className="bar">
        <strong>nucleation · sim lab</strong>
        <label className="file">
          open…
          <input
            type="file"
            accept=".litematic,.schem,.schematic,.nbt,.snbt,.mcstructure"
            onChange={(e) => {
              const f = e.target.files?.[0];
              if (f) void open(f);
            }}
          />
        </label>
        <button onClick={() => setRunning((r) => !r)} disabled={status.kind !== "ready"}>
          {running ? "pause" : "run"}
        </button>
        <button onClick={toggleRecording} disabled={status.kind !== "ready"}>
          {recording ? "stop" : "record"}
        </button>
        <span className="steps">
          <button onClick={() => stepBy(1)} disabled={status.kind !== "ready" || running}>
            step
          </button>
          {[10, 100].map((n) => (
            <button
              key={n}
              onClick={() => stepBy(n)}
              disabled={status.kind !== "ready" || running}
            >
              +{n}
            </button>
          ))}
          <input
            className="stepn"
            type="number"
            min={1}
            max={100000}
            value={stepN}
            onChange={(e) => setStepN(Math.max(1, Number(e.target.value) || 1))}
            title="how many ticks the › button advances"
          />
          <button onClick={() => stepBy(stepN)} disabled={status.kind !== "ready" || running}>
            ›
          </button>
        </span>
        <label className="settle" title="how the build is settled before tick 0">
          <select
            value={settle}
            onChange={(e) => {
              const mode = e.target.value as SettleName;
              setSettle(mode);
              setRunning(false);
              if (lastFile.current) void open(lastFile.current, mode);
            }}
          >
            <option value="in-world">as it stood (at rest)</option>
            <option value="placement">as if pasted (observers pulse)</option>
            <option value="quiet">quiet (onPlace only)</option>
          </select>
        </label>
        <label
          className="settle"
          title="mesh chunk size — smaller re-meshes less geometry per change; one chunk gives the build a single texture atlas, which never has to be decoded twice"
        >
          <select
            value={chunkSize}
            onChange={(e) => {
              const size = Number(e.target.value);
              setChunkSize(size);
              setRunning(false);
              if (lastFile.current) void open(lastFile.current, settle, size);
            }}
          >
            <option value={16}>chunks 16³</option>
            <option value={32}>chunks 32³</option>
            <option value={64}>chunks 64³</option>
            <option value={0}>one chunk (whole build)</option>
          </select>
        </label>
        <label className="rate">
          {fmtTps(rate)} tps
          <input
            type="range"
            min={0}
            max={SLIDER_MAX}
            value={slider}
            onChange={(e) => setSlider(Number(e.target.value))}
          />
        </label>
        {/* Target and achieved are separate readouts on purpose: when they
            diverge, that gap is the answer to how fast this build can go. */}
        <span className={`tick rate-actual${throttled ? " throttled" : ""}`}>
          {fmtTps(actualTps)} actual
          {throttled ? " · throttled" : ""}
        </span>
        <span className="tick">tick {tick}</span>
        {entities > 0 && <span className="tick">{entities} entities</span>}
        <span className={`status ${status.kind}`}>{status.message ?? "drop a schematic"}</span>
      </header>

      <TimelineStrip activity={activity} />

      <footer className="hud">
        <span className="target">{target || "—"}</span>
        <span className="hint">
          {locked
            ? "wasd fly · space/shift up-down · ctrl sprint · left-click breaks · right-click uses · esc release"
            : "click the view to fly"}
        </span>
        {info && <span className="info">{info}</span>}
      </footer>
    </div>
  );
}
