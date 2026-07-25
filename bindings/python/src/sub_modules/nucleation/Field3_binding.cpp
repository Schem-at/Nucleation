#include "diplomat_nanobind_common.hpp"


#include "Field3.hpp"

namespace nucleation {
void add_Field3_binding(nb::module_ mod) {
    PyType_Slot nucleation_Field3_slots[] = {
        {Py_tp_free, (void *)nucleation::Field3::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};

    nb::class_<nucleation::Field3> opaque(mod, "Field3", nb::type_slots(nucleation_Field3_slots));
    opaque
        .def("eval_at", &nucleation::Field3::eval_at, "x"_a, "y"_a, "z"_a)
        .def_static("from_json_string", std::move(maybe_op_unwrap(&nucleation::Field3::from_json_string)), "json"_a)
        .def("output_range", &nucleation::Field3::output_range)
        .def("to_json", &nucleation::Field3::to_json)
        .def_static("value_noise_fbm", std::move(maybe_op_unwrap(&nucleation::Field3::value_noise_fbm)), "frequency"_a, "seed"_a, "octaves"_a);
}

}
