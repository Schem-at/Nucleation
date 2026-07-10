#include "diplomat_nanobind_common.hpp"


#include "MeshConfig.hpp"
#include "MultiMeshResult.hpp"
#include "ResourcePack.hpp"
#include "Schematic.hpp"

namespace nucleation {
void add_MultiMeshResult_binding(nb::module_ mod) {
    PyType_Slot nucleation_MultiMeshResult_slots[] = {
        {Py_tp_free, (void *)nucleation::MultiMeshResult::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};
    
    nb::class_<nucleation::MultiMeshResult> opaque(mod, "MultiMeshResult", nb::type_slots(nucleation_MultiMeshResult_slots));
    opaque
        .def_static("create", std::move(maybe_op_unwrap(&nucleation::MultiMeshResult::create)), "schematic"_a, "pack"_a, "config"_a)
        .def("get_mesh", std::move(maybe_op_unwrap(&nucleation::MultiMeshResult::get_mesh)), "region_name"_a)
        .def("mesh_count", &nucleation::MultiMeshResult::mesh_count)
        .def("region_names_json", &nucleation::MultiMeshResult::region_names_json)
        .def("total_triangle_count", &nucleation::MultiMeshResult::total_triangle_count)
        .def("total_vertex_count", &nucleation::MultiMeshResult::total_vertex_count);
}

} 