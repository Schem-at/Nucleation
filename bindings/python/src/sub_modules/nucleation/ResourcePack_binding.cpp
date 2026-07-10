#include "diplomat_nanobind_common.hpp"


#include "ResourcePack.hpp"
#include "ResourcePackList.hpp"

namespace nucleation {
void add_ResourcePack_binding(nb::module_ mod) {
    PyType_Slot nucleation_ResourcePack_slots[] = {
        {Py_tp_free, (void *)nucleation::ResourcePack::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};
    
    nb::class_<nucleation::ResourcePack> opaque(mod, "ResourcePack", nb::type_slots(nucleation_ResourcePack_slots));
    opaque
        .def("add_blockstate_json", &nucleation::ResourcePack::add_blockstate_json, "name"_a, "json"_a)
        .def("add_model_json", &nucleation::ResourcePack::add_model_json, "name"_a, "json"_a)
        .def("add_texture", &nucleation::ResourcePack::add_texture, "name"_a, "width"_a, "height"_a, "pixels"_a)
        .def("blockstate_count", &nucleation::ResourcePack::blockstate_count)
        .def_static("from_bytes", std::move(maybe_op_unwrap(&nucleation::ResourcePack::from_bytes)), "data"_a)
        .def_static("from_list", std::move(maybe_op_unwrap(&nucleation::ResourcePack::from_list)), "list"_a)
        .def("get_blockstate_json", &nucleation::ResourcePack::get_blockstate_json, "name"_a)
        .def("get_model_json", &nucleation::ResourcePack::get_model_json, "name"_a)
        .def("get_texture_info", &nucleation::ResourcePack::get_texture_info, "name"_a)
        .def("get_texture_pixels_b64", &nucleation::ResourcePack::get_texture_pixels_b64, "name"_a)
        .def("list_blockstates_json", &nucleation::ResourcePack::list_blockstates_json)
        .def("list_models_json", &nucleation::ResourcePack::list_models_json)
        .def("list_textures_json", &nucleation::ResourcePack::list_textures_json)
        .def("model_count", &nucleation::ResourcePack::model_count)
        .def("namespaces_json", &nucleation::ResourcePack::namespaces_json)
        .def("register_mesh_exporter", &nucleation::ResourcePack::register_mesh_exporter)
        .def("texture_count", &nucleation::ResourcePack::texture_count);
}

} 