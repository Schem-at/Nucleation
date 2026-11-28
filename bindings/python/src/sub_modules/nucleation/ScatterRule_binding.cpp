#include "diplomat_nanobind_common.hpp"


#include "MaterialCondition.hpp"
#include "ScatterFace.hpp"
#include "ScatterRule.hpp"

namespace nucleation {
void add_ScatterRule_binding(nb::module_ mod) {
    PyType_Slot nucleation_ScatterRule_slots[] = {
        {Py_tp_free, (void *)nucleation::ScatterRule::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};

    nb::class_<nucleation::ScatterRule> opaque(mod, "ScatterRule", nb::type_slots(nucleation_ScatterRule_slots));
    opaque
        .def_static("create", std::move(maybe_op_unwrap(&nucleation::ScatterRule::create)), "face"_a, "density"_a, "blocks_json"_a, "weights"_a, "seed"_a)
        .def("set_condition", &nucleation::ScatterRule::set_condition, "condition"_a)
        .def("set_height_range", &nucleation::ScatterRule::set_height_range, "min"_a, "max"_a)
        .def("set_on_block", &nucleation::ScatterRule::set_on_block, "prefix"_a);
}

}
