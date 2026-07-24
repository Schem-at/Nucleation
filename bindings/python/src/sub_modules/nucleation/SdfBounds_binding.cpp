#include "diplomat_nanobind_common.hpp"


#include "SdfBounds.hpp"

namespace nucleation {
void add_SdfBounds_binding(nb::module_ mod) {
    nb::class_<nucleation::SdfBounds> st(mod, "SdfBounds");
    st
        .def(nb::init<>())
        .def(nb::init<float, float, float, float, float, float>(), "min_x"_a.none(),  "min_y"_a.none(),  "min_z"_a.none(),  "max_x"_a.none(),  "max_y"_a.none(),  "max_z"_a.none())
        .def_rw("min_x", &nucleation::SdfBounds::min_x)
        .def_rw("min_y", &nucleation::SdfBounds::min_y)
        .def_rw("min_z", &nucleation::SdfBounds::min_z)
        .def_rw("max_x", &nucleation::SdfBounds::max_x)
        .def_rw("max_y", &nucleation::SdfBounds::max_y)
        .def_rw("max_z", &nucleation::SdfBounds::max_z);
}

}
