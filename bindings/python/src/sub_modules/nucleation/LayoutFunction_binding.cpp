#include "diplomat_nanobind_common.hpp"


#include "LayoutFunction.hpp"

namespace nucleation {
void add_LayoutFunction_binding(nb::module_ mod) {
    PyType_Slot nucleation_LayoutFunction_slots[] = {
        {Py_tp_free, (void *)nucleation::LayoutFunction::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};
    
    nb::class_<nucleation::LayoutFunction> opaque(mod, "LayoutFunction", nb::type_slots(nucleation_LayoutFunction_slots));
    opaque
        .def_static("column_major", std::move(maybe_op_unwrap(&nucleation::LayoutFunction::column_major)), "rows"_a, "cols"_a, "bits_per_element"_a)
        .def_static("custom", std::move(maybe_op_unwrap(&nucleation::LayoutFunction::custom)), "mapping"_a)
        .def_static("one_to_one", std::move(maybe_op_unwrap(&nucleation::LayoutFunction::one_to_one)))
        .def_static("packed4", std::move(maybe_op_unwrap(&nucleation::LayoutFunction::packed4)))
        .def_static("row_major", std::move(maybe_op_unwrap(&nucleation::LayoutFunction::row_major)), "rows"_a, "cols"_a, "bits_per_element"_a)
        .def_static("scanline", std::move(maybe_op_unwrap(&nucleation::LayoutFunction::scanline)), "width"_a, "height"_a, "bits_per_pixel"_a);
}

} 