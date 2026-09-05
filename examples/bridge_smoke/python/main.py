"""End-to-end smoke test for the generated Python (nanobind) bindings."""
import base64
import json
import struct

import nucleation as m

# --- schematic: create/set/get + error path ---
s = m.Schematic.create("smoke")
assert s.set_block(1, 2, 3, "minecraft:stone") is True
assert s.get_block_name(1, 2, 3) == "minecraft:stone"
try:
    s.get_block_name(40, 40, 40)
    raise AssertionError("expected NotFound")
except Exception as e:
    assert "NotFound" in repr(e) or "NotFound" in str(e.args), e

# --- serialize roundtrip in-memory ---
b64 = s.to_litematic_b64()
loaded = m.Schematic.from_litematic(base64.b64decode(b64))
assert loaded.get_block_name(1, 2, 3) == "minecraft:stone"

# --- bulk block queries (count / replace / packed export) ---
counts = json.loads(s.count_blocks_json())
assert counts["minecraft:stone"] == 1, counts
assert s.replace_blocks_json('{"minecraft:stone":"minecraft:glass"}') == 1
assert s.replace_blocks_json('{"minecraft:glass":"minecraft:stone"}') == 1
packed = base64.b64decode(s.non_air_blocks_packed_b64())
(count,) = struct.unpack_from("<I", packed, 0)
assert count == 1, count
x, y, z, index = struct.unpack_from("<iiiH", packed, 4)
assert (x, y, z) == (1, 2, 3), (x, y, z)
(plen,) = struct.unpack_from("<I", packed, 4 + count * 14)
palette = json.loads(packed[8 + count * 14 : 8 + count * 14 + plen])
assert palette[index] == "minecraft:stone", palette

# --- builder: consuming build + AlreadyConsumed ---
b = m.SchematicBuilder.create()
b.map("s", "minecraft:stone")
b.layer('["s"]')
built = b.build()
try:
    b.build()
    raise AssertionError("expected AlreadyConsumed")
except Exception as e:
    assert "AlreadyConsumed" in repr(e) or "AlreadyConsumed" in str(e.args), e

# --- diff ---
diff = m.Diff.compute(s, loaded, "exact")
assert diff.distance() == 0

# --- autostack ---
assert m.Autostack.detect_structures(s).startswith("[")

# --- definition regions ---
r = m.DefinitionRegion.create()
r.add_point(1, 2, 3)
m.SchematicRegions.add(s, "io", r)
assert m.SchematicRegions.names_json(s) == '["io"]'

# --- store: mem:// roundtrip ---
store = m.Store.open("mem://")
store.save_schematic(s, "k1.litematic", "")
reopened = store.open_schematic("k1.litematic")
assert reopened.get_block_name(1, 2, 3) == "minecraft:stone"

# --- construction animation: fluent one-shot effect ---
animation = m.BuildAnimation.create("fluent")
effect = m.AnimationEffect.spin_in(600.0, 1.0)
assert animation.with_effect(effect).set_block(0, 0, 0, "minecraft:stone") == 0
assert animation.set_block(1, 0, 0, "minecraft:dirt") == 1
assert animation.group_count() == 2

# --- portable field program: length(position) - 2 ---
program_builder = m.FieldProgramBuilder.create()
distance = program_builder.add_slot(m.FieldProgramValueType.Scalar)
program_builder.set_output(distance)
program_builder.set_bounds(-2.0, -2.0, -2.0, 2.0, 2.0, 2.0)
program_builder.set_distance_kind(m.FieldProgramDistanceKind.Exact)
program_builder.push_pos()
program_builder.unary_op(m.FieldProgramUnaryOp.Length)
program_builder.push_const_scalar(2.0)
program_builder.binary_op(m.FieldProgramBinaryOp.Sub)
program_builder.store_local(distance)
program = program_builder.build()
assert '"version":1' in program.to_json()
program_sdf = m.Sdf.from_program(program)
assert program_sdf.eval_at(0.0, 0.0, 0.0) < 0.0

print("bridge smoke (Python) OK")
