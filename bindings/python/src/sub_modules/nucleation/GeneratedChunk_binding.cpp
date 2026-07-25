#include "diplomat_nanobind_common.hpp"


#include "GeneratedChunk.hpp"

namespace nucleation {
void add_GeneratedChunk_binding(nb::module_ mod) {
    PyType_Slot nucleation_GeneratedChunk_slots[] = {
        {Py_tp_free, (void *)nucleation::GeneratedChunk::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};

    nb::class_<nucleation::GeneratedChunk> opaque(mod, "GeneratedChunk", nb::type_slots(nucleation_GeneratedChunk_slots));
    opaque
        .def("coverage", &nucleation::GeneratedChunk::coverage)
        .def("cx", &nucleation::GeneratedChunk::cx)
        .def("cz", &nucleation::GeneratedChunk::cz)
        .def("source_id", &nucleation::GeneratedChunk::source_id)
        .def("take_view", std::move(maybe_op_unwrap(&nucleation::GeneratedChunk::take_view)))
        .def("version", &nucleation::GeneratedChunk::version);
}

}
