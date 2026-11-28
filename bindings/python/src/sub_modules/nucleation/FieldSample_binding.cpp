#include "diplomat_nanobind_common.hpp"


#include "FieldSample.hpp"

namespace nucleation {
void add_FieldSample_binding(nb::module_ mod) {
    nb::class_<nucleation::FieldSample> st(mod, "FieldSample");
    st
        .def(nb::init<>())
        .def(nb::init<float, uint16_t, uint16_t, float>(), "distance"_a.none(),  "material_id"_a.none(),  "blend_material_id"_a.none(),  "blend_weight"_a.none())
        .def_rw("distance", &nucleation::FieldSample::distance)
        .def_rw("material_id", &nucleation::FieldSample::material_id)
        .def_rw("blend_material_id", &nucleation::FieldSample::blend_material_id)
        .def_rw("blend_weight", &nucleation::FieldSample::blend_weight);
}

}
