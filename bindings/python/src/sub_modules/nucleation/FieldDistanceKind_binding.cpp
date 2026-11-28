#include "diplomat_nanobind_common.hpp"


#include "FieldDistanceKind.hpp"

namespace nucleation {
void add_FieldDistanceKind_binding(nb::module_ mod) {
    nb::class_<nucleation::FieldDistanceKind> e_class(mod, "FieldDistanceKind");

        nb::enum_<nucleation::FieldDistanceKind::Value> enumerator(e_class, "FieldDistanceKind");
        enumerator
            .value("Exact", nucleation::FieldDistanceKind::Exact)
            .value("LowerBound", nucleation::FieldDistanceKind::LowerBound)
            .value("Estimate", nucleation::FieldDistanceKind::Estimate)
            .value("Implicit", nucleation::FieldDistanceKind::Implicit)
            .export_values();

        e_class
            .def(nb::init_implicit<nucleation::FieldDistanceKind::Value>())
            .def(nb::self == nucleation::FieldDistanceKind::Value())
            .def("__repr__", [](const nucleation::FieldDistanceKind& self){
                return nb::str(nb::cast(nucleation::FieldDistanceKind::Value(self)));
            });
}

}
