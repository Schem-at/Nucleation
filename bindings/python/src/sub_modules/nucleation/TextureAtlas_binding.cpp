#include "diplomat_nanobind_common.hpp"


#include "MeshConfig.hpp"
#include "ResourcePack.hpp"
#include "Schematic.hpp"
#include "TextureAtlas.hpp"

namespace nucleation {
void add_TextureAtlas_binding(nb::module_ mod) {
    PyType_Slot nucleation_TextureAtlas_slots[] = {
        {Py_tp_free, (void *)nucleation::TextureAtlas::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};
    
    nb::class_<nucleation::TextureAtlas> opaque(mod, "TextureAtlas", nb::type_slots(nucleation_TextureAtlas_slots));
    opaque
        .def_static("build_global", std::move(maybe_op_unwrap(&nucleation::TextureAtlas::build_global)), "schematic"_a, "pack"_a, "config"_a)
        .def("height", &nucleation::TextureAtlas::height)
        .def("rgba_data_b64", &nucleation::TextureAtlas::rgba_data_b64)
        .def("width", &nucleation::TextureAtlas::width);
}

} 