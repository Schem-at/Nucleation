#include "diplomat_nanobind_common.hpp"


#include "Autostack.hpp"
#include "Schematic.hpp"

namespace nucleation {
void add_Autostack_binding(nb::module_ mod) {
    PyType_Slot nucleation_Autostack_slots[] = {
        {Py_tp_free, (void *)nucleation::Autostack::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};
    
    nb::class_<nucleation::Autostack> opaque(mod, "Autostack", nb::type_slots(nucleation_Autostack_slots));
    opaque
        .def_static("detect_structures", &nucleation::Autostack::detect_structures, "schematic"_a)
        .def_static("detect_structures_graph", &nucleation::Autostack::detect_structures_graph, "schematic"_a)
        .def_static("resize_1d", std::move(maybe_op_unwrap(&nucleation::Autostack::resize_1d)), "schematic"_a, "vx"_a, "vy"_a, "vz"_a, "units"_a)
        .def_static("resize_2d", std::move(maybe_op_unwrap(&nucleation::Autostack::resize_2d)), "schematic"_a, "v1x"_a, "v1y"_a, "v1z"_a, "v2x"_a, "v2y"_a, "v2z"_a, "n1"_a, "n2"_a);
}

} 