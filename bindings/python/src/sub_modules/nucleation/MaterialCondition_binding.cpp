#include "diplomat_nanobind_common.hpp"


#include "Field.hpp"
#include "FieldChannel.hpp"
#include "MaterialCondition.hpp"

namespace nucleation {
void add_MaterialCondition_binding(nb::module_ mod) {
    PyType_Slot nucleation_MaterialCondition_slots[] = {
        {Py_tp_free, (void *)nucleation::MaterialCondition::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};

    nb::class_<nucleation::MaterialCondition> opaque(mod, "MaterialCondition", nb::type_slots(nucleation_MaterialCondition_slots));
    opaque
        .def_static("always", std::move(maybe_op_unwrap(&nucleation::MaterialCondition::always)))
        .def("and_with", std::move(maybe_op_unwrap(&nucleation::MaterialCondition::and_with)), "other"_a)
        .def_static("chance", std::move(maybe_op_unwrap(&nucleation::MaterialCondition::chance)), "probability"_a, "seed"_a)
        .def_static("channel_range", std::move(maybe_op_unwrap(&nucleation::MaterialCondition::channel_range)), "channel"_a, "has_min"_a, "min"_a, "has_max"_a, "max"_a)
        .def_static("curvature_range", std::move(maybe_op_unwrap(&nucleation::MaterialCondition::curvature_range)), "has_min"_a, "min"_a, "has_max"_a, "max"_a)
        .def_static("depth_below_surface", std::move(maybe_op_unwrap(&nucleation::MaterialCondition::depth_below_surface)), "has_min"_a, "min"_a, "has_max"_a, "max"_a)
        .def_static("exposed_face", std::move(maybe_op_unwrap(&nucleation::MaterialCondition::exposed_face)), "mask"_a, "require_all"_a)
        .def_static("field_range", std::move(maybe_op_unwrap(&nucleation::MaterialCondition::field_range)), "field"_a, "has_min"_a, "min"_a, "has_max"_a, "max"_a)
        .def("negated", std::move(maybe_op_unwrap(&nucleation::MaterialCondition::negated)))
        .def_static("normal_dot", std::move(maybe_op_unwrap(&nucleation::MaterialCondition::normal_dot)), "dx"_a, "dy"_a, "dz"_a, "has_min"_a, "min"_a, "has_max"_a, "max"_a)
        .def("or_with", std::move(maybe_op_unwrap(&nucleation::MaterialCondition::or_with)), "other"_a)
        .def_static("slope_range", std::move(maybe_op_unwrap(&nucleation::MaterialCondition::slope_range)), "has_min"_a, "min"_a, "has_max"_a, "max"_a)
        .def_static("tag", std::move(maybe_op_unwrap(&nucleation::MaterialCondition::tag)), "material_id"_a)
        .def_static("y_range", std::move(maybe_op_unwrap(&nucleation::MaterialCondition::y_range)), "has_min"_a, "min"_a, "has_max"_a, "max"_a);
}

}
