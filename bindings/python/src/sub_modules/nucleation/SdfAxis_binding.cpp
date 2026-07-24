#include "diplomat_nanobind_common.hpp"


#include "SdfAxis.hpp"

namespace nucleation {
void add_SdfAxis_binding(nb::module_ mod) {
    nb::class_<nucleation::SdfAxis> e_class(mod, "SdfAxis");

        nb::enum_<nucleation::SdfAxis::Value> enumerator(e_class, "SdfAxis");
        enumerator
            .value("X", nucleation::SdfAxis::X)
            .value("Y", nucleation::SdfAxis::Y)
            .value("Z", nucleation::SdfAxis::Z)
            .export_values();

        e_class
            .def(nb::init_implicit<nucleation::SdfAxis::Value>())
            .def(nb::self == nucleation::SdfAxis::Value())
            .def("__repr__", [](const nucleation::SdfAxis& self){
                return nb::str(nb::cast(nucleation::SdfAxis::Value(self)));
            });
}

}
