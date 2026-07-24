#include "diplomat_nanobind_common.hpp"


#include "AnimationEffect.hpp"

namespace nucleation {
void add_AnimationEffect_binding(nb::module_ mod) {
    PyType_Slot nucleation_AnimationEffect_slots[] = {
        {Py_tp_free, (void *)nucleation::AnimationEffect::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};
    
    nb::class_<nucleation::AnimationEffect> opaque(mod, "AnimationEffect", nb::type_slots(nucleation_AnimationEffect_slots));
    opaque
        .def("add_keyframe", &nucleation::AnimationEffect::add_keyframe, "property_name"_a, "at"_a, "value"_a, "easing_name"_a)
        .def("add_tween", &nucleation::AnimationEffect::add_tween, "property_name"_a, "from"_a, "to"_a, "easing_name"_a)
        .def_static("create", std::move(maybe_op_unwrap(&nucleation::AnimationEffect::create)), "duration_ms"_a)
        .def_static("drop_and_pop", std::move(maybe_op_unwrap(&nucleation::AnimationEffect::drop_and_pop)), "duration_ms"_a, "height"_a)
        .def_static("drop_in", std::move(maybe_op_unwrap(&nucleation::AnimationEffect::drop_in)), "duration_ms"_a, "height"_a)
        .def_static("instant", std::move(maybe_op_unwrap(&nucleation::AnimationEffect::instant)))
        .def_static("pop_in", std::move(maybe_op_unwrap(&nucleation::AnimationEffect::pop_in)), "duration_ms"_a)
        .def("set_repeat_forever", &nucleation::AnimationEffect::set_repeat_forever)
        .def_static("spin_in", std::move(maybe_op_unwrap(&nucleation::AnimationEffect::spin_in)), "duration_ms"_a, "turns"_a)
        .def_static("turntable", std::move(maybe_op_unwrap(&nucleation::AnimationEffect::turntable)), "duration_ms"_a);
}

} 