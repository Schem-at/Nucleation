#include "diplomat_nanobind_common.hpp"


#include "DistanceField.hpp"
#include "Schematic.hpp"

namespace nucleation {
void add_DistanceField_binding(nb::module_ mod) {
    PyType_Slot nucleation_DistanceField_slots[] = {
        {Py_tp_free, (void *)nucleation::DistanceField::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};
    
    nb::class_<nucleation::DistanceField> opaque(mod, "DistanceField", nb::type_slots(nucleation_DistanceField_slots));
    opaque
        .def("depth", &nucleation::DistanceField::depth, "x"_a, "y"_a, "z"_a)
        .def_static("from_schematic", std::move(maybe_op_unwrap(&nucleation::DistanceField::from_schematic)), "schematic"_a)
        .def("normal_json", &nucleation::DistanceField::normal_json, "x"_a, "y"_a, "z"_a)
        .def("slope", &nucleation::DistanceField::slope, "x"_a, "y"_a, "z"_a);
}

} 