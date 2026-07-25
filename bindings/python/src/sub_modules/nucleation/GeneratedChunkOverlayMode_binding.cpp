#include "diplomat_nanobind_common.hpp"


#include "GeneratedChunkOverlayMode.hpp"

namespace nucleation {
void add_GeneratedChunkOverlayMode_binding(nb::module_ mod) {
    nb::class_<nucleation::GeneratedChunkOverlayMode> e_class(mod, "GeneratedChunkOverlayMode");

        nb::enum_<nucleation::GeneratedChunkOverlayMode::Value> enumerator(e_class, "GeneratedChunkOverlayMode");
        enumerator
            .value("Replace", nucleation::GeneratedChunkOverlayMode::Replace)
            .value("KeepExisting", nucleation::GeneratedChunkOverlayMode::KeepExisting)
            .export_values();

        e_class
            .def(nb::init_implicit<nucleation::GeneratedChunkOverlayMode::Value>())
            .def(nb::self == nucleation::GeneratedChunkOverlayMode::Value())
            .def("__repr__", [](const nucleation::GeneratedChunkOverlayMode& self){
                return nb::str(nb::cast(nucleation::GeneratedChunkOverlayMode::Value(self)));
            });
}

}
