#include "diplomat_nanobind_common.hpp"


#include "WorldChunkView.hpp"

namespace nucleation {
void add_WorldChunkView_binding(nb::module_ mod) {
    PyType_Slot nucleation_WorldChunkView_slots[] = {
        {Py_tp_free, (void *)nucleation::WorldChunkView::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};
    
    nb::class_<nucleation::WorldChunkView> opaque(mod, "WorldChunkView", nb::type_slots(nucleation_WorldChunkView_slots));
    opaque
        .def("biome_palette_json", &nucleation::WorldChunkView::biome_palette_json)
        .def_static("create", std::move(maybe_op_unwrap(&nucleation::WorldChunkView::create)), "cx"_a, "cz"_a)
        .def("cx", &nucleation::WorldChunkView::cx)
        .def("cz", &nucleation::WorldChunkView::cz)
        .def("set_biome", &nucleation::WorldChunkView::set_biome, "biome_name"_a)
        .def("set_block", &nucleation::WorldChunkView::set_block, "x"_a, "y"_a, "z"_a, "block_name"_a)
        .def("to_schematic", std::move(maybe_op_unwrap(&nucleation::WorldChunkView::to_schematic)));
}

} 