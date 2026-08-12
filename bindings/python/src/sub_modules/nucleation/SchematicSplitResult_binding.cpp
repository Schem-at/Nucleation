#include "diplomat_nanobind_common.hpp"


#include "SchematicSplitResult.hpp"

namespace nucleation {
void add_SchematicSplitResult_binding(nb::module_ mod) {
    PyType_Slot nucleation_SchematicSplitResult_slots[] = {
        {Py_tp_free, (void *)nucleation::SchematicSplitResult::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};

    nb::class_<nucleation::SchematicSplitResult> opaque(mod, "SchematicSplitResult", nb::type_slots(nucleation_SchematicSplitResult_slots));
    opaque
        .def("len", &nucleation::SchematicSplitResult::len)
        .def("piece", std::move(maybe_op_unwrap(&nucleation::SchematicSplitResult::piece)), "index"_a);
}

}
