#include "diplomat_nanobind_common.hpp"


#include "FieldProgramBinaryOp.hpp"

namespace nucleation {
void add_FieldProgramBinaryOp_binding(nb::module_ mod) {
    nb::class_<nucleation::FieldProgramBinaryOp> e_class(mod, "FieldProgramBinaryOp");

        nb::enum_<nucleation::FieldProgramBinaryOp::Value> enumerator(e_class, "FieldProgramBinaryOp");
        enumerator
            .value("Add", nucleation::FieldProgramBinaryOp::Add)
            .value("Sub", nucleation::FieldProgramBinaryOp::Sub)
            .value("Mul", nucleation::FieldProgramBinaryOp::Mul)
            .value("Div", nucleation::FieldProgramBinaryOp::Div)
            .value("Min", nucleation::FieldProgramBinaryOp::Min)
            .value("Max", nucleation::FieldProgramBinaryOp::Max)
            .value("Pow", nucleation::FieldProgramBinaryOp::Pow)
            .value("Atan2", nucleation::FieldProgramBinaryOp::Atan2)
            .value("Lt", nucleation::FieldProgramBinaryOp::Lt)
            .value("Le", nucleation::FieldProgramBinaryOp::Le)
            .value("Gt", nucleation::FieldProgramBinaryOp::Gt)
            .value("Ge", nucleation::FieldProgramBinaryOp::Ge)
            .value("Eq", nucleation::FieldProgramBinaryOp::Eq)
            .value("Dot", nucleation::FieldProgramBinaryOp::Dot)
            .value("Cross", nucleation::FieldProgramBinaryOp::Cross)
            .value("Scale", nucleation::FieldProgramBinaryOp::Scale)
            .export_values();

        e_class
            .def(nb::init_implicit<nucleation::FieldProgramBinaryOp::Value>())
            .def(nb::self == nucleation::FieldProgramBinaryOp::Value())
            .def("__repr__", [](const nucleation::FieldProgramBinaryOp& self){
                return nb::str(nb::cast(nucleation::FieldProgramBinaryOp::Value(self)));
            });
}

}
