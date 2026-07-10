#include "diplomat_nanobind_common.hpp"


#include "MeshBounds.hpp"

namespace nucleation {
void add_MeshBounds_binding(nb::module_ mod) {
    nb::class_<nucleation::MeshBounds> st(mod, "MeshBounds");
    st
        .def(nb::init<>())
        .def(nb::init<float, float, float, float, float, float>(), "min_x"_a.none(),  "min_y"_a.none(),  "min_z"_a.none(),  "max_x"_a.none(),  "max_y"_a.none(),  "max_z"_a.none())
        .def_rw("min_x", &nucleation::MeshBounds::min_x)
        .def_rw("min_y", &nucleation::MeshBounds::min_y)
        .def_rw("min_z", &nucleation::MeshBounds::min_z)
        .def_rw("max_x", &nucleation::MeshBounds::max_x)
        .def_rw("max_y", &nucleation::MeshBounds::max_y)
        .def_rw("max_z", &nucleation::MeshBounds::max_z);
}

} 