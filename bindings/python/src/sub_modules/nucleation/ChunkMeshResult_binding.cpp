#include "diplomat_nanobind_common.hpp"


#include "ChunkMeshResult.hpp"
#include "MeshConfig.hpp"
#include "ResourcePack.hpp"
#include "Schematic.hpp"
#include "TextureAtlas.hpp"

namespace nucleation {
void add_ChunkMeshResult_binding(nb::module_ mod) {
    PyType_Slot nucleation_ChunkMeshResult_slots[] = {
        {Py_tp_free, (void *)nucleation::ChunkMeshResult::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};
    
    nb::class_<nucleation::ChunkMeshResult> opaque(mod, "ChunkMeshResult", nb::type_slots(nucleation_ChunkMeshResult_slots));
    opaque
        .def("chunk_coordinate_at", &nucleation::ChunkMeshResult::chunk_coordinate_at, "index"_a)
        .def("chunk_count", &nucleation::ChunkMeshResult::chunk_count)
        .def_static("create", std::move(maybe_op_unwrap(&nucleation::ChunkMeshResult::create)), "schematic"_a, "pack"_a, "config"_a)
        .def_static("create_with_atlas", std::move(maybe_op_unwrap(&nucleation::ChunkMeshResult::create_with_atlas)), "schematic"_a, "pack"_a, "config"_a, "chunk_size"_a, "atlas"_a)
        .def_static("create_with_size", std::move(maybe_op_unwrap(&nucleation::ChunkMeshResult::create_with_size)), "schematic"_a, "pack"_a, "config"_a, "chunk_size"_a)
        .def("get_mesh", std::move(maybe_op_unwrap(&nucleation::ChunkMeshResult::get_mesh)), "cx"_a, "cy"_a, "cz"_a)
        .def("nucm_data_b64", &nucleation::ChunkMeshResult::nucm_data_b64)
        .def("nucm_data_with_atlas_b64", &nucleation::ChunkMeshResult::nucm_data_with_atlas_b64, "atlas"_a)
        .def("total_triangle_count", &nucleation::ChunkMeshResult::total_triangle_count)
        .def("total_vertex_count", &nucleation::ChunkMeshResult::total_vertex_count);
}

} 