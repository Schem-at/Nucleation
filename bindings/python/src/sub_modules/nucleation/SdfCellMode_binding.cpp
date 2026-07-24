#include "diplomat_nanobind_common.hpp"


#include "SdfCellMode.hpp"

namespace nucleation {
void add_SdfCellMode_binding(nb::module_ mod) {
    nb::class_<nucleation::SdfCellMode> e_class(mod, "SdfCellMode");

        nb::enum_<nucleation::SdfCellMode::Value> enumerator(e_class, "SdfCellMode");
        enumerator
            .value("F1", nucleation::SdfCellMode::F1)
            .value("F2", nucleation::SdfCellMode::F2)
            .value("F2MinusF1", nucleation::SdfCellMode::F2MinusF1)
            .value("CellValue", nucleation::SdfCellMode::CellValue)
            .export_values();

        e_class
            .def(nb::init_implicit<nucleation::SdfCellMode::Value>())
            .def(nb::self == nucleation::SdfCellMode::Value())
            .def("__repr__", [](const nucleation::SdfCellMode& self){
                return nb::str(nb::cast(nucleation::SdfCellMode::Value(self)));
            });
}

}
