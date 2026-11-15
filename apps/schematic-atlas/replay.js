/** Playback of actual retained tick-end records. This does not run the engine. */
export function mountReplay(host, capture) {
  const frames = [capture.initial, ...(capture.snapshots || [])];
  const section = document.createElement('section');
  section.className = 'atlas-replay';
  section.innerHTML = `<style>
  .atlas-replay{background:#1e2823;color:#ebefe5;border-radius:16px;overflow:hidden;font:13px system-ui}
  .replay-toolbar,.replay-transport{display:flex;align-items:center;gap:10px;flex-wrap:wrap;padding:14px 18px}
  .replay-toolbar{border-bottom:1px solid #ffffff19}.replay-toolbar small{color:#b9c6b6;margin-left:auto}
  .atlas-replay button,.atlas-replay select{background:#334039;color:#eff4e9;border:1px solid #68766b;border-radius:7px;padding:8px;cursor:pointer;font:inherit}
  .atlas-replay button:focus-visible,.atlas-replay input:focus-visible{outline:3px solid #d2ff5a;outline-offset:2px}
  .atlas-replay canvas{width:100%;height:370px;display:block;touch-action:none;cursor:grab}
  .replay-transport input{flex:1;min-width:120px;accent-color:#d2ff5a}.replay-stats{padding:0 18px 14px;display:flex;gap:20px;color:#d6dfce;flex-wrap:wrap}
  .replay-block{padding:12px 18px;background:#16201a;font:11px ui-monospace,monospace;min-height:36px;overflow-wrap:anywhere}
  .replay-records{display:grid;grid-template-columns:1fr 1fr;gap:18px;padding:16px 18px}.replay-records pre{white-space:pre-wrap;font-size:11px;max-height:130px;overflow:auto;color:#b9c6b6}
  @media(max-width:600px){.atlas-replay canvas{height:290px}.replay-records{grid-template-columns:1fr}}
  </style><div class="replay-toolbar"><strong>Recorded tick playback</strong><small>Minecraft 1.21.11 · not a live run</small>
  <select aria-label="Replay view"><option value="iso">Isometric</option><option value="top">Top view</option></select></div>
  <canvas aria-label="Recorded block-state projection. Drag to rotate; scroll to zoom."></canvas>
  <div class="replay-transport"><button type="button" data-action="prev" aria-label="Previous tick">←</button><button type="button" data-action="play">Play</button><button type="button" data-action="next" aria-label="Next tick">→</button><input type="range" min="0" max="${frames.length - 1}" value="0" aria-label="Captured tick"><output></output><select aria-label="Replay speed"><option value="5">5 TPS</option><option value="20" selected>20 TPS</option><option value="60">60 TPS</option></select></div>
  <div class="replay-stats"></div><div class="replay-block">Drag to orbit · scroll to zoom · hover a block to inspect its saved state</div>
  <div class="replay-records"><div><strong>Comparator outputs</strong><pre class="replay-comparators"></pre></div><div><strong>Occupied inventory slots</strong><pre class="replay-inventory"></pre></div></div>`;
  host.replaceChildren(section);
  const canvas = section.querySelector('canvas'), ctx = canvas.getContext('2d');
  const seek = section.querySelector('input'), output = section.querySelector('output');
  const play = section.querySelector('[data-action="play"]'), selects = section.querySelectorAll('select');
  let frame = 0, playing = false, angle = -.7, zoom = 1, last = 0, raf = 0, disposed = false, drag = null, projected = [];
  const positions = capture.fixture.map(b => b.pos);
  const min = [0, 1, 2].map(a => Math.min(...positions.map(p => p[a]))), max = [0, 1, 2].map(a => Math.max(...positions.map(p => p[a])));
  const center = min.map((v, i) => (v + max[i]) / 2);
  function palette(state) {
    if (state.includes('redstone_lamp') || state.includes('copper_bulb')) return state.includes('lit=true') ? '#ffe280' : '#766651';
    if (state.includes('redstone_wire')) return /power=0[,\]]/.test(state) ? '#6a3833' : '#fa6850';
    if (/piston|slime/.test(state)) return '#b9d975';
    if (/comparator|repeater|observer/.test(state)) return '#e18472';
    if (/hopper|chest|dropper|dispenser|crafter/.test(state)) return '#739eaf';
    if (/lever|button|torch/.test(state)) return /powered=true|lit=true/.test(state) ? '#ffcd67' : '#a59770';
    if (/copper/.test(state)) return '#6fa397';
    if (/glass/.test(state)) return '#95b3b0';
    if (/air/.test(state)) return 'transparent';
    return '#839087';
  }
  function update() {
    const current = frames[frame];
    seek.value = frame; output.textContent = `Tick ${current.tick} / ${frames.at(-1).tick}`;
    let changed = 0;
    if (frame) {
      const prior = new Map(frames[frame - 1].blocks.map(b => [b.slice(0, 3).join(','), b[3]]));
      for (const b of current.blocks) { const key = b.slice(0, 3).join(','); if (prior.get(key) !== b[3]) changed++; prior.delete(key); }
      changed += prior.size;
    }
    section.querySelector('.replay-stats').textContent = `${current.blocks.length} non-air blocks  ·  ${changed} changed cells this boundary  ·  ${current.players} players`;
    section.querySelector('.replay-comparators').textContent = current.comparator_outputs.map(r => `(${r.slice(0, 3).join(', ')}) → ${r[3]}`).join('\n') || 'No comparator outputs';
    section.querySelector('.replay-inventory').textContent = current.inventories.map(r => `(${r.slice(0, 3).join(', ')}) slot ${r[3]}: ${r[4]} × ${r[5]}`).join('\n') || 'No occupied slots';
    draw();
  }
  function draw() {
    const rect = canvas.getBoundingClientRect(), width = Math.max(1, rect.width), height = Math.max(1, rect.height), dpr = Math.min(2, devicePixelRatio || 1);
    if (canvas.width !== Math.round(width * dpr) || canvas.height !== Math.round(height * dpr)) { canvas.width = Math.round(width * dpr); canvas.height = Math.round(height * dpr); }
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0); ctx.clearRect(0, 0, width, height);
    const top = selects[0].value === 'top';
    const span = Math.max(max[0] - min[0] + max[2] - min[2] + 4, max[1] - min[1] + 4);
    const scale = Math.min(width / (span * 1.1), height / (span * .7)) * zoom;
    const co = Math.cos(angle), si = Math.sin(angle);
    projected = frames[frame].blocks.map(b => {
      const x = b[0] - center[0], y = b[1] - center[1], z = b[2] - center[2];
      const rx = x * co - z * si, rz = x * si + z * co;
      return {b, x: width / 2 + rx * scale, y: height / 2 + (top ? rz : rz * .45 - y * .78) * scale, depth: top ? y : rz + y * .1};
    }).sort((a, b) => a.depth - b.depth);
    for (const p of projected) {
      const s = scale * .46; ctx.fillStyle = palette(p.b[3]); ctx.strokeStyle = '#17231e99'; ctx.lineWidth = .7;
      if (top) { ctx.fillRect(p.x - s, p.y - s, s * 2, s * 2); ctx.strokeRect(p.x - s, p.y - s, s * 2, s * 2); }
      else {
        ctx.beginPath(); ctx.moveTo(p.x, p.y - s * .5); ctx.lineTo(p.x + s, p.y); ctx.lineTo(p.x + s, p.y + s * .8); ctx.lineTo(p.x, p.y + s * 1.3); ctx.lineTo(p.x - s, p.y + s * .8); ctx.lineTo(p.x - s, p.y); ctx.closePath(); ctx.fill(); ctx.stroke();
        ctx.strokeStyle = '#ffffff25'; ctx.beginPath(); ctx.moveTo(p.x - s, p.y); ctx.lineTo(p.x, p.y + s * .5); ctx.lineTo(p.x + s, p.y); ctx.moveTo(p.x, p.y + s * .5); ctx.lineTo(p.x, p.y + s * 1.3); ctx.stroke();
      }
    }
    ctx.fillStyle = '#a5b6a5'; ctx.font = '10px ui-monospace,monospace'; ctx.fillText('BLOCK-STATE PROJECTION · COLORS BY MECHANIC', 16, height - 14);
  }
  function setPlaying(value) { playing = value; play.textContent = value ? 'Pause' : 'Play'; last = performance.now(); }
  play.onclick = () => { if (frame === frames.length - 1) { frame = 0; update(); } setPlaying(!playing); };
  section.querySelector('[data-action="prev"]').onclick = () => { setPlaying(false); frame = Math.max(0, frame - 1); update(); };
  section.querySelector('[data-action="next"]').onclick = () => { setPlaying(false); frame = Math.min(frames.length - 1, frame + 1); update(); };
  seek.oninput = () => { setPlaying(false); frame = +seek.value; update(); };
  selects[0].onchange = draw;
  canvas.onpointerdown = e => { drag = e.clientX; canvas.setPointerCapture(e.pointerId); };
  canvas.onpointerup = () => { drag = null; };
  canvas.onpointercancel = () => { drag = null; };
  canvas.onpointermove = e => {
    if (drag !== null) { angle += (e.clientX - drag) / 130; drag = e.clientX; draw(); return; }
    const box = canvas.getBoundingClientRect(), x = e.clientX - box.left, y = e.clientY - box.top;
    let closest = null, distance = 18;
    for (const p of projected) { const d = Math.hypot(p.x - x, p.y - y); if (d < distance) { closest = p; distance = d; } }
    if (closest) section.querySelector('.replay-block').textContent = `[${closest.b.slice(0, 3).join(', ')}] ${closest.b[3]}`;
  };
  canvas.addEventListener('wheel', e => { e.preventDefault(); zoom = Math.max(.4, Math.min(4, zoom * Math.exp(-e.deltaY * .001))); draw(); }, {passive: false});
  const observer = new ResizeObserver(draw); observer.observe(canvas);
  function animate(now) { if (disposed) return; if (playing && now - last >= 1000 / +selects[1].value) { frame = Math.min(frames.length - 1, frame + 1); last = now; update(); if (frame === frames.length - 1) setPlaying(false); } raf = requestAnimationFrame(animate); }
  update(); raf = requestAnimationFrame(animate);
  return () => { disposed = true; cancelAnimationFrame(raf); observer.disconnect(); host.replaceChildren(); };
}
