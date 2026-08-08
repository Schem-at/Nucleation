/** Boot + UI wiring: the studio state, the canvas, and the side panels. */
import { loadEngine } from "./engine";
import { Studio, type Vec3, type PortInfo } from "./studio";
import { Viewer, type PortMarker, type GateMarker, type InstanceMarker } from "./viewer";
import { verilogToBlif, guessTop } from "./yosys";

const $ = <T extends HTMLElement = HTMLElement>(sel: string) =>
  document.querySelector(sel) as T;

const log = (msg: string) => {
  const el = $("#log");
  el.textContent = `${msg}\n${el.textContent ?? ""}`.slice(0, 8000);
  console.log(`[eda] ${msg}`);
};
const toast = (() => {
  let timer = 0;
  return (msg: string) => {
    const el = $("#toast");
    el.textContent = msg;
    el.style.display = "block";
    clearTimeout(timer);
    timer = window.setTimeout(() => (el.style.display = "none"), 2600);
  };
})();

const DEMO_VERILOG = `module add2(input [1:0] a, input [1:0] b, output [2:0] y);
  assign y = a + b;
endmodule
`;

async function boot() {
  const { core, d } = await loadEngine();
  $("#status").textContent = "engine ready";
  (window as any).__edaCore = core;

  let studio = new Studio(core, d, "studio");
  let pendingPort: string | null = null;
  let placingCell: string | null = null;
  let lastLive = 0;

  // ---- rendering --------------------------------------------------------

  const viewer = new Viewer($("#canvas-wrap"), {
    onPortClick(name) {
      const port = studio.ports.get(name);
      if (!port) return;
      if (!pendingPort) {
        pendingPort = name;
        toast(`port ${name} selected — click a second port to route`);
        return;
      }
      if (pendingPort === name) {
        pendingPort = null;
        return;
      }
      connectPorts(pendingPort, name);
      pendingPort = null;
    },
    onDragMove(kind, id, ground) {
      if (!($("#live-reroute") as HTMLInputElement).checked) return;
      const now = performance.now();
      if (now - lastLive < 250) return; // bounded live-reroute rate
      lastLive = now;
      applyMove(kind, id, ground, "live");
    },
    onDragEnd(kind, id, ground) {
      applyMove(kind, id, ground, "drop");
    },
    onGroundClick(ground) {
      if (!placingCell) return;
      const cell = placingCell;
      placingCell = null;
      try {
        const inst = studio.placeInstance(cell, ground);
        log(`placed ${inst.name} (${cell}) at ${ground.join(",")}`);
      } catch (err) {
        toast(String(err));
        log(`place failed: ${err}`);
      }
    },
  });

  function applyMove(kind: "instance" | "gate", id: string, ground: Vec3, phase: "live" | "drop") {
    try {
      if (kind === "instance") {
        const report = studio.moveInstance(id, ground);
        if (phase === "drop") {
          const failed = Object.entries(report.failed);
          log(`move ${id} -> ${ground.join(",")}  rerouted=[${report.rerouted}]` +
              (failed.length ? `  FAILED: ${failed.map(([b, r]) => `${b}: ${r}`).join("; ")}` : ""));
          if (failed.length) toast(`bus failed: ${failed.map(([b]) => b).join(", ")} (shown red)`);
        }
      } else {
        const [busName, gateName] = id.split(" ");
        const bus = studio.buses.get(busName);
        const gate = bus?.gates.find((g) => g.name === gateName);
        if (!gate) return;
        const report = studio.moveGate(busName, gateName, [ground[0], gate.anchor[1], ground[2]]);
        if (phase === "drop") {
          log(`gate ${id} -> ${ground[0]},${gate.anchor[1]},${ground[2]}  state=${report.state} segments=${report.rerouted_segments}`);
          if (report.state.startsWith("failed")) toast(`bus ${busName} ${report.state} (shown red)`);
        }
      }
    } catch (err) {
      if (phase === "drop") {
        toast(String(err));
        log(`move failed: ${err}`);
      }
    }
  }

  function refresh() {
    try {
      viewer.setLayers(studio.layers());
    } catch (err) {
      log(`layer render failed: ${err}`);
    }
    const ports: PortMarker[] = [...studio.ports.values()].map((p) => ({
      name: p.name, kind: p.kind, anchor: p.anchor, step: p.step, width: p.width,
    }));
    const gates: GateMarker[] = [...studio.buses.values()].flatMap((b) => {
      const width = studio.ports.get(b.driver)?.width ?? 1;
      return b.gates.map((g) => ({ bus: b.name, name: g.name, anchor: g.anchor, step: g.step, width }));
    });
    const instances: InstanceMarker[] = [...studio.instances.values()].map((i) => ({
      name: i.name, at: i.at, rot: i.rot,
      dims: studio.cells.get(i.cell)?.dims ?? [1, 1, 1],
    }));
    viewer.setMarkers(ports, gates, instances);
    renderPanels();
  }
  studio.onChange = refresh;

  // ---- side panels ------------------------------------------------------

  function renderPanels() {
    $("#cell-list").innerHTML = [...studio.cells.values()].map((c) => `
      <div class="item">
        <span class="name">${c.name}</span>
        <span class="meta">${c.dims.join("×")} · ${c.source}${c.warnings.length ? ` · ⚠ ${c.warnings.length}` : ""}</span>
        <div class="row"><button data-place="${c.name}">Place</button></div>
      </div>`).join("") || `<span class="meta">load a .schem/.litematic or compile Verilog</span>`;
    $("#cell-list").querySelectorAll("button[data-place]").forEach((b) =>
      b.addEventListener("click", () => {
        placingCell = (b as HTMLElement).dataset.place!;
        toast(`click the canvas ground to place ${placingCell}`);
      }));

    $("#instance-list").innerHTML = [...studio.instances.values()].map((i) => `
      <div class="item"><span class="name">${i.name}</span>
        <span class="meta">${i.cell} @ ${i.at.join(",")} rot ${i.rot}</span></div>`).join("")
      || `<span class="meta">none — drag from Library</span>`;

    $("#port-list").innerHTML = [...studio.ports.values()].map((p) => `
      <div class="item"><span class="name">${p.name}</span>
        <span class="meta">${p.kind} ${p.ty}[${p.width}] @ ${p.anchor.join(",")}</span></div>`).join("")
      || `<span class="meta">none — Load demo declares typed ports</span>`;

    $("#bus-list").innerHTML = [...studio.buses.values()].map((b) => {
      const state = studio.busState(b.name);
      const cls = state.startsWith("failed") ? "state-failed"
        : state === "routed" ? "state-routed" : "state-intended";
      return `<div class="item">
        <span class="swatch" style="background:#${b.color.toString(16).padStart(6, "0")}"></span>
        <span class="name">${b.name}</span> <span class="${cls}">${state}</span>
        <span class="meta">${b.driver} → ${b.sinks.join(",")}${b.gates.length ? ` · gates: ${b.gates.map((g) => g.name).join(",")}` : ""}</span>
      </div>`;
    }).join("") || `<span class="meta">click two ports to route</span>`;

    renderPoke();
  }

  function renderPoke() {
    const panel = $("#poke-panel");
    if (!studio.executor) {
      panel.innerHTML = `<span class="meta">bake first</span>`;
      return;
    }
    const { inputs, outputs } = studio.contractPorts();
    panel.innerHTML = `
      ${inputs.map((p) => `
        <div class="row"><label>${p.name}[${p.width}]</label>
          <input type="number" style="width:90px" data-poke-in="${p.name}" value="0" /></div>`).join("")}
      <div class="row"><button id="btn-poke" class="primary">Set + settle</button></div>
      ${outputs.map((p) => `
        <div class="row"><label>${p.name}[${p.width}]</label>
          <span data-poke-out="${p.name}" class="state-routed">–</span></div>`).join("")}`;
    $("#btn-poke")?.addEventListener("click", () => {
      try {
        for (const el of panel.querySelectorAll<HTMLInputElement>("[data-poke-in]")) {
          studio.executor.set(el.dataset.pokeIn!, Number(el.value) >>> 0);
        }
        studio.executor.settle(800);
        for (const el of panel.querySelectorAll<HTMLElement>("[data-poke-out]")) {
          const v = studio.executor.get(el.dataset.pokeOut!);
          el.textContent = `${v} (0x${Number(v).toString(16)})`;
        }
        log("poke: inputs set, settled, outputs read");
      } catch (err) {
        toast(String(err));
        log(`poke failed: ${err}`);
      }
    });
  }

  // ---- actions ----------------------------------------------------------

  function connectPorts(a: string, b: string) {
    const pa = studio.ports.get(a)!, pb = studio.ports.get(b)!;
    const driver = pa.kind === "input" ? pa : pb;
    const sink = pa.kind === "input" ? pb : pa;
    if (driver.kind !== "input" || sink.kind !== "output") {
      toast("route needs one input (driver) and one output (sink) port");
      return;
    }
    try {
      const bus = studio.routeBus(driver.name, [sink.name]);
      log(`routed ${bus.name}: ${driver.name} -> ${sink.name} (${studio.busState(bus.name)})`);
    } catch (err) {
      toast(String(err));
      log(`route failed: ${err}`);
    }
  }

  $("#cell-file").addEventListener("change", async (e) => {
    const files = (e.target as HTMLInputElement).files ?? [];
    for (const f of files) {
      try {
        const bytes = new Uint8Array(await f.arrayBuffer());
        const name = f.name.replace(/\.[^.]+$/, "");
        const info = studio.addCellFromBytes(name, bytes);
        log(`cell ${name}: ${info.dims.join("×")}${info.warnings.length ? `, warnings: ${info.warnings.join("; ")}` : ""}`);
      } catch (err) {
        toast(`${f.name}: ${err}`);
        log(`cell load failed (${f.name}): ${err}`);
      }
    }
  });

  ($("#verilog") as HTMLTextAreaElement).value = DEMO_VERILOG;
  $("#btn-compile").addEventListener("click", async () => {
    const src = ($("#verilog") as HTMLTextAreaElement).value;
    const name = ($("#verilog-name") as HTMLInputElement).value || guessTop(src) || "cell";
    const status = $("#verilog-status");
    try {
      status.textContent = "yosys…";
      const blif = await verilogToBlif(src, guessTop(src) ?? name);
      status.textContent = "compiling to redstone…";
      await new Promise((r) => setTimeout(r)); // let the status paint
      const cell = core.Hdl.compileBlif(blif, name, false);
      const contract = core.Hdl.compileBlifContract(blif, name);
      cell.setCellContractJson(contract);
      const info = studio.addCellSchematic(name, cell, "verilog");
      status.textContent = `ok: ${cell.blockCount()} blocks, ${info.dims.join("×")}`;
      log(`verilog cell ${name}: ${cell.blockCount()} blocks`);
    } catch (err) {
      status.textContent = String(err).slice(0, 300);
      log(`verilog compile failed: ${err}`);
    }
  });

  $("#btn-check").addEventListener("click", () => {
    try {
      const r = studio.check();
      log(`check: clean=${r.clean} drc=${r.drc.length} rules=${r.rules.length}` +
          (r.clean ? "" : `\n${JSON.stringify(r.raw, null, 1).slice(0, 1500)}`));
      toast(r.clean ? "check clean" : "check DIRTY — see log");
    } catch (err) {
      toast(String(err));
    }
  });

  $("#btn-bake").addEventListener("click", () => {
    try {
      const t0 = performance.now();
      studio.bake(4000);
      log(`baked + executor ready in ${(performance.now() - t0).toFixed(0)}ms`);
      toast("baked — poke panel is live");
    } catch (err) {
      toast(String(err));
      log(`bake failed: ${err}`);
    }
  });

  const download = (bytes: Uint8Array, filename: string) => {
    const url = URL.createObjectURL(new Blob([bytes as unknown as BlobPart]));
    const a = document.createElement("a");
    a.href = url;
    a.download = filename;
    a.click();
    URL.revokeObjectURL(url);
  };
  for (const [id, suffix] of [
    ["#btn-export-schem", "schem"], ["#btn-export-litematic", "litematic"], ["#btn-export-nucm", "nucm"],
  ] as const) {
    $(id).addEventListener("click", () => {
      try {
        download(studio.exportBytes(suffix), `${studio.name}.${suffix}`);
        log(`exported ${studio.name}.${suffix}`);
      } catch (err) {
        toast(String(err));
        log(`export failed: ${err}`);
      }
    });
  }

  // ---- demo (design_demo2's crossing buses, the acceptance geometry) ----

  function loadDemo() {
    const STONE = "minecraft:stone";
    const DUST = "minecraft:redstone_wire[east=none,north=none,power=0,south=none,west=none]";
    const LAMP = "minecraft:redstone_lamp[lit=false]";
    const LEVER = "minecraft:lever[face=floor,facing=north,powered=false]";
    const N = 8, STEP: Vec3 = [0, 2, 0];

    const s = core.Schematic.create("crossing");
    const leverBank = (x: number, z: number, dx: number, dz: number): Vec3 => {
      for (let i = 0; i < N; i++) {
        const y = 2 + 2 * i;
        s.setBlockFromString(x, y - 1, z, STONE);
        s.setBlockFromString(x, y, z, LEVER);
        s.setBlockFromString(x + dx, y - 1, z + dz, STONE);
        s.setBlockFromString(x + dx, y, z + dz, DUST);
      }
      return [x + dx, 2, z + dz];
    };
    const lampBank = (x: number, z: number): Vec3 => {
      for (let i = 0; i < N; i++) {
        const y = 2 + 2 * i;
        s.setBlockFromString(x, y - 1, z, LAMP);
        s.setBlockFromString(x, y, z, DUST);
      }
      return [x, 2, z];
    };
    const aIn = leverBank(0, 8, 1, 0), aOut = lampBank(16, 8);
    const bIn = leverBank(8, 0, 0, 1), bOut = lampBank(8, 16);

    studio = new Studio(core, d, "crossing", s);
    studio.onChange = refresh;
    const port = (name: string, kind: PortInfo["kind"], anchor: Vec3): PortInfo =>
      ({ name, kind, anchor, step: STEP, width: N, ty: "uint" });
    studio.declarePort(port("a_in", "input", aIn));
    studio.declarePort(port("a_out", "output", aOut));
    studio.declarePort(port("b_in", "input", bIn));
    studio.declarePort(port("b_out", "output", bOut));
    const busA = studio.routeBus("a_in", ["a_out"], [{ name: "g0", anchor: [8, 2, 8] as Vec3, step: STEP }]);
    // bus B has no gate: the crossing is IMPLICIT (dip-under tile)
    const busB = studio.routeBus("b_in", ["b_out"]);
    log(`demo loaded: ${busA.name}=${studio.busState(busA.name)} ${busB.name}=${studio.busState(busB.name)}`);
    log("drag the orange gate marker or route more buses; Bake then poke a_in/b_in");
    refresh();
  }
  $("#btn-demo").addEventListener("click", loadDemo);

  refresh();
  if (new URLSearchParams(location.search).has("demo")) loadDemo();
  (window as any).__edaReady = true;
  (window as any).__edaStudio = () => studio;
  (window as any).__edaShot = () => viewer.screenshotDataUrl();
  (window as any).__edaDrag = (kind: "instance" | "gate", id: string, ground: Vec3) =>
    applyMove(kind, id, ground, "drop");
}

boot().catch((err) => {
  $("#status").textContent = `engine failed: ${err}`;
  console.error(err);
});
