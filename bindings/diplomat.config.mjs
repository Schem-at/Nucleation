import { fileURLToPath } from "node:url";
import path from "node:path";

const here = path.dirname(fileURLToPath(import.meta.url));

export default {
  wasm_path: path.join(here, "..", "target", "wasm32-unknown-unknown", "release", "nucleation.wasm"),
};
