/** Static WebGL viewer for a block list: every state meshed by nucleation's
 * real mesher (see ../mesher.ts), instanced per block, OrbitControls with an
 * auto-fit isometric initial framing. Used by MachineViewer; the filmstrip
 * keeps the lightweight IsoThumb.
 *
 * Camera contract (round 3): the renderer, camera and controls are created
 * ONCE per component life (not per data refresh — the old effect keyed on an
 * inline callback prop and rebuilt everything every poll, resetting the
 * camera). Blocks are swapped INTO the persistent scene when their content
 * signature changes; the camera auto-fits only then — and never again once
 * the user has touched the controls (`onUserInteract` fires so the parent
 * can drop auto-follow too). Bumping `fitNonce` re-arms auto-fit. */

import { useEffect, useMemo, useRef, useState } from "react";
import * as THREE from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";
import { addDefaultLights, fitCamera, instanceFor } from "../mesher";
import type { Block } from "../types";

/** Debug registry so verification can assert the camera never moves on
 * polls: window.__fgaCameras[debugId] -> THREE.PerspectiveCamera. */
declare global {
  interface Window {
    __fgaCameras?: Record<string, THREE.PerspectiveCamera>;
  }
}

interface Stage {
  renderer: THREE.WebGLRenderer;
  camera: THREE.PerspectiveCamera;
  controls: OrbitControls;
  scene: THREE.Scene;
  content: THREE.Group;
}

export default function GLBlocks({
  blocks,
  height = 260,
  label,
  onFail,
  onUserInteract,
  fitNonce = 0,
  debugId,
}: {
  blocks: Block[];
  height?: number;
  label: string;
  onFail?: () => void;
  /** Fired once per grab when the user touches the orbit controls. */
  onUserInteract?: () => void;
  /** Bump to re-arm (and immediately run) camera auto-fit. */
  fitNonce?: number;
  /** Key in window.__fgaCameras for test introspection. */
  debugId?: string;
}) {
  const mountRef = useRef<HTMLDivElement>(null);
  const [ready, setReady] = useState(false);
  // Re-run the effect after a WebGL context loss (Chrome LRU-kills the
  // oldest context past its per-page cap).
  const [nonce, setNonce] = useState(0);
  const stageRef = useRef<Stage | null>(null);
  const interactedRef = useRef(false);
  const boxRef = useRef<THREE.Box3 | null>(null);
  const blocksRef = useRef(blocks);
  blocksRef.current = blocks;
  // Callback props live in refs so their identity never re-runs effects
  // (the old code keyed the GL effect on an inline onFail — one rebuild,
  // camera re-fit included, per leaderboard poll).
  const onFailRef = useRef(onFail);
  onFailRef.current = onFail;
  const onUserInteractRef = useRef(onUserInteract);
  onUserInteractRef.current = onUserInteract;
  // Leaderboard refreshes hand us a NEW array with identical content every
  // generation; keying the content swap on the signature keeps the scene
  // (and camera) untouched by polls.
  const sig = useMemo(
    () => blocks.map((b) => `${b.x},${b.y},${b.z},${b.state}`).join(";"),
    [blocks],
  );

  // Mount once: renderer + camera + controls + render loop.
  useEffect(() => {
    const mount = mountRef.current;
    if (!mount) return;

    let renderer: THREE.WebGLRenderer;
    try {
      renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true });
    } catch {
      onFailRef.current?.();
      return;
    }
    renderer.setClearColor(0x000000, 0);
    renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, 2));
    renderer.setSize(mount.clientWidth, height);
    mount.appendChild(renderer.domElement);
    const onLost = (e: Event) => {
      e.preventDefault();
      console.warn("[gl] viewer context lost — recreating");
      setNonce((n) => n + 1);
    };
    renderer.domElement.addEventListener("webglcontextlost", onLost);

    const scene = new THREE.Scene();
    addDefaultLights(scene);
    const content = new THREE.Group();
    scene.add(content);
    const camera = new THREE.PerspectiveCamera(
      36,
      mount.clientWidth / height,
      0.1,
      1000,
    );
    const controls = new OrbitControls(camera, renderer.domElement);
    controls.enableDamping = true;
    controls.dampingFactor = 0.1;
    const onStart = () => {
      if (!interactedRef.current) {
        interactedRef.current = true;
        onUserInteractRef.current?.();
      }
    };
    controls.addEventListener("start", onStart);

    if (debugId) {
      window.__fgaCameras = window.__fgaCameras ?? {};
      window.__fgaCameras[debugId] = camera;
    }

    stageRef.current = { renderer, camera, controls, scene, content };

    let raf = 0;
    const render = () => {
      raf = requestAnimationFrame(render);
      controls.update();
      renderer.render(scene, camera);
    };
    raf = requestAnimationFrame(render);

    const ro = new ResizeObserver(() => {
      const w = mount.clientWidth;
      renderer.setSize(w, height);
      camera.aspect = w / height;
      camera.updateProjectionMatrix();
    });
    ro.observe(mount);

    return () => {
      stageRef.current = null;
      if (debugId && window.__fgaCameras) delete window.__fgaCameras[debugId];
      ro.disconnect();
      cancelAnimationFrame(raf);
      controls.removeEventListener("start", onStart);
      controls.dispose();
      renderer.domElement.removeEventListener("webglcontextlost", onLost);
      renderer.dispose();
      renderer.forceContextLoss();
      mount.removeChild(renderer.domElement);
    };
  }, [height, nonce, debugId]);

  // Content swap: only when the DISPLAYED machine actually changes.
  useEffect(() => {
    const stage = stageRef.current;
    const blocks = blocksRef.current;
    if (!stage || blocks.length === 0) return;
    const { content, camera, controls } = stage;

    content.clear();
    setReady(false);

    const box = new THREE.Box3();
    for (const b of blocks) {
      box.expandByPoint(new THREE.Vector3(b.x, b.y, b.z));
      box.expandByPoint(new THREE.Vector3(b.x + 1, b.y + 1, b.z + 1));
    }
    boxRef.current = box;
    if (!interactedRef.current) {
      const center = fitCamera(camera, box);
      controls.target.copy(center);
      controls.update();
    }

    let disposed = false;
    let pending = blocks.length;
    for (const b of blocks) {
      instanceFor(b.state)
        .then((inst) => {
          if (disposed) return;
          inst.position.set(b.x, b.y, b.z);
          content.add(inst);
        })
        .catch((e) => console.warn("mesh failed for", b.state, e))
        .finally(() => {
          pending -= 1;
          if (pending === 0 && !disposed) setReady(true);
        });
    }
    return () => {
      disposed = true;
    };
  }, [sig, nonce]);

  // Re-arm + run auto-fit on demand (the "following ⏸" chip).
  useEffect(() => {
    if (fitNonce === 0) return;
    interactedRef.current = false;
    const stage = stageRef.current;
    const box = boxRef.current;
    if (stage && box) {
      const center = fitCamera(stage.camera, box);
      stage.controls.target.copy(center);
      stage.controls.update();
    }
  }, [fitNonce]);

  return (
    <div
      ref={mountRef}
      style={{ width: "100%", height, position: "relative" }}
      role="img"
      aria-label={label}
      data-testid="gl-blocks"
      data-ready={ready ? "1" : "0"}
    />
  );
}
