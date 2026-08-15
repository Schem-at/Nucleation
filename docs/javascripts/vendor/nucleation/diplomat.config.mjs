// Isomorphic: fs.readFileSync accepts file:// URLs in Node; the browser
// branch fetches the same URL relative to the module.
export default {
  wasm_path: new URL("./nucleation.wasm", import.meta.url),
};
