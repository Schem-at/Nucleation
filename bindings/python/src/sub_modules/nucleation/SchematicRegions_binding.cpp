#include "diplomat_nanobind_common.hpp"


#include "DefinitionRegion.hpp"
#include "Schematic.hpp"
#include "SchematicRegions.hpp"

namespace nucleation {
void add_SchematicRegions_binding(nb::module_ mod) {
    PyType_Slot nucleation_SchematicRegions_slots[] = {
        {Py_tp_free, (void *)nucleation::SchematicRegions::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};
    
    nb::class_<nucleation::SchematicRegions> opaque(mod, "SchematicRegions", nb::type_slots(nucleation_SchematicRegions_slots));
    opaque
        .def_static("add", &nucleation::SchematicRegions::add, "schematic"_a, "name"_a, "region"_a)
        .def_static("add_bounds_to", &nucleation::SchematicRegions::add_bounds_to, "schematic"_a, "name"_a, "min_x"_a, "min_y"_a, "min_z"_a, "max_x"_a, "max_y"_a, "max_z"_a)
        .def_static("add_point_to", &nucleation::SchematicRegions::add_point_to, "schematic"_a, "name"_a, "x"_a, "y"_a, "z"_a)
        .def_static("create", &nucleation::SchematicRegions::create, "schematic"_a, "name"_a)
        .def_static("create_from_bounds", &nucleation::SchematicRegions::create_from_bounds, "schematic"_a, "name"_a, "min_x"_a, "min_y"_a, "min_z"_a, "max_x"_a, "max_y"_a, "max_z"_a)
        .def_static("create_from_point", &nucleation::SchematicRegions::create_from_point, "schematic"_a, "name"_a, "x"_a, "y"_a, "z"_a)
        .def_static("create_region", std::move(maybe_op_unwrap(&nucleation::SchematicRegions::create_region)), "schematic"_a, "name"_a, "min_x"_a, "min_y"_a, "min_z"_a, "max_x"_a, "max_y"_a, "max_z"_a)
        .def_static("get", std::move(maybe_op_unwrap(&nucleation::SchematicRegions::get)), "schematic"_a, "name"_a)
        .def_static("names_json", &nucleation::SchematicRegions::names_json, "schematic"_a)
        .def_static("remove", &nucleation::SchematicRegions::remove, "schematic"_a, "name"_a)
        .def_static("set_metadata_on", &nucleation::SchematicRegions::set_metadata_on, "schematic"_a, "name"_a, "key"_a, "value"_a)
        .def_static("shift_region", &nucleation::SchematicRegions::shift_region, "schematic"_a, "name"_a, "dx"_a, "dy"_a, "dz"_a)
        .def_static("update", &nucleation::SchematicRegions::update, "schematic"_a, "name"_a, "region"_a);
}

} 