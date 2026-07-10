#include "diplomat_nanobind_common.hpp"


#include "MeshPhase.hpp"
#include "MeshProgress.hpp"

namespace nucleation {
void add_MeshProgress_binding(nb::module_ mod) {
    nb::class_<nucleation::MeshProgress> st(mod, "MeshProgress");
    st
        .def(nb::init<>())
        .def(nb::init<nucleation::MeshPhase, uint32_t, uint32_t>(), "phase"_a.none(),  "current"_a.none(),  "total"_a.none())
        .def_rw("phase", &nucleation::MeshProgress::phase)
        .def_rw("current", &nucleation::MeshProgress::current)
        .def_rw("total", &nucleation::MeshProgress::total);
}

} 