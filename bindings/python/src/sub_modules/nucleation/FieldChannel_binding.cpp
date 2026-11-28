#include "diplomat_nanobind_common.hpp"


#include "FieldChannel.hpp"

namespace nucleation {
void add_FieldChannel_binding(nb::module_ mod) {
    nb::class_<nucleation::FieldChannel> e_class(mod, "FieldChannel");

        nb::enum_<nucleation::FieldChannel::Value> enumerator(e_class, "FieldChannel");
        enumerator
            .value("WorldX", nucleation::FieldChannel::WorldX)
            .value("WorldY", nucleation::FieldChannel::WorldY)
            .value("WorldZ", nucleation::FieldChannel::WorldZ)
            .value("Distance", nucleation::FieldChannel::Distance)
            .value("DepthBelowSurface", nucleation::FieldChannel::DepthBelowSurface)
            .value("Slope", nucleation::FieldChannel::Slope)
            .value("NormalY", nucleation::FieldChannel::NormalY)
            .value("Curvature", nucleation::FieldChannel::Curvature)
            .export_values();

        e_class
            .def(nb::init_implicit<nucleation::FieldChannel::Value>())
            .def(nb::self == nucleation::FieldChannel::Value())
            .def("__repr__", [](const nucleation::FieldChannel& self){
                return nb::str(nb::cast(nucleation::FieldChannel::Value(self)));
            });
}

}
