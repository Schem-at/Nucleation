#include "diplomat_nanobind_common.hpp"


#include "MeshConfig.hpp"
#include "MeshResult.hpp"
#include "ResourcePack.hpp"
#include "Schematic.hpp"

namespace nucleation {
void add_MeshResult_binding(nb::module_ mod) {
    PyType_Slot nucleation_MeshResult_slots[] = {
        {Py_tp_free, (void *)nucleation::MeshResult::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};
    
    nb::class_<nucleation::MeshResult> opaque(mod, "MeshResult", nb::type_slots(nucleation_MeshResult_slots));
    opaque
        .def("bounds", &nucleation::MeshResult::bounds)
        .def_static("create", std::move(maybe_op_unwrap(&nucleation::MeshResult::create)), "schematic"_a, "pack"_a, "config"_a)
        .def_static("create_usdz", std::move(maybe_op_unwrap(&nucleation::MeshResult::create_usdz)), "schematic"_a, "pack"_a, "config"_a)
        .def("glb_data_b64", &nucleation::MeshResult::glb_data_b64)
        .def("has_transparency", &nucleation::MeshResult::has_transparency)
        .def("nucm_data_b64", &nucleation::MeshResult::nucm_data_b64)
        .def("triangle_count", &nucleation::MeshResult::triangle_count)
        .def("vertex_count", &nucleation::MeshResult::vertex_count);
}

} 