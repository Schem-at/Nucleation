#include "diplomat_nanobind_common.hpp"


#include "OutputCondition.hpp"
#include "Value.hpp"

namespace nucleation {
void add_OutputCondition_binding(nb::module_ mod) {
    PyType_Slot nucleation_OutputCondition_slots[] = {
        {Py_tp_free, (void *)nucleation::OutputCondition::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};
    
    nb::class_<nucleation::OutputCondition> opaque(mod, "OutputCondition", nb::type_slots(nucleation_OutputCondition_slots));
    opaque
        .def_static("bitwise_and", std::move(maybe_op_unwrap(&nucleation::OutputCondition::bitwise_and)), "mask"_a)
        .def_static("equals", std::move(maybe_op_unwrap(&nucleation::OutputCondition::equals)), "value"_a)
        .def_static("greater_than", std::move(maybe_op_unwrap(&nucleation::OutputCondition::greater_than)), "value"_a)
        .def_static("less_than", std::move(maybe_op_unwrap(&nucleation::OutputCondition::less_than)), "value"_a)
        .def_static("not_equals", std::move(maybe_op_unwrap(&nucleation::OutputCondition::not_equals)), "value"_a);
}

} 