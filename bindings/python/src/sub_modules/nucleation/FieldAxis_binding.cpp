#include "diplomat_nanobind_common.hpp"


#include "FieldAxis.hpp"

namespace nucleation {
void add_FieldAxis_binding(nb::module_ mod) {
    nb::class_<nucleation::FieldAxis> e_class(mod, "FieldAxis");

        nb::enum_<nucleation::FieldAxis::Value> enumerator(e_class, "FieldAxis");
        enumerator
            .value("X", nucleation::FieldAxis::X)
            .value("Y", nucleation::FieldAxis::Y)
            .value("Z", nucleation::FieldAxis::Z)
            .export_values();

        e_class
            .def(nb::init_implicit<nucleation::FieldAxis::Value>())
            .def(nb::self == nucleation::FieldAxis::Value())
            .def("__repr__", [](const nucleation::FieldAxis& self){
                return nb::str(nb::cast(nucleation::FieldAxis::Value(self)));
            });
}

}
