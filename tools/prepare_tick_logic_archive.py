#!/usr/bin/env python3
"""Fetch four pinned logic candidates and inspect their Sponge NBT without Rust.

Python standard library only. Downloads are data, never executed. Default output
is Git-ignored tests/benchmark-fixtures/candidates/logic-archive. Use --offline
to inspect retained files without network access; --catalog PATH also saves the
static catalog at that path. No simulation or vanilla-parity claim is made.
"""
import argparse
import collections
import gzip
import hashlib
import io
import json
import pathlib
import struct
import urllib.request

ROOT = pathlib.Path(__file__).resolve().parents[1]
COMMIT = "6d054aa0a87d8f58f672625b3e6d0e727bd97016"
REPOSITORY = "mattbatwings/Logical-Redstone-Archive"
RAW = f"https://raw.githubusercontent.com/{REPOSITORY}/{COMMIT}/"
MAX_FILE = 2 * 1024 * 1024
MAX_NBT = 32 * 1024 * 1024
SOURCES = [
    {
        "id": "LFF006", "name": "Data (D) Latch", "authors": [],
        "directory": "Archive/LFF_latches-flipflops/LFF006_Data_D_Latch",
        "filename": "LFF006_dlatch.schem", "bytes": 775,
        "sha256": "0663302dfc40ac8c33055d5a43aefe834be4290bdda9baaec6f2ccab238cad54",
        "git_blob_sha1": "dcbc328092b2a5f0487cb5ba9733869f11b7b544",
        "page": "https://mattbatwings.github.io/lra/archives/LFF006-Data-D-Latch/",
        "declared_size": [8, 3, 5],
        "declared_function": "Press Clock to save the current data into the repeater lock.",
        "proposed_checks": ["Store 0 and 1 with data stable throughout the clock pulse.",
            "Change data without a clock pulse and require the stored output to hold.",
            "Capture timing around clock release in vanilla before asserting edge semantics."],
    },
    {
        "id": "COUNT001", "name": "8-bit Counter with Load", "authors": ["mattbatwings"],
        "directory": "Archive/COUNT_counters/COUNT001_8-bit_Counter_with_Load",
        "filename": "COUNT001_8bit_counter_load.schem", "bytes": 1287,
        "sha256": "8e4cf8434924ee79186c59fd975c8941356bfec042da5754ae1245e557cf1aae",
        "git_blob_sha1": "ac5416d91cac1aeb1ae72dcac75042d914454c30",
        "page": "https://mattbatwings.github.io/lra/archives/COUNT001-8-bit-Counter-with-Load/",
        "declared_size": [8, 20, 4],
        "declared_function": "8-bit data bus, Load button, Count up button; 8-bit stored/count output.",
        "proposed_checks": ["Load 0,1,127,254,255 and verify each stored byte.",
            "Count across 127->128 and 255->0; then test a full modulo-256 cycle.",
            "Change load data without Load and require hold; keep Load and Count separate until vanilla defines simultaneous behavior."],
    },
    {
        "id": "DECODE004", "name": "Decoder (horizontal to vertical)", "authors": [],
        "directory": "Archive/DECODE_decoders/DECODE004_Decoder_horizontal_to_vertical",
        "filename": "DECODE004_decode_h2v.schem", "bytes": 637,
        "sha256": "09d6768f157d67845fa0615393077d610ed06c7fc9e57eb3721bb9bb3586267b",
        "git_blob_sha1": "d3d983c472f7a81c736abd1906a2bc15e48cd5d0",
        "page": "https://mattbatwings.github.io/lra/archives/DECODE004-Decoder-horizontal-to-vertical/",
        "declared_size": [7, 9, 11],
        "declared_function": "Programmable 4-bit input, four one-hot outputs if uniquely programmed; not a full 4-to-16 decoder.",
        "proposed_checks": ["Read each row's programmed bit pattern independently of simulation.",
            "Exhaust all 16 input words; output j must equal input==programmed_row_j.",
            "The retained file has four identical rows: do not assert one-hot on this original.",
            "For one-hot coverage, create a separately attributed and hashed derivative with distinct row programming, then capture it in vanilla."],
    },
    {
        "id": "ADD005", "name": "8-bit Carry Look-Everywhere Adder", "authors": ["Kuggo"],
        "directory": "Archive/ADD_adders-subtractors/ADD005_8-bit_Carry_Look-Everywhere_Adder_CLE",
        "filename": "ADD005_8bit_cle.schem", "bytes": 1560,
        "sha256": "3b4e13cd09dc8ad0b5770370d6f416e1d8140e2eec416c207ba76d8a79af46ca",
        "git_blob_sha1": "9da01e667f00ea22e343c005de4e8f0c6aaf4f72",
        "page": "https://mattbatwings.github.io/lra/archives/ADD005-8-bit-Carry-Look-Everywhere-Adder-CLE/",
        "declared_size": [13, 21, 20],
        "declared_function": "Two 8-bit inputs and an 8-bit sum; source advertises four ticks, unit/settling to be verified.",
        "proposed_checks": ["Check (A+B)&255 using zero, all-one, alternating and carry-chain operands.",
            "Exhaust all 65536 input pairs outside timed benchmarks after mapping port order.",
            "The actual schematic also labels CIN/COUT: cover carry-in 0/1 and carry-out separately after port confirmation.",
            "Capture representative transitions on vanilla 1.21.11, including 255+1 and 127+1."],
    },
]


