#include "diplomat_nanobind_common.hpp"


#include "GeneratedChunkCoverage.hpp"

namespace nucleation {
void add_GeneratedChunkCoverage_binding(nb::module_ mod) {
    nb::class_<nucleation::GeneratedChunkCoverage> e_class(mod, "GeneratedChunkCoverage");

        nb::enum_<nucleation::GeneratedChunkCoverage::Value> enumerator(e_class, "GeneratedChunkCoverage");
        enumerator
            .value("Complete", nucleation::GeneratedChunkCoverage::Complete)
            .value("Partial", nucleation::GeneratedChunkCoverage::Partial)
            .value("Outside", nucleation::GeneratedChunkCoverage::Outside)
            .export_values();

        e_class
            .def(nb::init_implicit<nucleation::GeneratedChunkCoverage::Value>())
            .def(nb::self == nucleation::GeneratedChunkCoverage::Value())
            .def("__repr__", [](const nucleation::GeneratedChunkCoverage& self){
                return nb::str(nb::cast(nucleation::GeneratedChunkCoverage::Value(self)));
            });
}

}
