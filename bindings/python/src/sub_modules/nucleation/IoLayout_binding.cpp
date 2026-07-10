#include "diplomat_nanobind_common.hpp"


#include "IoLayout.hpp"

namespace nucleation {
void add_IoLayout_binding(nb::module_ mod) {
    PyType_Slot nucleation_IoLayout_slots[] = {
        {Py_tp_free, (void *)nucleation::IoLayout::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};
    
    nb::class_<nucleation::IoLayout> opaque(mod, "IoLayout", nb::type_slots(nucleation_IoLayout_slots));
    opaque
        .def("input_names_json", &nucleation::IoLayout::input_names_json)
        .def("output_names_json", &nucleation::IoLayout::output_names_json);
}

} 