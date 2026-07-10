#include "diplomat_nanobind_common.hpp"


#include "IoType.hpp"

namespace nucleation {
void add_IoType_binding(nb::module_ mod) {
    PyType_Slot nucleation_IoType_slots[] = {
        {Py_tp_free, (void *)nucleation::IoType::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};
    
    nb::class_<nucleation::IoType> opaque(mod, "IoType", nb::type_slots(nucleation_IoType_slots));
    opaque
        .def_static("ascii", std::move(maybe_op_unwrap(&nucleation::IoType::ascii)), "chars"_a)
        .def_static("boolean", std::move(maybe_op_unwrap(&nucleation::IoType::boolean)))
        .def_static("float32", std::move(maybe_op_unwrap(&nucleation::IoType::float32)))
        .def_static("signed_int", std::move(maybe_op_unwrap(&nucleation::IoType::signed_int)), "bits"_a)
        .def_static("unsigned_int", std::move(maybe_op_unwrap(&nucleation::IoType::unsigned_int)), "bits"_a);
}

} 