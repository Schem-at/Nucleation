#include "diplomat_nanobind_common.hpp"


#include "MeshConfig.hpp"
#include "RawMeshExport.hpp"
#include "ResourcePack.hpp"
#include "Schematic.hpp"

namespace nucleation {
void add_RawMeshExport_binding(nb::module_ mod) {
    PyType_Slot nucleation_RawMeshExport_slots[] = {
        {Py_tp_free, (void *)nucleation::RawMeshExport::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};
    
    nb::class_<nucleation::RawMeshExport> opaque(mod, "RawMeshExport", nb::type_slots(nucleation_RawMeshExport_slots));
    opaque
        .def("colors_b64", &nucleation::RawMeshExport::colors_b64)
        .def_static("create", std::move(maybe_op_unwrap(&nucleation::RawMeshExport::create)), "schematic"_a, "pack"_a, "config"_a)
        .def("indices_b64", &nucleation::RawMeshExport::indices_b64)
        .def("normals_b64", &nucleation::RawMeshExport::normals_b64)
        .def("positions_b64", &nucleation::RawMeshExport::positions_b64)
        .def("texture_height", &nucleation::RawMeshExport::texture_height)
        .def("texture_rgba_b64", &nucleation::RawMeshExport::texture_rgba_b64)
        .def("texture_width", &nucleation::RawMeshExport::texture_width)
        .def("triangle_count", &nucleation::RawMeshExport::triangle_count)
        .def("uvs_b64", &nucleation::RawMeshExport::uvs_b64)
        .def("vertex_count", &nucleation::RawMeshExport::vertex_count);
}

} 