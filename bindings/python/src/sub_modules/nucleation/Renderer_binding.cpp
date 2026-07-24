#include "diplomat_nanobind_common.hpp"


#include "RenderConfig.hpp"
#include "Renderer.hpp"
#include "ResourcePack.hpp"
#include "Schematic.hpp"

namespace nucleation {
void add_Renderer_binding(nb::module_ mod) {
    PyType_Slot nucleation_Renderer_slots[] = {
        {Py_tp_free, (void *)nucleation::Renderer::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};
    
    nb::class_<nucleation::Renderer> opaque(mod, "Renderer", nb::type_slots(nucleation_Renderer_slots));
    opaque
        .def_static("render_pixels_b64", &nucleation::Renderer::render_pixels_b64, "schematic"_a, "pack_zip"_a, "config"_a)
        .def_static("render_pixels_b64_with_pack", &nucleation::Renderer::render_pixels_b64_with_pack, "schematic"_a, "pack"_a, "config"_a)
        .def_static("render_png_b64", &nucleation::Renderer::render_png_b64, "schematic"_a, "pack_zip"_a, "config"_a)
        .def_static("render_png_b64_with_pack", &nucleation::Renderer::render_png_b64_with_pack, "schematic"_a, "pack"_a, "config"_a)
        .def_static("render_to_file", &nucleation::Renderer::render_to_file, "schematic"_a, "pack_zip"_a, "config"_a, "path"_a)
        .def_static("render_to_file_with_pack", &nucleation::Renderer::render_to_file_with_pack, "schematic"_a, "pack"_a, "config"_a, "path"_a);
}

} 