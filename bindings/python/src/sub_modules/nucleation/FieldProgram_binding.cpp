#include "diplomat_nanobind_common.hpp"


#include "FieldProgram.hpp"

namespace nucleation {
void add_FieldProgram_binding(nb::module_ mod) {
    PyType_Slot nucleation_FieldProgram_slots[] = {
        {Py_tp_free, (void *)nucleation::FieldProgram::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};

    nb::class_<nucleation::FieldProgram> opaque(mod, "FieldProgram", nb::type_slots(nucleation_FieldProgram_slots));
    opaque
        .def("bounds", &nucleation::FieldProgram::bounds)
        .def("distance_kind", &nucleation::FieldProgram::distance_kind)
        .def("eval_at", &nucleation::FieldProgram::eval_at, "x"_a, "y"_a, "z"_a)
        .def_static("from_json_string", std::move(maybe_op_unwrap(&nucleation::FieldProgram::from_json_string)), "json"_a)
        .def("gradient", &nucleation::FieldProgram::gradient, "x"_a, "y"_a, "z"_a, "epsilon"_a)
        .def("to_json", &nucleation::FieldProgram::to_json);
}

}
