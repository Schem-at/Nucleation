#include "diplomat_nanobind_common.hpp"


#include "TickSettleMode.hpp"

namespace nucleation {
void add_TickSettleMode_binding(nb::module_ mod) {
    nb::class_<nucleation::TickSettleMode> e_class(mod, "TickSettleMode");

        nb::enum_<nucleation::TickSettleMode::Value> enumerator(e_class, "TickSettleMode");
        enumerator
            .value("Placement", nucleation::TickSettleMode::Placement)
            .value("Quiet", nucleation::TickSettleMode::Quiet)
            .value("InWorld", nucleation::TickSettleMode::InWorld)
            .export_values();

        e_class
            .def(nb::init_implicit<nucleation::TickSettleMode::Value>())
            .def(nb::self == nucleation::TickSettleMode::Value())
            .def("__repr__", [](const nucleation::TickSettleMode& self){
                return nb::str(nb::cast(nucleation::TickSettleMode::Value(self)));
            });
}

}
