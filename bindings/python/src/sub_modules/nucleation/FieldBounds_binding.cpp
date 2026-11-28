#include "diplomat_nanobind_common.hpp"


#include "FieldBounds.hpp"

namespace nucleation {
void add_FieldBounds_binding(nb::module_ mod) {
    nb::class_<nucleation::FieldBounds> st(mod, "FieldBounds");
    st
        .def(nb::init<>())
        .def(nb::init<float, float, float, float, float, float>(), "min_x"_a.none(),  "min_y"_a.none(),  "min_z"_a.none(),  "max_x"_a.none(),  "max_y"_a.none(),  "max_z"_a.none())
        .def_rw("min_x", &nucleation::FieldBounds::min_x)
        .def_rw("min_y", &nucleation::FieldBounds::min_y)
        .def_rw("min_z", &nucleation::FieldBounds::min_z)
        .def_rw("max_x", &nucleation::FieldBounds::max_x)
        .def_rw("max_y", &nucleation::FieldBounds::max_y)
        .def_rw("max_z", &nucleation::FieldBounds::max_z);
}

}
