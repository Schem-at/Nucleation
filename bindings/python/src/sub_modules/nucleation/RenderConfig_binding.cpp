#include "diplomat_nanobind_common.hpp"


#include "RenderConfig.hpp"

namespace nucleation {
void add_RenderConfig_binding(nb::module_ mod) {
    PyType_Slot nucleation_RenderConfig_slots[] = {
        {Py_tp_free, (void *)nucleation::RenderConfig::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};
    
    nb::class_<nucleation::RenderConfig> opaque(mod, "RenderConfig", nb::type_slots(nucleation_RenderConfig_slots));
    opaque
        .def("clear_background", &nucleation::RenderConfig::clear_background)
        .def_static("create", std::move(maybe_op_unwrap(&nucleation::RenderConfig::create)), "width"_a, "height"_a)
        .def("set_background", &nucleation::RenderConfig::set_background, "r"_a, "g"_a, "b"_a, "a"_a)
        .def("set_fov", &nucleation::RenderConfig::set_fov, "fov"_a)
        .def("set_isometric", &nucleation::RenderConfig::set_isometric)
        .def("set_orthographic", &nucleation::RenderConfig::set_orthographic, "orthographic"_a)
        .def("set_pitch", &nucleation::RenderConfig::set_pitch, "pitch"_a)
        .def("set_yaw", &nucleation::RenderConfig::set_yaw, "yaw"_a)
        .def("set_zoom", &nucleation::RenderConfig::set_zoom, "zoom"_a);
}

} 