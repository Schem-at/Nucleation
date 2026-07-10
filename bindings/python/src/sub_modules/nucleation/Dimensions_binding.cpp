#include "diplomat_nanobind_common.hpp"


#include "Dimensions.hpp"
NB_MAKE_OPAQUE(std::vector<nucleation::Dimensions>)

namespace nucleation {
void add_Dimensions_binding(nb::module_ mod) {
    
    // Python lists are represented as PyObject**, which runs somewhat counter to any use cases where we want to be able to transparently pass over lists without copying over memory in any ways.
    // bind_vector solves this issue by exposing std::vector<nucleation::Dimensions> as a type that will exist inside of C++, with functions to access its memory from Python.
    // TL;DR: this creates a faux list type that makes it easier to pass vectors of this type in Python without copying. 
    nb::bind_vector<std::vector<nucleation::Dimensions>>(mod, "DimensionsSlice"); 
    nb::class_<nucleation::Dimensions> st(mod, "Dimensions");
    st
        .def(nb::init<>())
        .def(nb::init<int32_t, int32_t, int32_t>(), "x"_a.none(),  "y"_a.none(),  "z"_a.none())
        .def_rw("x", &nucleation::Dimensions::x)
        .def_rw("y", &nucleation::Dimensions::y)
        .def_rw("z", &nucleation::Dimensions::z);
}

} 