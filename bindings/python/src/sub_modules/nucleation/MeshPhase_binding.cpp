#include "diplomat_nanobind_common.hpp"


#include "MeshPhase.hpp"

namespace nucleation {
void add_MeshPhase_binding(nb::module_ mod) {
    nb::class_<nucleation::MeshPhase> e_class(mod, "MeshPhase");
    
        nb::enum_<nucleation::MeshPhase::Value> enumerator(e_class, "MeshPhase");
        enumerator
            .value("BuildingAtlas", nucleation::MeshPhase::BuildingAtlas)
            .value("MeshingChunks", nucleation::MeshPhase::MeshingChunks)
            .value("Complete", nucleation::MeshPhase::Complete)
            .value("Failed", nucleation::MeshPhase::Failed)
            .export_values();
    
        e_class
            .def(nb::init_implicit<nucleation::MeshPhase::Value>())
            .def(nb::self == nucleation::MeshPhase::Value())
            .def("__repr__", [](const nucleation::MeshPhase& self){
                return nb::str(nb::cast(nucleation::MeshPhase::Value(self)));
            });
}

} 