class NBTReader:
    """Bounded, big-endian Java NBT reader; enough for all standard NBT tags."""
    def __init__(self, data):
        self.stream = io.BytesIO(data)

    def take(self, size):
        if size < 0 or size > MAX_NBT:
            raise ValueError("invalid NBT length")
        data = self.stream.read(size)
        if len(data) != size:
            raise ValueError("truncated NBT")
        return data

    def number(self, fmt):
        return struct.unpack(">" + fmt, self.take(struct.calcsize(">" + fmt)))[0]

    def string(self):
        # NBT uses Java modified UTF-8. JSON sign text normally contains ASCII;
        # support encoded NUL and surrogate pairs for non-ASCII labels as well.
        raw = self.take(self.number("H")).replace(b"\xc0\x80", b"\x00")
        text = raw.decode("utf-8", errors="surrogatepass")
        return text.encode("utf-16", errors="surrogatepass").decode("utf-16")

    def count(self):
        count = self.number("i")
        if not 0 <= count <= MAX_NBT:
            raise ValueError("invalid NBT array/list count")
        return count

    def payload(self, tag, depth=0):
        if depth > 64:
            raise ValueError("NBT too deeply nested")
        if tag in (1, 2, 3, 4, 5, 6):
            return self.number({1: "b", 2: "h", 3: "i", 4: "q", 5: "f", 6: "d"}[tag])
        if tag == 7:
            return self.take(self.count())
        if tag == 8:
            return self.string()
        if tag == 9:
            inner, count = self.number("B"), self.count()
            return [self.payload(inner, depth + 1) for _ in range(count)]
        if tag == 10:
            value = {}
            while (inner := self.number("B")) != 0:
                name = self.string()
                if name in value:
                    raise ValueError(f"duplicate NBT key {name!r}")
                value[name] = self.payload(inner, depth + 1)
            return value
        if tag in (11, 12):
            return [self.number("i" if tag == 11 else "q") for _ in range(self.count())]
        raise ValueError(f"unsupported NBT tag {tag}")

    def root(self):
        if self.number("B") != 10:
            raise ValueError("NBT root must be a compound")
        name, value = self.string(), self.payload(10)
        if self.stream.read(1):
            raise ValueError("unexpected trailing NBT data")
        return name, value


def varints(data, expected):
    values, value, shift = [], 0, 0
    for byte in data:
        value |= (byte & 127) << shift
        if byte & 128:
            shift += 7
            if shift >= 35:
                raise ValueError("invalid palette varint")
        else:
            values.append(value)
            value, shift = 0, 0
            if len(values) > expected:
                raise ValueError("too many block indices")
    if shift or len(values) != expected:
        raise ValueError("incorrect block data length")
    return values


def text_components(value):
    """Extract visible sign text without pretending it is an exact port map."""
    if isinstance(value, str):
        try:
            return text_components(json.loads(value))
        except (ValueError, TypeError):
            return value
    if isinstance(value, list):
        return "".join(text_components(v) for v in value)
    if isinstance(value, dict):
        return text_components(value.get("text", "")) + text_components(value.get("extra", []))
    return str(value)


