#include "diplomat_nanobind_common.hpp"


#include "InterpolationSpace.hpp"

namespace nucleation {
void add_InterpolationSpace_binding(nb::module_ mod) {
    nb::class_<nucleation::InterpolationSpace> e_class(mod, "InterpolationSpace");
    
        nb::enum_<nucleation::InterpolationSpace::Value> enumerator(e_class, "InterpolationSpace");
        enumerator
            .value("Rgb", nucleation::InterpolationSpace::Rgb)
            .value("Oklab", nucleation::InterpolationSpace::Oklab)
            .export_values();
    
        e_class
            .def(nb::init_implicit<nucleation::InterpolationSpace::Value>())
            .def(nb::self == nucleation::InterpolationSpace::Value())
            .def("__repr__", [](const nucleation::InterpolationSpace& self){
                return nb::str(nb::cast(nucleation::InterpolationSpace::Value(self)));
            });
}

} 