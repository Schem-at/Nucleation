#include "diplomat_nanobind_common.hpp"


#include "Blocks.hpp"

namespace nucleation {
void add_Blocks_binding(nb::module_ mod) {
    PyType_Slot nucleation_Blocks_slots[] = {
        {Py_tp_free, (void *)nucleation::Blocks::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};
    
    nb::class_<nucleation::Blocks> opaque(mod, "Blocks", nb::type_slots(nucleation_Blocks_slots));
    opaque
        .def_static("by_color_json", &nucleation::Blocks::by_color_json, "r"_a, "g"_a, "b"_a, "max_distance"_a)
        .def_static("by_kind_json", &nucleation::Blocks::by_kind_json, "kind"_a)
        .def_static("by_tag_json", &nucleation::Blocks::by_tag_json, "tag"_a)
        .def_static("count", &nucleation::Blocks::count)
        .def_static("get_json", &nucleation::Blocks::get_json, "id"_a)
        .def_static("ids_json", &nucleation::Blocks::ids_json)
        .def_static("states_json", &nucleation::Blocks::states_json, "id"_a)
        .def_static("tags_json", &nucleation::Blocks::tags_json)
        .def_static("variants_of_json", &nucleation::Blocks::variants_of_json, "base_id"_a);
}

} 