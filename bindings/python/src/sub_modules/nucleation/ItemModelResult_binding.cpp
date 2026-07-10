#include "diplomat_nanobind_common.hpp"


#include "ItemModelConfig.hpp"
#include "ItemModelPackBuilder.hpp"
#include "ItemModelResult.hpp"
#include "ResourcePack.hpp"
#include "Schematic.hpp"

namespace nucleation {
void add_ItemModelResult_binding(nb::module_ mod) {
    PyType_Slot nucleation_ItemModelResult_slots[] = {
        {Py_tp_free, (void *)nucleation::ItemModelResult::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};
    
    nb::class_<nucleation::ItemModelResult> opaque(mod, "ItemModelResult", nb::type_slots(nucleation_ItemModelResult_slots));
    opaque
        .def("add_to_pack", &nucleation::ItemModelResult::add_to_pack, "builder"_a)
        .def_static("create", std::move(maybe_op_unwrap(&nucleation::ItemModelResult::create)), "schematic"_a, "pack"_a, "config"_a)
        .def("dimensions", &nucleation::ItemModelResult::dimensions)
        .def("element_count", &nucleation::ItemModelResult::element_count)
        .def("model_json", &nucleation::ItemModelResult::model_json)
        .def("plane_count", &nucleation::ItemModelResult::plane_count)
        .def("scale", &nucleation::ItemModelResult::scale)
        .def("texture_count", &nucleation::ItemModelResult::texture_count)
        .def("to_resource_pack_zip_b64", &nucleation::ItemModelResult::to_resource_pack_zip_b64);
}

} 