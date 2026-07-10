#include "diplomat_nanobind_common.hpp"


#include "SchematicBuilder.hpp"

namespace nucleation {
void add_SchematicBuilder_binding(nb::module_ mod) {
    PyType_Slot nucleation_SchematicBuilder_slots[] = {
        {Py_tp_free, (void *)nucleation::SchematicBuilder::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};
    
    nb::class_<nucleation::SchematicBuilder> opaque(mod, "SchematicBuilder", nb::type_slots(nucleation_SchematicBuilder_slots));
    opaque
        .def("build", std::move(maybe_op_unwrap(&nucleation::SchematicBuilder::build)))
        .def_static("create", std::move(maybe_op_unwrap(&nucleation::SchematicBuilder::create)))
        .def_static("from_template", std::move(maybe_op_unwrap(&nucleation::SchematicBuilder::from_template)), "template"_a)
        .def("layer", &nucleation::SchematicBuilder::layer, "rows_json"_a)
        .def("layers", &nucleation::SchematicBuilder::layers, "layers_json"_a)
        .def("map", &nucleation::SchematicBuilder::map, "ch"_a, "block"_a)
        .def("name", &nucleation::SchematicBuilder::name, "name"_a)
        .def("offset", &nucleation::SchematicBuilder::offset, "x"_a, "y"_a, "z"_a)
        .def("palette", &nucleation::SchematicBuilder::palette, "pairs_json"_a)
        .def("to_template", &nucleation::SchematicBuilder::to_template)
        .def("use_compact_palette", &nucleation::SchematicBuilder::use_compact_palette)
        .def("use_minimal_palette", &nucleation::SchematicBuilder::use_minimal_palette)
        .def("use_standard_palette", &nucleation::SchematicBuilder::use_standard_palette)
        .def("validate", &nucleation::SchematicBuilder::validate);
}

} 