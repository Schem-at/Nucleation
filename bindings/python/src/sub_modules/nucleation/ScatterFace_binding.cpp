#include "diplomat_nanobind_common.hpp"


#include "ScatterFace.hpp"

namespace nucleation {
void add_ScatterFace_binding(nb::module_ mod) {
    nb::class_<nucleation::ScatterFace> e_class(mod, "ScatterFace");

        nb::enum_<nucleation::ScatterFace::Value> enumerator(e_class, "ScatterFace");
        enumerator
            .value("Top", nucleation::ScatterFace::Top)
            .value("Bottom", nucleation::ScatterFace::Bottom)
            .value("North", nucleation::ScatterFace::North)
            .value("South", nucleation::ScatterFace::South)
            .value("East", nucleation::ScatterFace::East)
            .value("West", nucleation::ScatterFace::West)
            .export_values();

        e_class
            .def(nb::init_implicit<nucleation::ScatterFace::Value>())
            .def(nb::self == nucleation::ScatterFace::Value())
            .def("__repr__", [](const nucleation::ScatterFace& self){
                return nb::str(nb::cast(nucleation::ScatterFace::Value(self)));
            });
}

}
