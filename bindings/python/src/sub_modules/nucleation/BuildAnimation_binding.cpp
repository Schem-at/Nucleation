#include "diplomat_nanobind_common.hpp"


#include "AnimationEffect.hpp"
#include "Brush.hpp"
#include "BuildAnimation.hpp"
#include "RenderConfig.hpp"
#include "Schematic.hpp"
#include "Shape.hpp"

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
        .def("clear_loop_period", &nucleation::BuildAnimation::clear_loop_period)
        .def("clear_stagger", &nucleation::BuildAnimation::clear_stagger)
        .def_static("create", std::move(maybe_op_unwrap(&nucleation::BuildAnimation::create)), "name"_a)
        .def("create_region", &nucleation::BuildAnimation::create_region, "name"_a, "min_x"_a, "min_y"_a, "min_z"_a, "max_x"_a, "max_y"_a, "max_z"_a)
        .def("duration_ms", &nucleation::BuildAnimation::duration_ms)
        .def("end_group", &nucleation::BuildAnimation::end_group)
        .def("fill_along_parameter", &nucleation::BuildAnimation::fill_along_parameter, "shape"_a, "brush"_a, "group_count"_a)
        .def("flip_all_x", &nucleation::BuildAnimation::flip_all_x, "duration_ms"_a)
        .def("flip_all_y", &nucleation::BuildAnimation::flip_all_y, "duration_ms"_a)
        .def("flip_all_z", &nucleation::BuildAnimation::flip_all_z, "duration_ms"_a)
        .def("flip_region_x", &nucleation::BuildAnimation::flip_region_x, "region"_a, "duration_ms"_a)
        .def("flip_region_y", &nucleation::BuildAnimation::flip_region_y, "region"_a, "duration_ms"_a)
        .def("flip_region_z", &nucleation::BuildAnimation::flip_region_z, "region"_a, "duration_ms"_a)
        .def("flip_x", &nucleation::BuildAnimation::flip_x, "duration_ms"_a)
        .def("flip_y", &nucleation::BuildAnimation::flip_y, "duration_ms"_a)
        .def("flip_z", &nucleation::BuildAnimation::flip_z, "duration_ms"_a)
        .def("frame_count", &nucleation::BuildAnimation::frame_count, "fps"_a, "hold_ms"_a)
        .def("frame_json", &nucleation::BuildAnimation::frame_json, "time_ms"_a)
        .def("group_count", &nucleation::BuildAnimation::group_count)
        .def("operations_json", &nucleation::BuildAnimation::operations_json)
        .def("render_frames", &nucleation::BuildAnimation::render_frames, "pack_zip"_a, "config"_a, "prefix"_a, "fps"_a, "hold_ms"_a)
        .def("render_gif", &nucleation::BuildAnimation::render_gif, "pack_zip"_a, "config"_a, "path"_a, "fps"_a, "hold_ms"_a)
        .def("rotate_all_x", &nucleation::BuildAnimation::rotate_all_x, "degrees"_a, "duration_ms"_a)
        .def("rotate_all_y", &nucleation::BuildAnimation::rotate_all_y, "degrees"_a, "duration_ms"_a)
        .def("rotate_all_z", &nucleation::BuildAnimation::rotate_all_z, "degrees"_a, "duration_ms"_a)
        .def("rotate_region_x", &nucleation::BuildAnimation::rotate_region_x, "region"_a, "degrees"_a, "duration_ms"_a)
        .def("rotate_region_y", &nucleation::BuildAnimation::rotate_region_y, "region"_a, "degrees"_a, "duration_ms"_a)
        .def("rotate_region_z", &nucleation::BuildAnimation::rotate_region_z, "region"_a, "degrees"_a, "duration_ms"_a)
        .def("rotate_x", &nucleation::BuildAnimation::rotate_x, "degrees"_a, "duration_ms"_a)
        .def("rotate_y", &nucleation::BuildAnimation::rotate_y, "degrees"_a, "duration_ms"_a)
        .def("rotate_z", &nucleation::BuildAnimation::rotate_z, "degrees"_a, "duration_ms"_a)
        .def("save_to_file", &nucleation::BuildAnimation::save_to_file, "path"_a)
        .def("set_block", &nucleation::BuildAnimation::set_block, "x"_a, "y"_a, "z"_a, "block"_a)
        .def("set_block_in_region", &nucleation::BuildAnimation::set_block_in_region, "region"_a, "x"_a, "y"_a, "z"_a, "block"_a)
        .def("set_default_effect", &nucleation::BuildAnimation::set_default_effect, "effect"_a)
        .def("set_loop_period_ms", &nucleation::BuildAnimation::set_loop_period_ms, "period_ms"_a)
        .def("set_operation_gizmos", &nucleation::BuildAnimation::set_operation_gizmos, "enabled"_a)
        .def("set_stagger_offset_ms", &nucleation::BuildAnimation::set_stagger_offset_ms, "offset_ms"_a)
        .def("set_stagger_total_ms", &nucleation::BuildAnimation::set_stagger_total_ms, "total_ms"_a)
        .def("set_step_ms", &nucleation::BuildAnimation::set_step_ms, "step_ms"_a)
        .def("stamp_box", &nucleation::BuildAnimation::stamp_box, "source"_a, "min_x"_a, "min_y"_a, "min_z"_a, "max_x"_a, "max_y"_a, "max_z"_a, "x"_a, "y"_a, "z"_a, "exclusions"_a, "duration_ms"_a)
        .def("stamp_region", &nucleation::BuildAnimation::stamp_region, "source"_a, "region"_a, "x"_a, "y"_a, "z"_a, "exclusions"_a, "duration_ms"_a)
        .def("translate", &nucleation::BuildAnimation::translate, "x"_a, "y"_a, "z"_a, "duration_ms"_a)
        .def("translate_all", &nucleation::BuildAnimation::translate_all, "x"_a, "y"_a, "z"_a, "duration_ms"_a)
        .def("translate_region", &nucleation::BuildAnimation::translate_region, "region"_a, "x"_a, "y"_a, "z"_a, "duration_ms"_a)
        .def("with_effect", &nucleation::BuildAnimation::with_effect, "effect"_a, nb::rv_policy::reference_internal);
}

} 