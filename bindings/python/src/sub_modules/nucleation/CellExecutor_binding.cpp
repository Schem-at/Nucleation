#include "diplomat_nanobind_common.hpp"


#include "CellExecutor.hpp"
#include "Schematic.hpp"
#include "Value.hpp"

namespace nucleation {
void add_CellExecutor_binding(nb::module_ mod) {
    PyType_Slot nucleation_CellExecutor_slots[] = {
        {Py_tp_free, (void *)nucleation::CellExecutor::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};

    nb::class_<nucleation::CellExecutor> opaque(mod, "CellExecutor", nb::type_slots(nucleation_CellExecutor_slots));
    opaque
        .def_static("for_schematic", std::move(maybe_op_unwrap(&nucleation::CellExecutor::for_schematic)), "schematic"_a)
        .def("read_output", std::move(maybe_op_unwrap(&nucleation::CellExecutor::read_output)), "name"_a)
        .def("reset", &nucleation::CellExecutor::reset)
        .def("set_input", &nucleation::CellExecutor::set_input, "name"_a, "value"_a)
        .def("settle", &nucleation::CellExecutor::settle, "budget"_a);
}

}
