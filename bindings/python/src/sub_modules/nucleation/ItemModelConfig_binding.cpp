#include "diplomat_nanobind_common.hpp"


#include "ItemModelConfig.hpp"

namespace nucleation {
void add_ItemModelConfig_binding(nb::module_ mod) {
    PyType_Slot nucleation_ItemModelConfig_slots[] = {
        {Py_tp_free, (void *)nucleation::ItemModelConfig::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};
    
    nb::class_<nucleation::ItemModelConfig> opaque(mod, "ItemModelConfig", nb::type_slots(nucleation_ItemModelConfig_slots));
    opaque
        .def_static("create", std::move(maybe_op_unwrap(&nucleation::ItemModelConfig::create)), "model_name"_a)
        .def("set_center", &nucleation::ItemModelConfig::set_center, "center"_a)
        .def("set_custom_model_data", &nucleation::ItemModelConfig::set_custom_model_data, "cmd"_a)
        .def("set_item", &nucleation::ItemModelConfig::set_item, "item"_a)
        .def("set_namespace", &nucleation::ItemModelConfig::set_namespace, "namespace"_a)
        .def("set_scale", &nucleation::ItemModelConfig::set_scale, "scale"_a)
        .def("set_scale_auto", &nucleation::ItemModelConfig::set_scale_auto)
        .def("set_scale_xyz", &nucleation::ItemModelConfig::set_scale_xyz, "sx"_a, "sy"_a, "sz"_a)
        .def("set_texture_resolution", &nucleation::ItemModelConfig::set_texture_resolution, "resolution"_a);
}

} 