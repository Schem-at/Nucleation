/** Verilog -> BLIF via YoWASP yosys (wasm, fully in-browser).
 *
 * Mirrors the verified Python pipeline's recipe exactly
 * (redstone-eda/hdl/hdl2redstone.py):
 *   read_verilog; synth -top <top> -lut 4; opt_clean; write_blif
 * `-lut 4` makes yosys emit `.names` truth tables, which is the only BLIF
 * dialect `Hdl.compileBlif` accepts (combinational; no .latch/.subckt).
 */
import { runYosys } from "@yowasp/yosys";

export async function verilogToBlif(source: string, top: string): Promise<string> {
  const logs: string[] = [];
  const script = `synth -top ${top} -lut 4; opt_clean; write_blif out.blif`;
  let out: Record<string, unknown>;
  try {
    out = (await runYosys(["-q", "-p", script, "design.v"], { "design.v": source }, {
      stdout: (bytes: Uint8Array | null) => { if (bytes != null) logs.push(new TextDecoder().decode(bytes)); },
      stderr: (bytes: Uint8Array | null) => { if (bytes != null) logs.push(new TextDecoder().decode(bytes)); },
    })) as unknown as Record<string, unknown>;
  } catch (err) {
    throw new Error(`yosys failed: ${err}\n${logs.slice(-15).join("\n")}`);
  }
  const blif = out?.["out.blif"];
  if (blif == null) throw new Error(`yosys produced no BLIF\n${logs.slice(-15).join("\n")}`);
  return typeof blif === "string" ? blif : new TextDecoder().decode(blif as Uint8Array);
}

/** Guess the top module name from the source (first `module x` found). */
export function guessTop(source: string): string | null {
  return /\bmodule\s+([A-Za-z_][A-Za-z0-9_$]*)/.exec(source)?.[1] ?? null;
}
