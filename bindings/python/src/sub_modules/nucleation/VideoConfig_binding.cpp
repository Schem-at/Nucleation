#include "diplomat_nanobind_common.hpp"


#include "VideoConfig.hpp"

namespace nucleation {
void add_VideoConfig_binding(nb::module_ mod) {
    PyType_Slot nucleation_VideoConfig_slots[] = {
        {Py_tp_free, (void *)nucleation::VideoConfig::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};
    
    nb::class_<nucleation::VideoConfig> opaque(mod, "VideoConfig", nb::type_slots(nucleation_VideoConfig_slots));
    opaque
        .def_static("h264", std::move(maybe_op_unwrap(&nucleation::VideoConfig::h264)), "fps"_a)
        .def_static("prores_4444", std::move(maybe_op_unwrap(&nucleation::VideoConfig::prores_4444)), "fps"_a)
        .def("set_ffmpeg_path", &nucleation::VideoConfig::set_ffmpeg_path, "path"_a);
}

} 