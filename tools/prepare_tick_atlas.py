#!/usr/bin/env python3
"""Assemble local Atlas browser assets from a WASM binary built on harrison-work.

This helper does not compile Rust, download files or publish the local archive.
The resource pack is visual input, not a reference for simulation mechanics.
"""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import zipfile

ROOT = Path(__file__).resolve().parents[1]


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def copy(source, target):
    if source.resolve() != target.resolve(): shutil.copy2(source, target)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--wasm', type=Path, required=True)
    parser.add_argument('--pack', type=Path, required=True)
    parser.add_argument('--engine-commit', required=True)
    parser.add_argument('--compiler', default='not recorded')
    parser.add_argument('--metadata-output', type=Path)
    args = parser.parse_args()
    with args.wasm.open('rb') as file:
        if file.read(8) != b'\0asm\x01\0\0\0': raise ValueError('Expected a WebAssembly v1 binary')
    with zipfile.ZipFile(args.pack) as archive:
        if archive.testzip() is not None: raise ValueError('Resource pack CRC failure')
    public = ROOT / 'apps/sim-lab-wasm/public'
    engine, pack = public / 'engine', public / 'pack'
    engine.mkdir(parents=True, exist_ok=True); pack.mkdir(parents=True, exist_ok=True)
    for pattern in ['*.mjs', '*.d.ts', '*.d.mts']:
        for source in (ROOT / 'bindings/js').glob(pattern): copy(source, engine / source.name)
    copy(args.wasm, engine / 'nucleation.wasm')
    copy(args.pack, pack / 'mesher-pack.zip')
    (engine / 'diplomat.config.mjs').write_text('export default { wasm_path: new URL("./nucleation.wasm", import.meta.url) };\n')
    glue = engine / 'diplomat-wasm.mjs'
    glue.write_text(glue.read_text().replace("'../diplomat.config.mjs'", "'./diplomat.config.mjs'"))
    metadata = {'host': 'harrison-work', 'rustc': args.compiler, 'engine_commit': args.engine_commit,
                'target': 'wasm32-unknown-unknown', 'features': ['bridge','mc-tick','meshing'],
                'profile': 'release', 'wasm_sha256': digest(engine / 'nucleation.wasm'),
                'wasm_bytes': (engine / 'nucleation.wasm').stat().st_size,
                'asset_pack_sha256': digest(pack / 'mesher-pack.zip'),
                'asset_pack_note': 'Retained visualization pack; not a physics reference version.',
                'simulation': 'Single-threaded browser mc-tick reference configuration; not the native PGO benchmark binary.'}
    (engine / 'build-info.json').write_text(json.dumps(metadata, indent=2) + '\n')
    if args.metadata_output: args.metadata_output.write_text(json.dumps(metadata, indent=2) + '\n')
    print(json.dumps(metadata))


if __name__ == '__main__': main()
