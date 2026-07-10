#include "diplomat_nanobind_common.hpp"


#include "MeshConfig.hpp"
#include "MeshJob.hpp"
#include "ResourcePack.hpp"
#include "Schematic.hpp"
#include "TextureAtlas.hpp"

namespace nucleation {
void add_MeshJob_binding(nb::module_ mod) {
    PyType_Slot nucleation_MeshJob_slots[] = {
        {Py_tp_free, (void *)nucleation::MeshJob::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};
    
    nb::class_<nucleation::MeshJob> opaque(mod, "MeshJob", nb::type_slots(nucleation_MeshJob_slots));
    opaque
        .def("poll_progress", &nucleation::MeshJob::poll_progress)
        .def_static("start", std::move(maybe_op_unwrap(&nucleation::MeshJob::start)), "schematic"_a, "pack"_a, "config"_a, "chunk_size"_a, "atlas"_a)
        .def("take_result", std::move(maybe_op_unwrap(&nucleation::MeshJob::take_result)));
}

} 