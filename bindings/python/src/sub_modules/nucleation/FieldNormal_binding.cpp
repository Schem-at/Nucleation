#include "diplomat_nanobind_common.hpp"


#include "FieldNormal.hpp"

namespace nucleation {
void add_FieldNormal_binding(nb::module_ mod) {
    nb::class_<nucleation::FieldNormal> st(mod, "FieldNormal");
    st
        .def(nb::init<>())
        .def(nb::init<float, float, float>(), "x"_a.none(),  "y"_a.none(),  "z"_a.none())
        .def_rw("x", &nucleation::FieldNormal::x)
        .def_rw("y", &nucleation::FieldNormal::y)
        .def_rw("z", &nucleation::FieldNormal::z);
}

}
