#include "diplomat_nanobind_common.hpp"


#include "FieldNoiseBasis.hpp"

namespace nucleation {
void add_FieldNoiseBasis_binding(nb::module_ mod) {
    nb::class_<nucleation::FieldNoiseBasis> e_class(mod, "FieldNoiseBasis");

        nb::enum_<nucleation::FieldNoiseBasis::Value> enumerator(e_class, "FieldNoiseBasis");
        enumerator
            .value("ValueNoise", nucleation::FieldNoiseBasis::ValueNoise)
            .value("Perlin", nucleation::FieldNoiseBasis::Perlin)
            .export_values();

        e_class
            .def(nb::init_implicit<nucleation::FieldNoiseBasis::Value>())
            .def(nb::self == nucleation::FieldNoiseBasis::Value())
            .def("__repr__", [](const nucleation::FieldNoiseBasis& self){
                return nb::str(nb::cast(nucleation::FieldNoiseBasis::Value(self)));
            });
}

}
