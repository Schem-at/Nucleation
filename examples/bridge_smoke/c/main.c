/* End-to-end smoke test for the generated C bindings, exercising one method from
 * every unconditionally-compiled bridge module against the real crate. */
#include <assert.h>
#include <stdio.h>
#include <string.h>

#include "Autostack.h"
#include "DefinitionRegion.h"
#include "Diff.h"
#include "Schematic.h"
#include "SchematicBuilder.h"
#include "SchematicRegions.h"
#include "Store.h"

static DiplomatStringView sv(const char *s) {
    DiplomatStringView v = {s, strlen(s)};
    return v;
}

int main(void) {
    /* --- schematic: create/set/get + error path --- */
    Schematic *s = Schematic_create(sv("smoke"));
    Schematic_set_block_result set_res = Schematic_set_block(s, 1, 2, 3, sv("minecraft:stone"));
    assert(set_res.is_ok && set_res.ok);

    char buf[4096];
    DiplomatWrite w = diplomat_simple_write(buf, sizeof(buf));
    Schematic_get_block_name_result gn = Schematic_get_block_name(s, 1, 2, 3, &w);
    assert(gn.is_ok);
    assert(strncmp(buf, "minecraft:stone", w.len) == 0);

    w = diplomat_simple_write(buf, sizeof(buf));
    Schematic_get_block_name_result miss = Schematic_get_block_name(s, 40, 40, 40, &w);
    assert(!miss.is_ok && miss.err == NucleationError_NotFound);

    /* --- save/load roundtrip --- */
    Schematic_save_to_file_result sf = Schematic_save_to_file(s, sv("/tmp/bridge_smoke.litematic"));
    assert(sf.is_ok);
    Schematic_load_from_file_result lf = Schematic_load_from_file(sv("/tmp/bridge_smoke.litematic"));
    assert(lf.is_ok);
    Schematic *loaded = lf.ok;

    /* --- builder: consuming build + AlreadyConsumed --- */
    SchematicBuilder *b = SchematicBuilder_create();
    assert(SchematicBuilder_map(b, sv("s"), sv("minecraft:stone")).is_ok);
    assert(SchematicBuilder_layer(b, sv("[\"s\"]")).is_ok);
    SchematicBuilder_build_result br = SchematicBuilder_build(b);
    assert(br.is_ok);
    Schematic *built = br.ok;
    SchematicBuilder_build_result br2 = SchematicBuilder_build(b);
    assert(!br2.is_ok && br2.err == NucleationError_AlreadyConsumed);

    /* --- diff: distance between original and its saved copy is 0 --- */
    Diff_compute_result dr = Diff_compute(s, loaded, sv("exact"));
    assert(dr.is_ok);
    assert(Diff_distance(dr.ok) == 0);
    Diff_destroy(dr.ok);

    /* --- autostack: JSON out --- */
    w = diplomat_simple_write(buf, sizeof(buf));
    Autostack_detect_structures(s, &w);
    assert(w.len > 0 && buf[0] == '[');

    /* --- definition regions --- */
    DefinitionRegion *r = DefinitionRegion_create();
    DefinitionRegion_add_point(r, 1, 2, 3);
    SchematicRegions_add_result ar = SchematicRegions_add(s, sv("io"), r);
    assert(ar.is_ok);
    w = diplomat_simple_write(buf, sizeof(buf));
    SchematicRegions_names_json_result nj = SchematicRegions_names_json(s, &w);
    assert(nj.is_ok);
    assert(strncmp(buf, "[\"io\"]", w.len) == 0);
    DefinitionRegion_destroy(r);

    /* --- store: mem:// save/open roundtrip --- */
    Store_open_result so = Store_open(sv("mem://"));
    assert(so.is_ok);
    Store *store = so.ok;
    Store_save_schematic_result ss = Store_save_schematic(store, s, sv("k1.litematic"), sv(""));
    assert(ss.is_ok);
    Store_open_schematic_result os = Store_open_schematic(store, sv("k1.litematic"));
    assert(os.is_ok);
    Schematic_destroy(os.ok);
    Store_destroy(store);

    Schematic_destroy(built);
    Schematic_destroy(loaded);
    Schematic_destroy(s);
    remove("/tmp/bridge_smoke.litematic");

    printf("bridge smoke (C) OK\n");
    return 0;
}
