#include "diplomat_nanobind_common.hpp"


#include "Shape.hpp"

namespace nucleation {
void add_Shape_binding(nb::module_ mod) {
    PyType_Slot nucleation_Shape_slots[] = {
        {Py_tp_free, (void *)nucleation::Shape::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};
    
    nb::class_<nucleation::Shape> opaque(mod, "Shape", nb::type_slots(nucleation_Shape_slots));
    opaque
        .def_static("bezier", std::move(maybe_op_unwrap(&nucleation::Shape::bezier)), "control_points"_a, "thickness"_a, "resolution"_a)
        .def_static("cone", std::move(maybe_op_unwrap(&nucleation::Shape::cone)), "ax"_a, "ay"_a, "az"_a, "dx"_a, "dy"_a, "dz"_a, "radius"_a, "height"_a)
        .def_static("cuboid", std::move(maybe_op_unwrap(&nucleation::Shape::cuboid)), "min_x"_a, "min_y"_a, "min_z"_a, "max_x"_a, "max_y"_a, "max_z"_a)
        .def_static("cylinder", std::move(maybe_op_unwrap(&nucleation::Shape::cylinder)), "bx"_a, "by"_a, "bz"_a, "ax"_a, "ay"_a, "az"_a, "radius"_a, "height"_a)
        .def_static("cylinder_between", std::move(maybe_op_unwrap(&nucleation::Shape::cylinder_between)), "x1"_a, "y1"_a, "z1"_a, "x2"_a, "y2"_a, "z2"_a, "radius"_a)
        .def("difference_with", std::move(maybe_op_unwrap(&nucleation::Shape::difference_with)), "other"_a)
        .def_static("disk", std::move(maybe_op_unwrap(&nucleation::Shape::disk)), "cx"_a, "cy"_a, "cz"_a, "radius"_a, "nx"_a, "ny"_a, "nz"_a, "thickness"_a)
        .def_static("ellipsoid", std::move(maybe_op_unwrap(&nucleation::Shape::ellipsoid)), "cx"_a, "cy"_a, "cz"_a, "rx"_a, "ry"_a, "rz"_a)
        .def("hollow", std::move(maybe_op_unwrap(&nucleation::Shape::hollow)), "thickness"_a)
        .def("intersection_with", std::move(maybe_op_unwrap(&nucleation::Shape::intersection_with)), "other"_a)
        .def_static("line", std::move(maybe_op_unwrap(&nucleation::Shape::line)), "x1"_a, "y1"_a, "z1"_a, "x2"_a, "y2"_a, "z2"_a, "thickness"_a)
        .def_static("plane", std::move(maybe_op_unwrap(&nucleation::Shape::plane)), "ox"_a, "oy"_a, "oz"_a, "ux"_a, "uy"_a, "uz"_a, "vx"_a, "vy"_a, "vz"_a, "u_ext"_a, "v_ext"_a, "thickness"_a)
        .def_static("pyramid", std::move(maybe_op_unwrap(&nucleation::Shape::pyramid)), "bx"_a, "by"_a, "bz"_a, "half_w"_a, "half_d"_a, "height"_a, "ax"_a, "ay"_a, "az"_a)
        .def_static("sdf", std::move(maybe_op_unwrap(&nucleation::Shape::sdf)), "sdf_json"_a)
        .def_static("sdf_bounded", std::move(maybe_op_unwrap(&nucleation::Shape::sdf_bounded)), "sdf_json"_a, "min_x"_a, "min_y"_a, "min_z"_a, "max_x"_a, "max_y"_a, "max_z"_a)
        .def_static("sphere", std::move(maybe_op_unwrap(&nucleation::Shape::sphere)), "cx"_a, "cy"_a, "cz"_a, "radius"_a)
        .def_static("torus", std::move(maybe_op_unwrap(&nucleation::Shape::torus)), "cx"_a, "cy"_a, "cz"_a, "major_r"_a, "minor_r"_a, "ax"_a, "ay"_a, "az"_a)
        .def_static("triangle", std::move(maybe_op_unwrap(&nucleation::Shape::triangle)), "ax"_a, "ay"_a, "az"_a, "bx"_a, "by"_a, "bz"_a, "cx"_a, "cy"_a, "cz"_a, "thickness"_a)
        .def("union_with", std::move(maybe_op_unwrap(&nucleation::Shape::union_with)), "other"_a);
}

} 