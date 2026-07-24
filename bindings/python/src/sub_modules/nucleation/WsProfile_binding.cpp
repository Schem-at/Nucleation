#include "diplomat_nanobind_common.hpp"


#include "WsProfile.hpp"

namespace nucleation {
void add_WsProfile_binding(nb::module_ mod) {
    PyType_Slot nucleation_WsProfile_slots[] = {
        {Py_tp_free, (void *)nucleation::WsProfile::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};
    
    nb::class_<nucleation::WsProfile> opaque(mod, "WsProfile", nb::type_slots(nucleation_WsProfile_slots));
    opaque
        .def("band_max", &nucleation::WsProfile::band_max)
        .def("band_min", &nucleation::WsProfile::band_min)
        .def_static("derive_from_dir", std::move(maybe_op_unwrap(&nucleation::WsProfile::derive_from_dir)), "world_dir"_a, "min_y"_a, "max_y"_a, "sample"_a, "coverage"_a)
        .def("palette_len", &nucleation::WsProfile::palette_len)
        .def("write_palette_json", &nucleation::WsProfile::write_palette_json);
}

} 