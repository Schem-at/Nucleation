#include "diplomat_nanobind_common.hpp"


#include "FieldProgramDistanceKind.hpp"

namespace nucleation {
void add_FieldProgramDistanceKind_binding(nb::module_ mod) {
    nb::class_<nucleation::FieldProgramDistanceKind> e_class(mod, "FieldProgramDistanceKind");

        nb::enum_<nucleation::FieldProgramDistanceKind::Value> enumerator(e_class, "FieldProgramDistanceKind");
        enumerator
            .value("Exact", nucleation::FieldProgramDistanceKind::Exact)
            .value("LowerBound", nucleation::FieldProgramDistanceKind::LowerBound)
            .value("Estimate", nucleation::FieldProgramDistanceKind::Estimate)
            .value("Implicit", nucleation::FieldProgramDistanceKind::Implicit)
            .export_values();

        e_class
            .def(nb::init_implicit<nucleation::FieldProgramDistanceKind::Value>())
            .def(nb::self == nucleation::FieldProgramDistanceKind::Value())
            .def("__repr__", [](const nucleation::FieldProgramDistanceKind& self){
                return nb::str(nb::cast(nucleation::FieldProgramDistanceKind::Value(self)));
            });
}

}
