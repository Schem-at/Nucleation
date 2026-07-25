#include "diplomat_nanobind_common.hpp"


#include "Brush.hpp"
#include "CellularSdfConfig.hpp"
#include "GeneratedChunkOverlayMode.hpp"
#include "Sdf.hpp"
#include "WorldGenerator.hpp"

namespace nucleation {
void add_WorldGenerator_binding(nb::module_ mod) {
    PyType_Slot nucleation_WorldGenerator_slots[] = {
        {Py_tp_free, (void *)nucleation::WorldGenerator::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};

    nb::class_<nucleation::WorldGenerator> opaque(mod, "WorldGenerator", nb::type_slots(nucleation_WorldGenerator_slots));
    opaque
        .def("add_layer", &nucleation::WorldGenerator::add_layer, "source"_a, "mode"_a)
        .def_static("cellular_sdf", std::move(maybe_op_unwrap(&nucleation::WorldGenerator::cellular_sdf)), "volume"_a, "material"_a, "min_y"_a, "max_y"_a, "config"_a, "source_id"_a, "version"_a)
        .def_static("composite", std::move(maybe_op_unwrap(&nucleation::WorldGenerator::composite)), "source_id"_a, "version"_a)
        .def("generate", std::move(maybe_op_unwrap(&nucleation::WorldGenerator::generate)), "cx"_a, "cz"_a)
        .def_static("projected_footprints", std::move(maybe_op_unwrap(&nucleation::WorldGenerator::projected_footprints)), "buildings_json"_a, "base_block"_a, "source_id"_a, "version"_a)
        .def_static("sdf", std::move(maybe_op_unwrap(&nucleation::WorldGenerator::sdf)), "volume"_a, "material"_a, "min_y"_a, "max_y"_a, "source_id"_a, "version"_a)
        .def("stream", std::move(maybe_op_unwrap(&nucleation::WorldGenerator::stream)), "min_cx"_a, "min_cz"_a, "max_cx"_a, "max_cz"_a);
}

}
