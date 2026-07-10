#include "diplomat_nanobind_common.hpp"


#include "DefinitionRegion.hpp"
#include "Schematic.hpp"

namespace nucleation {
void add_DefinitionRegion_binding(nb::module_ mod) {
    PyType_Slot nucleation_DefinitionRegion_slots[] = {
        {Py_tp_free, (void *)nucleation::DefinitionRegion::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};
    
    nb::class_<nucleation::DefinitionRegion> opaque(mod, "DefinitionRegion", nb::type_slots(nucleation_DefinitionRegion_slots));
    opaque
        .def("add_bounds", &nucleation::DefinitionRegion::add_bounds, "min_x"_a, "min_y"_a, "min_z"_a, "max_x"_a, "max_y"_a, "max_z"_a)
        .def("add_filter", &nucleation::DefinitionRegion::add_filter, "filter"_a)
        .def("add_point", &nucleation::DefinitionRegion::add_point, "x"_a, "y"_a, "z"_a)
        .def("all_metadata_json", &nucleation::DefinitionRegion::all_metadata_json)
        .def("blocks_json", &nucleation::DefinitionRegion::blocks_json, "schematic"_a)
        .def("bounds", &nucleation::DefinitionRegion::bounds)
        .def("box_count", &nucleation::DefinitionRegion::box_count)
        .def("boxes_json", &nucleation::DefinitionRegion::boxes_json)
        .def("center", &nucleation::DefinitionRegion::center)
        .def("center_f32_json", &nucleation::DefinitionRegion::center_f32_json)
        .def("connected_components", &nucleation::DefinitionRegion::connected_components)
        .def("contains", &nucleation::DefinitionRegion::contains, "x"_a, "y"_a, "z"_a)
        .def("contract", &nucleation::DefinitionRegion::contract, "amount"_a)
        .def("contracted", std::move(maybe_op_unwrap(&nucleation::DefinitionRegion::contracted)), "amount"_a)
        .def("copy", std::move(maybe_op_unwrap(&nucleation::DefinitionRegion::copy)))
        .def_static("create", std::move(maybe_op_unwrap(&nucleation::DefinitionRegion::create)))
        .def("dimensions", &nucleation::DefinitionRegion::dimensions)
        .def("exclude_block", &nucleation::DefinitionRegion::exclude_block, "schematic"_a, "block_name"_a)
        .def("expand", &nucleation::DefinitionRegion::expand, "x"_a, "y"_a, "z"_a)
        .def("expanded", std::move(maybe_op_unwrap(&nucleation::DefinitionRegion::expanded)), "x"_a, "y"_a, "z"_a)
        .def("filter_by_block", std::move(maybe_op_unwrap(&nucleation::DefinitionRegion::filter_by_block)), "schematic"_a, "block_name"_a)
        .def("filter_by_properties", std::move(maybe_op_unwrap(&nucleation::DefinitionRegion::filter_by_properties)), "schematic"_a, "properties_json"_a)
        .def_static("from_bounding_boxes", std::move(maybe_op_unwrap(&nucleation::DefinitionRegion::from_bounding_boxes)), "boxes"_a)
        .def_static("from_bounds", std::move(maybe_op_unwrap(&nucleation::DefinitionRegion::from_bounds)), "min_x"_a, "min_y"_a, "min_z"_a, "max_x"_a, "max_y"_a, "max_z"_a)
        .def_static("from_positions", std::move(maybe_op_unwrap(&nucleation::DefinitionRegion::from_positions)), "positions"_a)
        .def("get_box", &nucleation::DefinitionRegion::get_box, "index"_a)
        .def("get_metadata", &nucleation::DefinitionRegion::get_metadata, "key"_a)
        .def("intersected", std::move(maybe_op_unwrap(&nucleation::DefinitionRegion::intersected)), "other"_a)
        .def("intersects_bounds", &nucleation::DefinitionRegion::intersects_bounds, "min_x"_a, "min_y"_a, "min_z"_a, "max_x"_a, "max_y"_a, "max_z"_a)
        .def("is_contiguous", &nucleation::DefinitionRegion::is_contiguous)
        .def("is_empty", &nucleation::DefinitionRegion::is_empty)
        .def("merge", &nucleation::DefinitionRegion::merge, "other"_a)
        .def("metadata_keys_json", &nucleation::DefinitionRegion::metadata_keys_json)
        .def("positions_json", &nucleation::DefinitionRegion::positions_json)
        .def("positions_sorted_json", &nucleation::DefinitionRegion::positions_sorted_json)
        .def("set_color", &nucleation::DefinitionRegion::set_color, "color"_a)
        .def("set_metadata", &nucleation::DefinitionRegion::set_metadata, "key"_a, "value"_a)
        .def("shift", &nucleation::DefinitionRegion::shift, "dx"_a, "dy"_a, "dz"_a)
        .def("shifted", std::move(maybe_op_unwrap(&nucleation::DefinitionRegion::shifted)), "dx"_a, "dy"_a, "dz"_a)
        .def("simplify", &nucleation::DefinitionRegion::simplify)
        .def("subtracted", std::move(maybe_op_unwrap(&nucleation::DefinitionRegion::subtracted)), "other"_a)
        .def("sync", &nucleation::DefinitionRegion::sync, "schematic"_a, "name"_a)
        .def("union_into", &nucleation::DefinitionRegion::union_into, "other"_a)
        .def("union_with", std::move(maybe_op_unwrap(&nucleation::DefinitionRegion::union_with)), "other"_a)
        .def("volume", &nucleation::DefinitionRegion::volume);
}

} 