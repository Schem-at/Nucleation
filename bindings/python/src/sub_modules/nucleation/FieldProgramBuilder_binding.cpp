#include "diplomat_nanobind_common.hpp"


#include "FieldProgramBinaryOp.hpp"
#include "FieldProgramBuilder.hpp"
#include "FieldProgramDistanceKind.hpp"
#include "FieldProgramUnaryOp.hpp"
#include "FieldProgramValueType.hpp"

namespace nucleation {
void add_FieldProgramBuilder_binding(nb::module_ mod) {
    PyType_Slot nucleation_FieldProgramBuilder_slots[] = {
        {Py_tp_free, (void *)nucleation::FieldProgramBuilder::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};

    nb::class_<nucleation::FieldProgramBuilder> opaque(mod, "FieldProgramBuilder", nb::type_slots(nucleation_FieldProgramBuilder_slots));
    opaque
        .def("add_slot", &nucleation::FieldProgramBuilder::add_slot, "value_type"_a)
        .def("begin_repeat", &nucleation::FieldProgramBuilder::begin_repeat, "count"_a)
        .def("binary_op", &nucleation::FieldProgramBuilder::binary_op, "op"_a)
        .def("break_if", &nucleation::FieldProgramBuilder::break_if)
        .def("build", std::move(maybe_op_unwrap(&nucleation::FieldProgramBuilder::build)))
        .def("clamp", &nucleation::FieldProgramBuilder::clamp)
        .def_static("create", std::move(maybe_op_unwrap(&nucleation::FieldProgramBuilder::create)))
        .def("end_repeat", &nucleation::FieldProgramBuilder::end_repeat)
        .def("load_local", &nucleation::FieldProgramBuilder::load_local, "slot"_a)
        .def("make_vec3", &nucleation::FieldProgramBuilder::make_vec3)
        .def("pop", &nucleation::FieldProgramBuilder::pop)
        .def("push_const_bool", &nucleation::FieldProgramBuilder::push_const_bool, "value"_a)
        .def("push_const_scalar", &nucleation::FieldProgramBuilder::push_const_scalar, "value"_a)
        .def("push_const_vec3", &nucleation::FieldProgramBuilder::push_const_vec3, "x"_a, "y"_a, "z"_a)
        .def("push_pos", &nucleation::FieldProgramBuilder::push_pos)
        .def("select", &nucleation::FieldProgramBuilder::select)
        .def("set_bounds", &nucleation::FieldProgramBuilder::set_bounds, "min_x"_a, "min_y"_a, "min_z"_a, "max_x"_a, "max_y"_a, "max_z"_a)
        .def("set_distance_kind", &nucleation::FieldProgramBuilder::set_distance_kind, "kind"_a)
        .def("set_output", &nucleation::FieldProgramBuilder::set_output, "slot"_a)
        .def("store_local", &nucleation::FieldProgramBuilder::store_local, "slot"_a)
        .def("unary_op", &nucleation::FieldProgramBuilder::unary_op, "op"_a);
}

}
