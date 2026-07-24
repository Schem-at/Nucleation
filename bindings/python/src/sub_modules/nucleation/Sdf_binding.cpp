#include "diplomat_nanobind_common.hpp"


#include "FieldProgram.hpp"
#include "Sdf.hpp"
#include "SdfAxis.hpp"
#include "SdfCellMode.hpp"

namespace nucleation {
void add_Sdf_binding(nb::module_ mod) {
    PyType_Slot nucleation_Sdf_slots[] = {
        {Py_tp_free, (void *)nucleation::Sdf::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};

    nb::class_<nucleation::Sdf> opaque(mod, "Sdf", nb::type_slots(nucleation_Sdf_slots));
    opaque
        .def("bend", std::move(maybe_op_unwrap(&nucleation::Sdf::bend)), "amount"_a)
        .def("bounds", &nucleation::Sdf::bounds)
        .def_static("box_frame", std::move(maybe_op_unwrap(&nucleation::Sdf::box_frame)), "half_x"_a, "half_y"_a, "half_z"_a, "thickness"_a)
        .def_static("box_shape", std::move(maybe_op_unwrap(&nucleation::Sdf::box_shape)), "half_x"_a, "half_y"_a, "half_z"_a, "rounding"_a)
        .def_static("capped_cone", std::move(maybe_op_unwrap(&nucleation::Sdf::capped_cone)), "half_height"_a, "bottom_radius"_a, "top_radius"_a)
        .def_static("capped_cylinder", std::move(maybe_op_unwrap(&nucleation::Sdf::capped_cylinder)), "radius"_a, "half_height"_a)
        .def_static("capped_torus", std::move(maybe_op_unwrap(&nucleation::Sdf::capped_torus)), "major_radius"_a, "minor_radius"_a, "cap_angle_degrees"_a)
        .def_static("capsule", std::move(maybe_op_unwrap(&nucleation::Sdf::capsule)), "ax"_a, "ay"_a, "az"_a, "bx"_a, "by"_a, "bz"_a, "radius"_a)
        .def_static("cells", std::move(maybe_op_unwrap(&nucleation::Sdf::cells)), "frequency"_a, "seed"_a, "jitter"_a, "mode"_a, "threshold"_a)
        .def_static("cut_hollow_sphere", std::move(maybe_op_unwrap(&nucleation::Sdf::cut_hollow_sphere)), "radius"_a, "height"_a, "thickness"_a)
        .def_static("cut_sphere", std::move(maybe_op_unwrap(&nucleation::Sdf::cut_sphere)), "radius"_a, "height"_a)
        .def("displace", std::move(maybe_op_unwrap(&nucleation::Sdf::displace)), "amplitude"_a, "frequency"_a, "seed"_a, "octaves"_a)
        .def_static("ellipsoid", std::move(maybe_op_unwrap(&nucleation::Sdf::ellipsoid)), "radius_x"_a, "radius_y"_a, "radius_z"_a)
        .def("elongate", std::move(maybe_op_unwrap(&nucleation::Sdf::elongate)), "half_x"_a, "half_y"_a, "half_z"_a)
        .def_static("eval", &nucleation::Sdf::eval, "sdf_json"_a, "x"_a, "y"_a, "z"_a)
        .def("eval_at", &nucleation::Sdf::eval_at, "x"_a, "y"_a, "z"_a)
        .def_static("from_json_string", std::move(maybe_op_unwrap(&nucleation::Sdf::from_json_string)), "json"_a)
        .def_static("from_program", std::move(maybe_op_unwrap(&nucleation::Sdf::from_program)), "program"_a)
        .def_static("hex_prism", std::move(maybe_op_unwrap(&nucleation::Sdf::hex_prism)), "radius"_a, "half_height"_a)
        .def_static("infinite_cone", std::move(maybe_op_unwrap(&nucleation::Sdf::infinite_cone)), "angle_degrees"_a)
        .def_static("infinite_cylinder", std::move(maybe_op_unwrap(&nucleation::Sdf::infinite_cylinder)), "radius"_a)
        .def("intersection_with", std::move(maybe_op_unwrap(&nucleation::Sdf::intersection_with)), "other"_a)
        .def_static("link", std::move(maybe_op_unwrap(&nucleation::Sdf::link)), "major_radius"_a, "minor_radius"_a, "half_length"_a)
        .def("mirror", std::move(maybe_op_unwrap(&nucleation::Sdf::mirror)), "axis"_a)
        .def("normal", &nucleation::Sdf::normal, "x"_a, "y"_a, "z"_a, "epsilon"_a)
        .def_static("octahedron", std::move(maybe_op_unwrap(&nucleation::Sdf::octahedron)), "size"_a)
        .def_static("plane", std::move(maybe_op_unwrap(&nucleation::Sdf::plane)), "normal_x"_a, "normal_y"_a, "normal_z"_a, "offset"_a)
        .def("repeat_counted", std::move(maybe_op_unwrap(&nucleation::Sdf::repeat_counted)), "spacing_x"_a, "spacing_y"_a, "spacing_z"_a, "count_x"_a, "count_y"_a, "count_z"_a)
        .def("repeat_infinite", std::move(maybe_op_unwrap(&nucleation::Sdf::repeat_infinite)), "spacing_x"_a, "spacing_y"_a, "spacing_z"_a)
        .def("rotate", std::move(maybe_op_unwrap(&nucleation::Sdf::rotate)), "x_degrees"_a, "y_degrees"_a, "z_degrees"_a)
        .def_static("round_cone", std::move(maybe_op_unwrap(&nucleation::Sdf::round_cone)), "ax"_a, "ay"_a, "az"_a, "bx"_a, "by"_a, "bz"_a, "r1"_a, "r2"_a)
        .def("rounded", std::move(maybe_op_unwrap(&nucleation::Sdf::rounded)), "radius"_a)
        .def("scale", std::move(maybe_op_unwrap(&nucleation::Sdf::scale)), "factor"_a)
        .def_static("schematic_from_sdf", std::move(maybe_op_unwrap(&nucleation::Sdf::schematic_from_sdf)), "sdf_json"_a, "rules_json"_a, "has_bounds"_a, "min_x"_a, "min_y"_a, "min_z"_a, "max_x"_a, "max_y"_a, "max_z"_a)
        .def_static("schematic_from_sdf_auto", std::move(maybe_op_unwrap(&nucleation::Sdf::schematic_from_sdf_auto)), "sdf_json"_a, "rules_json"_a)
        .def("shell", std::move(maybe_op_unwrap(&nucleation::Sdf::shell)), "thickness"_a)
        .def("smooth_intersection", std::move(maybe_op_unwrap(&nucleation::Sdf::smooth_intersection)), "other"_a, "radius"_a)
        .def("smooth_subtract", std::move(maybe_op_unwrap(&nucleation::Sdf::smooth_subtract)), "other"_a, "radius"_a)
        .def("smooth_union", std::move(maybe_op_unwrap(&nucleation::Sdf::smooth_union)), "other"_a, "radius"_a)
        .def_static("solid_angle", std::move(maybe_op_unwrap(&nucleation::Sdf::solid_angle)), "radius"_a, "angle_degrees"_a)
        .def_static("sphere", std::move(maybe_op_unwrap(&nucleation::Sdf::sphere)), "radius"_a)
        .def_static("square_pyramid", std::move(maybe_op_unwrap(&nucleation::Sdf::square_pyramid)), "half_base"_a, "height"_a)
        .def("subtract", std::move(maybe_op_unwrap(&nucleation::Sdf::subtract)), "other"_a)
        .def_static("super_prism", std::move(maybe_op_unwrap(&nucleation::Sdf::super_prism)), "half_x"_a, "half_y"_a, "half_z"_a, "exponent"_a)
        .def("to_json", &nucleation::Sdf::to_json)
        .def("to_shape", std::move(maybe_op_unwrap(&nucleation::Sdf::to_shape)))
        .def("to_shape_bounded", std::move(maybe_op_unwrap(&nucleation::Sdf::to_shape_bounded)), "min_x"_a, "min_y"_a, "min_z"_a, "max_x"_a, "max_y"_a, "max_z"_a)
        .def_static("torus", std::move(maybe_op_unwrap(&nucleation::Sdf::torus)), "major_radius"_a, "minor_radius"_a)
        .def("translate", std::move(maybe_op_unwrap(&nucleation::Sdf::translate)), "x"_a, "y"_a, "z"_a)
        .def("twist", std::move(maybe_op_unwrap(&nucleation::Sdf::twist)), "amount"_a)
        .def("union_with", std::move(maybe_op_unwrap(&nucleation::Sdf::union_with)), "other"_a)
        .def("warp", std::move(maybe_op_unwrap(&nucleation::Sdf::warp)), "amplitude"_a, "frequency"_a, "seed"_a)
        .def("xor_with", std::move(maybe_op_unwrap(&nucleation::Sdf::xor_with)), "other"_a);
}

}
