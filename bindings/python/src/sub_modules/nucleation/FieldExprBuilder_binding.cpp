#include "diplomat_nanobind_common.hpp"


#include "Field.hpp"
#include "FieldDistanceKind.hpp"
#include "FieldExprBuilder.hpp"

namespace nucleation {
void add_FieldExprBuilder_binding(nb::module_ mod) {
    PyType_Slot nucleation_FieldExprBuilder_slots[] = {
        {Py_tp_free, (void *)nucleation::FieldExprBuilder::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};

    nb::class_<nucleation::FieldExprBuilder> opaque(mod, "FieldExprBuilder", nb::type_slots(nucleation_FieldExprBuilder_slots));
    opaque
        .def("bind", &nucleation::FieldExprBuilder::bind, "name"_a, "field"_a)
        .def("compile", std::move(maybe_op_unwrap(&nucleation::FieldExprBuilder::compile)), "source"_a)
        .def_static("create", std::move(maybe_op_unwrap(&nucleation::FieldExprBuilder::create)))
        .def("set_bounds", &nucleation::FieldExprBuilder::set_bounds, "min_x"_a, "min_y"_a, "min_z"_a, "max_x"_a, "max_y"_a, "max_z"_a)
        .def("set_distance_kind", &nucleation::FieldExprBuilder::set_distance_kind, "kind"_a);
}

}
