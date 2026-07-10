#include "diplomat_nanobind_common.hpp"


#include "Value.hpp"

namespace nucleation {
void add_Value_binding(nb::module_ mod) {
    PyType_Slot nucleation_Value_slots[] = {
        {Py_tp_free, (void *)nucleation::Value::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};
    
    nb::class_<nucleation::Value> opaque(mod, "Value", nb::type_slots(nucleation_Value_slots));
    opaque
        .def("as_bool", &nucleation::Value::as_bool)
        .def("as_f32", &nucleation::Value::as_f32)
        .def("as_i32", &nucleation::Value::as_i32)
        .def("as_string", &nucleation::Value::as_string)
        .def("as_u32", &nucleation::Value::as_u32)
        .def_static("from_bool", std::move(maybe_op_unwrap(&nucleation::Value::from_bool)), "v"_a)
        .def_static("from_f32", std::move(maybe_op_unwrap(&nucleation::Value::from_f32)), "v"_a)
        .def_static("from_i32", std::move(maybe_op_unwrap(&nucleation::Value::from_i32)), "v"_a)
        .def_static("from_string", std::move(maybe_op_unwrap(&nucleation::Value::from_string)), "s"_a)
        .def_static("from_u32", std::move(maybe_op_unwrap(&nucleation::Value::from_u32)), "v"_a)
        .def("type_name", &nucleation::Value::type_name);
}

} 