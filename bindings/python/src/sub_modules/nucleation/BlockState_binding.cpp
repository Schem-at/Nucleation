#include "diplomat_nanobind_common.hpp"


#include "BlockState.hpp"

namespace nucleation {
void add_BlockState_binding(nb::module_ mod) {
    PyType_Slot nucleation_BlockState_slots[] = {
        {Py_tp_free, (void *)nucleation::BlockState::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};
    
    nb::class_<nucleation::BlockState> opaque(mod, "BlockState", nb::type_slots(nucleation_BlockState_slots));
    opaque
        .def_static("create", std::move(maybe_op_unwrap(&nucleation::BlockState::create)), "name"_a)
        .def("name", &nucleation::BlockState::name)
        .def("properties_json", &nucleation::BlockState::properties_json)
        .def("with_property", std::move(maybe_op_unwrap(&nucleation::BlockState::with_property)), "key"_a, "value"_a);
}

} 