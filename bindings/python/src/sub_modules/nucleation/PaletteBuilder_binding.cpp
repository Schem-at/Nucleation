#include "diplomat_nanobind_common.hpp"


#include "PaletteBuilder.hpp"

namespace nucleation {
void add_PaletteBuilder_binding(nb::module_ mod) {
    PyType_Slot nucleation_PaletteBuilder_slots[] = {
        {Py_tp_free, (void *)nucleation::PaletteBuilder::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};
    
    nb::class_<nucleation::PaletteBuilder> opaque(mod, "PaletteBuilder", nb::type_slots(nucleation_PaletteBuilder_slots));
    opaque
        .def("build", std::move(maybe_op_unwrap(&nucleation::PaletteBuilder::build)))
        .def("chroma_below", &nucleation::PaletteBuilder::chroma_below, "max"_a)
        .def("color_near", &nucleation::PaletteBuilder::color_near, "r"_a, "g"_a, "b"_a, "max_distance"_a)
        .def_static("create", std::move(maybe_op_unwrap(&nucleation::PaletteBuilder::create)))
        .def("exclude_falling", &nucleation::PaletteBuilder::exclude_falling)
        .def("exclude_keyword", &nucleation::PaletteBuilder::exclude_keyword, "keyword"_a)
        .def("exclude_light_sources", &nucleation::PaletteBuilder::exclude_light_sources)
        .def("exclude_needs_support", &nucleation::PaletteBuilder::exclude_needs_support)
        .def("exclude_tag", &nucleation::PaletteBuilder::exclude_tag, "t"_a)
        .def("exclude_tile_entities", &nucleation::PaletteBuilder::exclude_tile_entities)
        .def("exclude_transparent", &nucleation::PaletteBuilder::exclude_transparent)
        .def("full_blocks_only", &nucleation::PaletteBuilder::full_blocks_only)
        .def("include_keyword", &nucleation::PaletteBuilder::include_keyword, "keyword"_a)
        .def("kind", &nucleation::PaletteBuilder::kind, "k"_a)
        .def("lightness_between", &nucleation::PaletteBuilder::lightness_between, "min"_a, "max"_a)
        .def("survival_only", &nucleation::PaletteBuilder::survival_only)
        .def("tag", &nucleation::PaletteBuilder::tag, "t"_a);
}

} 