def inspect(data, code):
    with gzip.GzipFile(fileobj=io.BytesIO(data)) as compressed:
        raw = compressed.read(MAX_NBT + 1)
    if len(raw) > MAX_NBT:
        raise ValueError("NBT exceeds decompression limit")
    root_name, root = NBTReader(raw).root()
    schematic = root.get("Schematic", root)
    dimensions = [schematic[k] for k in ("Width", "Height", "Length")]
    width, height, length = dimensions
    volume = width * height * length
    if min(dimensions) <= 0 or volume > 1_000_000:
        raise ValueError("unexpected dimensions")
    block_section = schematic.get("Blocks", schematic)
    palette = block_section["Palette"]
    by_id = {index: descriptor for descriptor, index in palette.items()}
    if len(by_id) != len(palette):
        raise ValueError("duplicate palette index")
    values = varints(block_section.get("Data", block_section.get("BlockData")), volume)
    histogram = collections.Counter()
    controls, lamps, boundary_diodes, programming_cells = [], [], [], []
    for index, palette_id in enumerate(values):
        descriptor = by_id[palette_id]
        name = descriptor.split("[", 1)[0]
        histogram[name] += 1
        pos = [index % width, index // (width * length), (index // width) % length]
        cell = {"pos": pos, "state": descriptor}
        if name.endswith(":lever") or name.endswith("_button"):
            controls.append(cell)
        if name in ("minecraft:redstone_lamp", "minecraft:copper_bulb"):
            lamps.append(cell)
        if name in ("minecraft:repeater", "minecraft:comparator") and any(
                p in (0, dimension - 1) for p, dimension in zip(pos, dimensions)):
            boundary_diodes.append(cell)
        if code == "DECODE004" and pos[0] == 1 and pos[1] in (2, 4, 6, 8) and pos[2] in (4, 6, 8, 10):
            programming_cells.append(cell)
    block_entities = block_section.get("BlockEntities", schematic.get("BlockEntities", []))
    signs = []
    for entity in block_entities:
        fields = entity.get("Data", entity)
        label = entity.get("Id", fields.get("id", fields.get("Id", "")))
        if "sign" not in label:
            continue
        sides = {side: [text_components(t) for t in fields.get(side, {}).get("messages", [])]
                 for side in ("front_text", "back_text")}
        if not any(sides.values()):
            sides["legacy"] = [text_components(fields.get("Text" + str(i), "")) for i in range(1, 5)]
        signs.append({"pos": entity.get("Pos", fields.get("Pos")), "id": label, "text": sides})
    return {
        "root_name": root_name, "sponge_version": schematic["Version"],
        "data_version": schematic["DataVersion"], "dimensions": dimensions,
        "offset": schematic.get("Offset"), "volume": volume,
        "non_air_blocks": sum(n for block, n in histogram.items() if block not in
            ("minecraft:air", "minecraft:cave_air", "minecraft:void_air")),
        "entity_count": len(schematic.get("Entities", [])), "block_entity_count": len(block_entities),
        "block_histogram": dict(sorted(histogram.items())),
        "palette": dict(sorted(palette.items(), key=lambda item: item[1])),
        "piston_blocks": sum(n for block, n in histogram.items() if "piston" in block),
        "controls": controls, "lamps": lamps, "boundary_diodes": boundary_diodes, "signs": signs,
        "decoder_programming_cells": programming_cells,
        "coordinate_convention": "Schematic-local [x,y,z], X fastest then Z then Y; Offset retained separately.",
        "io_mapping_status": "Static controls/signs only; bit order, electrical ports and timing not yet validated.",
    }


def io_candidates(code):
    """Human-readable static findings, deliberately separate from validated I/O."""
    if code == "LFF006":
        return {"clock_button": [7, 1, 2], "data_lever": [0, 1, 4], "output_lamp": [0, 1, 0],
                "evidence": "Clock/Data signs adjacent to their controls; single output lamp. Timing not simulated."}
    if code == "COUNT001":
        return {"load_button": [0, 1, 2], "count_button": [0, 3, 2],
                "data_levers_bottom_to_top": [[0, 5 + 2*i, 2] for i in range(8)],
                "output_lamps_bottom_to_top": [[5, 5 + 2*i, 3] for i in range(8)],
                "evidence": "Load/Count signs identify buttons. Separate data-adjacent and output lamp columns. Bit order unconfirmed.",
                "initialization_note": "One saved output lamp is lit at [5,11,3]; explicitly load zero after settling rather than assuming a zero counter."}
    if code == "DECODE004":
        return {"input_levers_increasing_z": [[6, 1, z] for z in (4, 6, 8, 10)],
                "output_lamps_bottom_to_top": [[0, y, 0] for y in (2, 4, 6, 8)],
                "evidence": "No sign labels. Each output row has repeater/torch/repeater/torch at the same four z positions.",
                "programming_warning": "All four rows are identical in the downloaded artifact. The page's one-hot claim is conditional on unique programming; this original does not meet that condition.",
                "expected_input_pattern_hypothesis": [0, 1, 0, 1],
                "hypothesis_scope": "Inferred from repeater/torch mismatch detection feeding output inversion; confirm in vanilla before treating as a fixture oracle."}
    if code == "ADD005":
        return {"input_pair_positions_by_weight": [{"weight": 1 << i,
                    "positions": [[0, 4 + 2*i, 17 - 2*i], [0, 4 + 2*i, 19 - 2*i]]} for i in range(8)],
                "sum_lamps_by_weight": [{"weight": 1 << i, "pos": [11, 1 + 2*i, 15 - 2*i]} for i in range(8)],
                "carry_in_lever": [5, 1, 18], "carry_out_lamp": [10, 18, 7],
                "evidence": "Input and sum sign labels give weights 1..128; CIN/COUT signs identify extra carry controls/output. A/B names are interchangeable for addition.",
                "timing_note": "COUT sign says '(2 ticks)' while archive advertises four ticks for sum; capture exact game-tick timing independently."}
    raise ValueError(code)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=pathlib.Path,
                        default=ROOT / "tests/benchmark-fixtures/candidates/logic-archive")
    parser.add_argument("--catalog", type=pathlib.Path, help="Also save catalog here (no schematic bytes).")
    parser.add_argument("--offline", action="store_true", help="Require retained pinned source files.")
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)
    catalog = {
        "schema_version": 1, "repository": "https://github.com/" + REPOSITORY,
        "commit": COMMIT, "upstream_commit_date": "2026-11-07T04:01:42Z",
        "license_status": "No license file found in the pinned schematic repository tree; GitHub license metadata is null. The separate website license is not a schematic license.",
        "artifact_policy": "Retain downloaded artifacts in ignored storage; this catalog contains metadata and inspection only.",
        "status": "Downloaded and statically inspected; not imported, simulated or validated against vanilla.",
        "intended_reference": "Minecraft Java 1.21.11; preserve each artifact's actual DataVersion when importing.",
        "provenance_note": "All four entries were archived 2026-02-15 and endorsed by mattbatwings. Empty authors means upstream did not name an author, not public domain.",
        "files": [],
    }
    for source in SOURCES:
        relative = source["directory"] + "/attachments/" + source["filename"]
        url, destination = RAW + relative, args.output / source["filename"]
        if destination.exists():
            data = destination.read_bytes()
        elif args.offline:
            raise FileNotFoundError(destination)
        else:
            with urllib.request.urlopen(url, timeout=30) as response:
                data = response.read(MAX_FILE + 1)
        sha256 = hashlib.sha256(data).hexdigest()
        blob = hashlib.sha1(b"blob " + str(len(data)).encode() + b"\0" + data).hexdigest()
        if len(data) != source["bytes"] or sha256 != source["sha256"] or blob != source["git_blob_sha1"]:
            raise ValueError(f"{source['id']}: pinned length, SHA-256 or Git blob identity mismatch")
        observed = inspect(data, source["id"])
        if observed["dimensions"] != source["declared_size"]:
            raise ValueError(f"{source['id']}: advertised dimensions disagree with NBT")
        if not destination.exists():
            destination.write_bytes(data)
        entry = {**source, "download_url": url, "declared_minecraft_version": "1.21.4",
                 "metadata_url": RAW + source["directory"] + "/data.json",
                 "sha256_verified": True, "git_blob_verified": True, "inspection": observed,
                 "candidate_io_mapping": io_candidates(source["id"]),
                 "simulation_status": "not_run", "vanilla_status": "not_run"}
        catalog["files"].append(entry)
        print(f"{source['id']}: DV={observed['data_version']} size={observed['dimensions']} "
              f"nonair={observed['non_air_blocks']} entities={observed['entity_count']} "
              f"pistons={observed['piston_blocks']}; simulation not run")
    serialized = json.dumps(catalog, indent=2, ensure_ascii=False) + "\n"
    (args.output / "manifest.json").write_text(serialized)
    if args.catalog:
        args.catalog.parent.mkdir(parents=True, exist_ok=True)
        args.catalog.write_text(serialized)


if __name__ == "__main__":
    main()
