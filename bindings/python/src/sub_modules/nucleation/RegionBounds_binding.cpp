#include "diplomat_nanobind_common.hpp"


#include "RegionBounds.hpp"
NB_MAKE_OPAQUE(std::vector<nucleation::RegionBounds>)

namespace nucleation {
void add_RegionBounds_binding(nb::module_ mod) {
    
    // Python lists are represented as PyObject**, which runs somewhat counter to any use cases where we want to be able to transparently pass over lists without copying over memory in any ways.
    // bind_vector solves this issue by exposing std::vector<nucleation::RegionBounds> as a type that will exist inside of C++, with functions to access its memory from Python.
    // TL;DR: this creates a faux list type that makes it easier to pass vectors of this type in Python without copying. 
    nb::bind_vector<std::vector<nucleation::RegionBounds>>(mod, "RegionBoundsSlice"); 
    nb::class_<nucleation::RegionBounds> st(mod, "RegionBounds");
    st
        .def(nb::init<>())
        .def(nb::init<int32_t, int32_t, int32_t, int32_t, int32_t, int32_t>(), "min_x"_a.none(),  "min_y"_a.none(),  "min_z"_a.none(),  "max_x"_a.none(),  "max_y"_a.none(),  "max_z"_a.none())
        .def_rw("min_x", &nucleation::RegionBounds::min_x)
        .def_rw("min_y", &nucleation::RegionBounds::min_y)
        .def_rw("min_z", &nucleation::RegionBounds::min_z)
        .def_rw("max_x", &nucleation::RegionBounds::max_x)
        .def_rw("max_y", &nucleation::RegionBounds::max_y)
        .def_rw("max_z", &nucleation::RegionBounds::max_z);
}

} 