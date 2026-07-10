#include "diplomat_nanobind_common.hpp"


#include "Sdf.hpp"

namespace nucleation {
void add_Sdf_binding(nb::module_ mod) {
    PyType_Slot nucleation_Sdf_slots[] = {
        {Py_tp_free, (void *)nucleation::Sdf::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};
    
    nb::class_<nucleation::Sdf> opaque(mod, "Sdf", nb::type_slots(nucleation_Sdf_slots));
    opaque
        .def_static("eval", &nucleation::Sdf::eval, "sdf_json"_a, "x"_a, "y"_a, "z"_a)
        .def_static("schematic_from_sdf", std::move(maybe_op_unwrap(&nucleation::Sdf::schematic_from_sdf)), "sdf_json"_a, "rules_json"_a, "has_bounds"_a, "min_x"_a, "min_y"_a, "min_z"_a, "max_x"_a, "max_y"_a, "max_z"_a);
}

} 