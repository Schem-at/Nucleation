#include "diplomat_nanobind_common.hpp"


#include "Material.hpp"
#include "MaterialBuilder.hpp"
#include "MaterialCondition.hpp"

namespace nucleation {
void add_MaterialBuilder_binding(nb::module_ mod) {
    PyType_Slot nucleation_MaterialBuilder_slots[] = {
        {Py_tp_free, (void *)nucleation::MaterialBuilder::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};

    nb::class_<nucleation::MaterialBuilder> opaque(mod, "MaterialBuilder", nb::type_slots(nucleation_MaterialBuilder_slots));
    opaque
        .def("add_layer", &nucleation::MaterialBuilder::add_layer, "condition"_a, "material"_a)
        .def("add_tag_case", &nucleation::MaterialBuilder::add_tag_case, "material_id"_a, "material"_a)
        .def("build", std::move(maybe_op_unwrap(&nucleation::MaterialBuilder::build)))
        .def_static("create_layered", std::move(maybe_op_unwrap(&nucleation::MaterialBuilder::create_layered)))
        .def_static("create_tag_matcher", std::move(maybe_op_unwrap(&nucleation::MaterialBuilder::create_tag_matcher)), "blend_seams"_a, "seed"_a)
        .def("set_default", &nucleation::MaterialBuilder::set_default, "material"_a);
}

}
