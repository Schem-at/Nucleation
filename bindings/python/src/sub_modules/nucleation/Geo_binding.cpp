#include "diplomat_nanobind_common.hpp"


#include "Geo.hpp"

namespace nucleation {
void add_Geo_binding(nb::module_ mod) {
    PyType_Slot nucleation_Geo_slots[] = {
        {Py_tp_free, (void *)nucleation::Geo::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};
    
    nb::class_<nucleation::Geo> opaque(mod, "Geo", nb::type_slots(nucleation_Geo_slots));
    opaque
        .def_static("extrude_footprints", std::move(maybe_op_unwrap(&nucleation::Geo::extrude_footprints)), "buildings_json"_a, "base_block"_a, "name"_a)
        .def_static("heightmap_terrain", std::move(maybe_op_unwrap(&nucleation::Geo::heightmap_terrain)), "heights_json"_a, "width"_a, "surface_block"_a, "subsurface_block"_a, "surface_depth"_a, "name"_a);
}

} 