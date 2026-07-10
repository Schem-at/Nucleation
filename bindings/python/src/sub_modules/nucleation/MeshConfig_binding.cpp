#include "diplomat_nanobind_common.hpp"


#include "MeshConfig.hpp"

namespace nucleation {
void add_MeshConfig_binding(nb::module_ mod) {
    PyType_Slot nucleation_MeshConfig_slots[] = {
        {Py_tp_free, (void *)nucleation::MeshConfig::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};
    
    nb::class_<nucleation::MeshConfig> opaque(mod, "MeshConfig", nb::type_slots(nucleation_MeshConfig_slots));
    opaque
        .def("ambient_occlusion", &nucleation::MeshConfig::ambient_occlusion)
        .def("ao_intensity", &nucleation::MeshConfig::ao_intensity)
        .def("atlas_max_size", &nucleation::MeshConfig::atlas_max_size)
        .def("biome", &nucleation::MeshConfig::biome)
        .def("clear_biome", &nucleation::MeshConfig::clear_biome)
        .def_static("create", std::move(maybe_op_unwrap(&nucleation::MeshConfig::create)))
        .def("cull_hidden_faces", &nucleation::MeshConfig::cull_hidden_faces)
        .def("cull_occluded_blocks", &nucleation::MeshConfig::cull_occluded_blocks)
        .def("greedy_meshing", &nucleation::MeshConfig::greedy_meshing)
        .def("set_ambient_occlusion", &nucleation::MeshConfig::set_ambient_occlusion, "val"_a)
        .def("set_ao_intensity", &nucleation::MeshConfig::set_ao_intensity, "val"_a)
        .def("set_atlas_max_size", &nucleation::MeshConfig::set_atlas_max_size, "size"_a)
        .def("set_biome", &nucleation::MeshConfig::set_biome, "biome"_a)
        .def("set_cull_hidden_faces", &nucleation::MeshConfig::set_cull_hidden_faces, "val"_a)
        .def("set_cull_occluded_blocks", &nucleation::MeshConfig::set_cull_occluded_blocks, "val"_a)
        .def("set_greedy_meshing", &nucleation::MeshConfig::set_greedy_meshing, "val"_a);
}

} 