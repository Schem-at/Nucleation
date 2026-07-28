/** The flight stage — the lab's signature element. Renders one detected
 * period of the champion's flight on a canvas, translate-compensated so the
 * machine flies in place while the corridor floor scrolls underneath.
 * Seamless by construction: frame t0+period is frame t0 shifted +dx, and the
 * camera shift advances exactly dx per cycle. */

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { downloadBlob, exportLoopGif } from "../gif";
import { drawScene, type CanvasPalette } from "../iso";
import { onTexturesChanged } from "../textures";
import type { BestRecord } from "../types";

const TPS = 10; // playback ticks per second (engine flies 1 block / ~10 ticks)
const CSS_H = 320;

function readPalette(): CanvasPalette {
  const cs = getComputedStyle(document.documentElement);
  return {
    page: cs.getPropertyValue("--page").trim() || "#f4f4f1",
    grid: cs.getPropertyValue("--grid-iso").trim() || "rgba(0,0,0,0.06)",
    edge: "rgba(0,0,0,0.25)",
  };
}

interface Props {
  best: BestRecord | null;
  /** True when `best` is the reigning champion (vs a filmstrip pick). */
  isChampion: boolean;
  loading: boolean;
}

export default function FlightLoop({ best, isChampion, loading }: Props) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const wrapRef = useRef<HTMLDivElement>(null);
  const [exporting, setExporting] = useState(false);
  const loop = best?.loop ?? null;

  /** Scene geometry in compensated space (machine ~static around x=0). */
  const geom = useMemo(() => {
    if (!loop || loop.frames.length === 0) return null;
    let minX = Infinity,
      maxX = -Infinity,
      minY = Infinity,
      maxY = -Infinity,
      minZ = Infinity,
      maxZ = -Infinity;
    const period = Math.max(loop.period, 1);
    loop.frames.forEach((f, t) => {
      const shift = loop.anchorX + (t / period) * loop.dx;
      for (const b of f.blocks) {
        minX = Math.min(minX, b.x - shift);
        maxX = Math.max(maxX, b.x - shift + 1);
        minY = Math.min(minY, b.y);
        maxY = Math.max(maxY, b.y + 1);
        minZ = Math.min(minZ, b.z);
        maxZ = Math.max(maxZ, b.z + 1);
      }
    });
    if (!Number.isFinite(minX)) return null;
    const focus: [number, number, number] = [
      (minX + maxX) / 2,
      (minY + maxY) / 2 - 0.5,
      (minZ + maxZ) / 2,
    ];
    const floor = {
      minX: minX - 5,
      maxX: maxX + 6,
      minZ: minZ - 2,
      maxZ: maxZ + 3,
      y: Math.min(minY, 0),
    };
    return { focus, floor };
  }, [loop]);

  // rAF playback.
  useEffect(() => {
    const canvas = canvasRef.current;
    const wrap = wrapRef.current;
    if (!canvas || !wrap || !loop || !geom) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const reduced = matchMedia("(prefers-reduced-motion: reduce)").matches;
    let raf = 0;
    const t0 = performance.now();

    const render = (now: number) => {
      const cssW = wrap.clientWidth;
      const dpr = window.devicePixelRatio || 1;
      if (canvas.width !== Math.round(cssW * dpr)) {
        canvas.width = Math.round(cssW * dpr);
        canvas.height = Math.round(CSS_H * dpr);
      }
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);

      const period = Math.max(loop.period, 1);
      const phase =
        loop.period > 0 && !reduced
          ? ((((now - t0) / 1000) * TPS) % period) / period
          : 0;
      const tick = Math.min(Math.floor(phase * period), loop.frames.length - 1);
      const shift = loop.anchorX + phase * loop.dx;
      drawScene(
        ctx,
        cssW,
        CSS_H,
        loop.frames[tick].blocks,
        shift,
        readPalette(),
        26,
        geom.floor,
        geom.focus,
        loop.cast ?? null,
        phase * period,
      );
      if (loop.period > 0 && !reduced) raf = requestAnimationFrame(render);
    };
    raf = requestAnimationFrame(render);
    // Static scenes (reduced motion / no period) don't loop, so repaint once
    // whenever a block texture finishes decoding; the rAF loop covers the rest.
    const unsub = onTexturesChanged(() => {
      if (reduced || loop.period === 0) raf = requestAnimationFrame(render);
    });
    return () => {
      unsub();
      cancelAnimationFrame(raf);
    };
  }, [loop, geom]);

  const onExport = useCallback(() => {
    if (!loop || !geom || !best) return;
    setExporting(true);
    // Let the button repaint before the synchronous encode.
    setTimeout(() => {
      try {
        const blob = exportLoopGif(loop, readPalette(), geom.focus, geom.floor);
        downloadBlob(blob, `flight-gen${best.gen}-${best.fitness.toFixed(1)}.gif`);
      } finally {
        setExporting(false);
      }
    }, 30);
  }, [loop, geom, best]);

  if (!best) {
    return (
      <div className="viewer-empty stage-empty">
        The first champion's flight appears here once a machine flies.
      </div>
    );
  }

  return (
    <div className="stage">
      <div className="stage-meta">
        <div className="big">
          {best.fitness.toFixed(1)}
          <span className="unit">blocks flown</span>
        </div>
        <div className="kv">
          champion of<b>gen {best.gen}</b>
        </div>
        <div className="kv">
          period
          <b>{loop && loop.period > 0 ? `${loop.period} ticks / +${loop.dx}x` : "—"}</b>
        </div>
        <div className="spacer" />
        <button
          className="icon-btn"
          onClick={onExport}
          disabled={!loop || loop.period === 0 || exporting}
          data-testid="export-gif"
        >
          {exporting ? "encoding…" : "Export GIF"}
        </button>
      </div>

      <div className="stage-canvas" ref={wrapRef}>
        <canvas
          ref={canvasRef}
          style={{ width: "100%", height: CSS_H }}
          role="img"
          aria-label={`Looping flight of the generation ${best.gen} champion`}
        />
        {(loading || !loop) && (
          <div className="stage-overlay">
            {loading ? "re-simulating flight…" : "no replay yet"}
          </div>
        )}
        {!isChampion && <div className="stage-tag">filmstrip pick</div>}
      </div>

      {loop && (
        <p className="stage-note">
          {loop.period > 0
            ? `loop = 1 period, detected via ${loop.method}`
            : "machine did not settle into a periodic gait — showing final state"}
        </p>
      )}
    </div>
  );
}
