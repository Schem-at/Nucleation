#include "diplomat_nanobind_common.hpp"


#include "Brush.hpp"
#include "InterpolationSpace.hpp"
#include "Palette.hpp"

namespace nucleation {
void add_Brush_binding(nb::module_ mod) {
    PyType_Slot nucleation_Brush_slots[] = {
        {Py_tp_free, (void *)nucleation::Brush::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};
    
    nb::class_<nucleation::Brush> opaque(mod, "Brush", nb::type_slots(nucleation_Brush_slots));
    opaque
        .def_static("bilinear_gradient", std::move(maybe_op_unwrap(&nucleation::Brush::bilinear_gradient)), "ox"_a, "oy"_a, "oz"_a, "ux"_a, "uy"_a, "uz"_a, "vx"_a, "vy"_a, "vz"_a, "r00"_a, "g00"_a, "b00"_a, "r10"_a, "g10"_a, "b10"_a, "r01"_a, "g01"_a, "b01"_a, "r11"_a, "g11"_a, "b11"_a, "space"_a)
        .def_static("color", std::move(maybe_op_unwrap(&nucleation::Brush::color)), "r"_a, "g"_a, "b"_a)
        .def_static("curve_gradient", std::move(maybe_op_unwrap(&nucleation::Brush::curve_gradient)), "stops"_a, "colors"_a, "space"_a)
        .def_static("field", std::move(maybe_op_unwrap(&nucleation::Brush::field)), "field_json"_a, "stops"_a, "colors"_a, "lo"_a, "hi"_a, "space"_a)
        .def_static("linear_gradient", std::move(maybe_op_unwrap(&nucleation::Brush::linear_gradient)), "x1"_a, "y1"_a, "z1"_a, "r1"_a, "g1"_a, "b1"_a, "x2"_a, "y2"_a, "z2"_a, "r2"_a, "g2"_a, "b2"_a, "space"_a)
        .def_static("point_gradient", std::move(maybe_op_unwrap(&nucleation::Brush::point_gradient)), "positions"_a, "colors"_a, "falloff"_a, "space"_a)
        .def("set_palette", &nucleation::Brush::set_palette, "palette"_a)
        .def_static("shaded", std::move(maybe_op_unwrap(&nucleation::Brush::shaded)), "r"_a, "g"_a, "b"_a, "lx"_a, "ly"_a, "lz"_a)
        .def_static("solid", std::move(maybe_op_unwrap(&nucleation::Brush::solid)), "block_name"_a)
        .def_static("spotlight", std::move(maybe_op_unwrap(&nucleation::Brush::spotlight)), "px"_a, "py"_a, "pz"_a, "dx"_a, "dy"_a, "dz"_a, "cone_angle_deg"_a, "r"_a, "g"_a, "b"_a);
}

} 