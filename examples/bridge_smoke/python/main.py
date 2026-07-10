"""End-to-end smoke test for the generated Python (nanobind) bindings."""
import base64
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

print("bridge smoke (Python) OK")
