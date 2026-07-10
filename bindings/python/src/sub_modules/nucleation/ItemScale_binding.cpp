#include "diplomat_nanobind_common.hpp"


#include "ItemScale.hpp"

namespace nucleation {
void add_ItemScale_binding(nb::module_ mod) {
    nb::class_<nucleation::ItemScale> st(mod, "ItemScale");
    st
        .def(nb::init<>())
        .def(nb::init<float, float, float>(), "x"_a.none(),  "y"_a.none(),  "z"_a.none())
        .def_rw("x", &nucleation::ItemScale::x)
        .def_rw("y", &nucleation::ItemScale::y)
        .def_rw("z", &nucleation::ItemScale::z);
}

} 