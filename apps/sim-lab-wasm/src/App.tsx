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
import { World, type SettleName } from "./world";
import { Player } from "./player";
import { addDefaultLights } from "./mesher";

type Status = { kind: "idle" | "loading" | "ready" | "error"; message?: string };

const INTERACTIVE = /lever|button|note_block|trapdoor|_door|_gate|repeater|comparator|daylight/;

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
  const [rate, setRate] = useState(10);
  const [stepN, setStepN] = useState(20);
  const [settle, setSettle] = useState<SettleName>("in-world");
  const [tick, setTick] = useState(0);
  const [target, setTarget] = useState<string>("");
  const [info, setInfo] = useState<string>("");
  const [locked, setLocked] = useState(false);

  useEffect(() => {
    runningRef.current = running;
  }, [running]);
  useEffect(() => {
    rateRef.current = rate;
  }, [rate]);

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
    const frame = (now: number) => {
      raf = requestAnimationFrame(frame);
      const dt = Math.min((now - last) / 1000, 0.1);
      last = now;
      player.update(dt);
      setLocked(player.pointerLocked);

      const world = worldRef.current;
      if (world?.sim) {
        if (runningRef.current) {
          carry += dt * rateRef.current;
          let steps = Math.floor(carry);
          carry -= steps;
          steps = Math.min(steps, 200); // never freeze the tab on a big rate
          for (let i = 0; i < steps; i++) world.sim.step();
          // One drain for the whole batch: the log is cumulative, so
          // reading it per step is quadratic.
          if (steps) world.applyChanges(world.drainChanges());
          if (steps) setTick(Number(world.sim.tickCount?.() ?? 0));
        }
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

  const open = useCallback(async (file: File, mode: SettleName = settle) => {
    setStatus({ kind: "loading", message: `meshing ${file.name}…` });
    setRunning(false);
    try {
      const bytes = new Uint8Array(await file.arrayBuffer());
      const world = await World.load(bytes, mode);
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
    } catch (e) {
      setStatus({ kind: "error", message: String(e) });
    }
  }, [settle]);

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
    setInfo(
      `stepped ${n} tick${n === 1 ? "" : "s"} — ${changes.length} block change${
        changes.length === 1 ? "" : "s"
      } in ${ms.toFixed(1)}ms${world.sim.isQuiescent?.() ? " · quiescent" : ""}`,
    );
  }, []);

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
        <label className="rate">
          {rate} tps
          <input
            type="range"
            min={1}
            max={60}
            value={rate}
            onChange={(e) => setRate(Number(e.target.value))}
          />
        </label>
        <span className="tick">tick {tick}</span>
        <span className={`status ${status.kind}`}>{status.message ?? "drop a schematic"}</span>
      </header>

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
