#include "diplomat_nanobind_common.hpp"


#include "FieldProgramValueType.hpp"

namespace nucleation {
void add_FieldProgramValueType_binding(nb::module_ mod) {
    nb::class_<nucleation::FieldProgramValueType> e_class(mod, "FieldProgramValueType");

        nb::enum_<nucleation::FieldProgramValueType::Value> enumerator(e_class, "FieldProgramValueType");
        enumerator
            .value("Scalar", nucleation::FieldProgramValueType::Scalar)
            .value("Vec3", nucleation::FieldProgramValueType::Vec3)
            .value("Bool", nucleation::FieldProgramValueType::Bool)
            .export_values();

        e_class
            .def(nb::init_implicit<nucleation::FieldProgramValueType::Value>())
            .def(nb::self == nucleation::FieldProgramValueType::Value())
            .def("__repr__", [](const nucleation::FieldProgramValueType& self){
                return nb::str(nb::cast(nucleation::FieldProgramValueType::Value(self)));
            });
}

}
