#include "diplomat_nanobind_common.hpp"


#include "FieldCellMode.hpp"

namespace nucleation {
void add_FieldCellMode_binding(nb::module_ mod) {
    nb::class_<nucleation::FieldCellMode> e_class(mod, "FieldCellMode");

        nb::enum_<nucleation::FieldCellMode::Value> enumerator(e_class, "FieldCellMode");
        enumerator
            .value("F1", nucleation::FieldCellMode::F1)
            .value("F2", nucleation::FieldCellMode::F2)
            .value("F2MinusF1", nucleation::FieldCellMode::F2MinusF1)
            .value("CellValue", nucleation::FieldCellMode::CellValue)
            .export_values();

        e_class
            .def(nb::init_implicit<nucleation::FieldCellMode::Value>())
            .def(nb::self == nucleation::FieldCellMode::Value())
            .def("__repr__", [](const nucleation::FieldCellMode& self){
                return nb::str(nb::cast(nucleation::FieldCellMode::Value(self)));
            });
}

}
