#include "diplomat_nanobind_common.hpp"


#include "AnimationEffect.hpp"
#include "BuildAnimation.hpp"
#include "RenderConfig.hpp"

namespace nucleation {
void add_BuildAnimation_binding(nb::module_ mod) {
    PyType_Slot nucleation_BuildAnimation_slots[] = {
        {Py_tp_free, (void *)nucleation::BuildAnimation::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};
    
    nb::class_<nucleation::BuildAnimation> opaque(mod, "BuildAnimation", nb::type_slots(nucleation_BuildAnimation_slots));
    opaque
        .def("add_armor_stand", &nucleation::BuildAnimation::add_armor_stand, "x"_a, "y"_a, "z"_a, "yaw"_a, "armor_material"_a)
        .def("animate_camera", &nucleation::BuildAnimation::animate_camera, "effect"_a, "offset_ms"_a)
        .def("begin_group", &nucleation::BuildAnimation::begin_group)
        .def("begin_keyed_group", &nucleation::BuildAnimation::begin_keyed_group, "key"_a)
        .def("clear_stagger", &nucleation::BuildAnimation::clear_stagger)
        .def_static("create", std::move(maybe_op_unwrap(&nucleation::BuildAnimation::create)), "name"_a)
        .def("duration_ms", &nucleation::BuildAnimation::duration_ms)
        .def("end_group", &nucleation::BuildAnimation::end_group)
        .def("frame_count", &nucleation::BuildAnimation::frame_count, "fps"_a, "hold_ms"_a)
        .def("group_count", &nucleation::BuildAnimation::group_count)
        .def("render_frames", &nucleation::BuildAnimation::render_frames, "pack_zip"_a, "config"_a, "prefix"_a, "fps"_a, "hold_ms"_a)
        .def("render_gif", &nucleation::BuildAnimation::render_gif, "pack_zip"_a, "config"_a, "path"_a, "fps"_a, "hold_ms"_a)
        .def("save_to_file", &nucleation::BuildAnimation::save_to_file, "path"_a)
        .def("set_block", &nucleation::BuildAnimation::set_block, "x"_a, "y"_a, "z"_a, "block"_a)
        .def("set_default_effect", &nucleation::BuildAnimation::set_default_effect, "effect"_a)
        .def("set_stagger_total_ms", &nucleation::BuildAnimation::set_stagger_total_ms, "total_ms"_a)
        .def("set_step_ms", &nucleation::BuildAnimation::set_step_ms, "step_ms"_a)
        .def("with_effect", &nucleation::BuildAnimation::with_effect, "effect"_a, nb::rv_policy::reference_internal);
}

} 