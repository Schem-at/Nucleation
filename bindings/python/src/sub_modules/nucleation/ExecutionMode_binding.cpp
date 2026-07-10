#include "diplomat_nanobind_common.hpp"


#include "ExecutionMode.hpp"
#include "OutputCondition.hpp"

namespace nucleation {
void add_ExecutionMode_binding(nb::module_ mod) {
    PyType_Slot nucleation_ExecutionMode_slots[] = {
        {Py_tp_free, (void *)nucleation::ExecutionMode::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};
    
    nb::class_<nucleation::ExecutionMode> opaque(mod, "ExecutionMode", nb::type_slots(nucleation_ExecutionMode_slots));
    opaque
        .def_static("fixed_ticks", std::move(maybe_op_unwrap(&nucleation::ExecutionMode::fixed_ticks)), "ticks"_a)
        .def_static("until_change", std::move(maybe_op_unwrap(&nucleation::ExecutionMode::until_change)), "max_ticks"_a, "check_interval"_a)
        .def_static("until_condition", std::move(maybe_op_unwrap(&nucleation::ExecutionMode::until_condition)), "output_name"_a, "condition"_a, "max_ticks"_a, "check_interval"_a)
        .def_static("until_stable", std::move(maybe_op_unwrap(&nucleation::ExecutionMode::until_stable)), "stable_ticks"_a, "max_ticks"_a);
}

} 