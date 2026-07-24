#include "diplomat_nanobind_common.hpp"


#include "FieldProgramUnaryOp.hpp"

namespace nucleation {
void add_FieldProgramUnaryOp_binding(nb::module_ mod) {
    nb::class_<nucleation::FieldProgramUnaryOp> e_class(mod, "FieldProgramUnaryOp");

        nb::enum_<nucleation::FieldProgramUnaryOp::Value> enumerator(e_class, "FieldProgramUnaryOp");
        enumerator
            .value("Neg", nucleation::FieldProgramUnaryOp::Neg)
            .value("Abs", nucleation::FieldProgramUnaryOp::Abs)
            .value("Sqrt", nucleation::FieldProgramUnaryOp::Sqrt)
            .value("Log", nucleation::FieldProgramUnaryOp::Log)
            .value("Sin", nucleation::FieldProgramUnaryOp::Sin)
            .value("Cos", nucleation::FieldProgramUnaryOp::Cos)
            .value("Acos", nucleation::FieldProgramUnaryOp::Acos)
            .value("VecX", nucleation::FieldProgramUnaryOp::VecX)
            .value("VecY", nucleation::FieldProgramUnaryOp::VecY)
            .value("VecZ", nucleation::FieldProgramUnaryOp::VecZ)
            .value("Length", nucleation::FieldProgramUnaryOp::Length)
            .value("Normalize", nucleation::FieldProgramUnaryOp::Normalize)
            .export_values();

        e_class
            .def(nb::init_implicit<nucleation::FieldProgramUnaryOp::Value>())
            .def(nb::self == nucleation::FieldProgramUnaryOp::Value())
            .def("__repr__", [](const nucleation::FieldProgramUnaryOp& self){
                return nb::str(nb::cast(nucleation::FieldProgramUnaryOp::Value(self)));
            });
}

}
