#include "diplomat_nanobind_common.hpp"


#include "Brush.hpp"
#include "Field.hpp"
#include "FieldChannel.hpp"
#include "InterpolationSpace.hpp"
#include "Material.hpp"
#include "Palette.hpp"

namespace nucleation {
void add_Material_binding(nb::module_ mod) {
    PyType_Slot nucleation_Material_slots[] = {
        {Py_tp_free, (void *)nucleation::Material::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};

    nb::class_<nucleation::Material> opaque(mod, "Material", nb::type_slots(nucleation_Material_slots));
    opaque
        .def_static("channel_gradient", std::move(maybe_op_unwrap(&nucleation::Material::channel_gradient)), "channel"_a, "stops"_a, "colors"_a, "lo"_a, "hi"_a, "palette"_a, "space"_a)
        .def_static("channel_ramp", std::move(maybe_op_unwrap(&nucleation::Material::channel_ramp)), "channel"_a, "lo"_a, "hi"_a, "blocks_json"_a, "dither"_a)
        .def_static("color", std::move(maybe_op_unwrap(&nucleation::Material::color)), "r"_a, "g"_a, "b"_a, "palette"_a)
        .def_static("field_gradient", std::move(maybe_op_unwrap(&nucleation::Material::field_gradient)), "field"_a, "stops"_a, "colors"_a, "lo"_a, "hi"_a, "palette"_a, "space"_a)
        .def_static("field_ramp", std::move(maybe_op_unwrap(&nucleation::Material::field_ramp)), "field"_a, "lo"_a, "hi"_a, "blocks_json"_a, "dither"_a)
        .def_static("from_brush", std::move(maybe_op_unwrap(&nucleation::Material::from_brush)), "brush"_a)
        .def_static("shaded", std::move(maybe_op_unwrap(&nucleation::Material::shaded)), "r"_a, "g"_a, "b"_a, "lx"_a, "ly"_a, "lz"_a, "ambient"_a, "palette"_a)
        .def_static("skip", std::move(maybe_op_unwrap(&nucleation::Material::skip)))
        .def_static("solid", std::move(maybe_op_unwrap(&nucleation::Material::solid)), "block"_a)
        .def_static("spotlight", std::move(maybe_op_unwrap(&nucleation::Material::spotlight)), "px"_a, "py"_a, "pz"_a, "dx"_a, "dy"_a, "dz"_a, "cone_angle_deg"_a, "r"_a, "g"_a, "b"_a, "ambient"_a, "palette"_a)
        .def_static("weighted", std::move(maybe_op_unwrap(&nucleation::Material::weighted)), "blocks_json"_a, "weights"_a, "seed"_a);
}

}
