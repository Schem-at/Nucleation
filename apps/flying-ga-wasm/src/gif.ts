/** Client-side GIF export of one seamless flight-loop period via gifenc. */

import { GIFEncoder, quantize, applyPalette } from "gifenc";
import { drawScene, type CanvasPalette } from "./iso";
import type { FlightLoopData } from "./types";

const W = 480;
const H = 300;
const TPS = 10; // playback ticks/sec, matches the live loop

export function exportLoopGif(
  loop: FlightLoopData,
  pal: CanvasPalette,
  focus: [number, number, number],
  floor: { minX: number; maxX: number; minZ: number; maxZ: number; y: number },
): Blob {
  const period = Math.max(loop.period, 1);
  // 2 subframes per tick keeps the compensated scroll smooth.
  const sub = period <= 24 ? 2 : 1;
  const nFrames = period * sub;
  const delayMs = 1000 / (TPS * sub);

  const canvas = document.createElement("canvas");
  canvas.width = W;
  canvas.height = H;
  const ctx = canvas.getContext("2d", { willReadFrequently: true })!;

  const gif = GIFEncoder();
  for (let k = 0; k < nFrames; k++) {
    const phase = k / nFrames;
    const tick = Math.floor(phase * period);
    const shift = loop.anchorX + phase * loop.dx;
    drawScene(
      ctx,
      W,
      H,
      loop.frames[tick].blocks,
      shift,
      pal,
      22,
      floor,
      focus,
      loop.cast ?? null,
      phase * period,
    );
    const { data } = ctx.getImageData(0, 0, W, H);
    const palette = quantize(data, 256);
    const index = applyPalette(data, palette);
    gif.writeFrame(index, W, H, {
      palette,
      delay: Math.round(delayMs),
      repeat: 0,
    });
  }
  gif.finish();
  const bytes = gif.bytes();
  const copy = new Uint8Array(bytes.length);
  copy.set(bytes);
  return new Blob([copy.buffer as ArrayBuffer], { type: "image/gif" });
}

export function downloadBlob(blob: Blob, filename: string): void {
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  a.click();
  setTimeout(() => URL.revokeObjectURL(url), 5000);
}
