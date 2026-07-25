#include "diplomat_nanobind_common.hpp"


#include "FieldRange.hpp"

namespace nucleation {
void add_FieldRange_binding(nb::module_ mod) {
    nb::class_<nucleation::FieldRange> st(mod, "FieldRange");
    st
        .def(nb::init<>())
        .def(nb::init<float, float>(), "min"_a.none(),  "max"_a.none())
        .def_rw("min", &nucleation::FieldRange::min)
        .def_rw("max", &nucleation::FieldRange::max);
}

}
