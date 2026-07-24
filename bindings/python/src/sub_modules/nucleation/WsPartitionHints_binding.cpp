#include "diplomat_nanobind_common.hpp"


#include "WsPartitionHints.hpp"

namespace nucleation {
void add_WsPartitionHints_binding(nb::module_ mod) {
    PyType_Slot nucleation_WsPartitionHints_slots[] = {
        {Py_tp_free, (void *)nucleation::WsPartitionHints::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};
    
    nb::class_<nucleation::WsPartitionHints> opaque(mod, "WsPartitionHints", nb::type_slots(nucleation_WsPartitionHints_slots));
    opaque
        .def("add", &nucleation::WsPartitionHints::add, "id"_a, "x0"_a, "x1"_a, "z0"_a, "z1"_a)
        .def_static("create", std::move(maybe_op_unwrap(&nucleation::WsPartitionHints::create)))
        .def("len", &nucleation::WsPartitionHints::len);
}

} 