#include "diplomat_nanobind_common.hpp"


#include "SdfNormal.hpp"

namespace nucleation {
void add_SdfNormal_binding(nb::module_ mod) {
    nb::class_<nucleation::SdfNormal> st(mod, "SdfNormal");
    st
        .def(nb::init<>())
        .def(nb::init<float, float, float>(), "x"_a.none(),  "y"_a.none(),  "z"_a.none())
        .def_rw("x", &nucleation::SdfNormal::x)
        .def_rw("y", &nucleation::SdfNormal::y)
        .def_rw("z", &nucleation::SdfNormal::z);
}

}
