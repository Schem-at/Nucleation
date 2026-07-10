#include "diplomat_nanobind_common.hpp"


#include "Nbt.hpp"

namespace nucleation {
void add_Nbt_binding(nb::module_ mod) {
    PyType_Slot nucleation_Nbt_slots[] = {
        {Py_tp_free, (void *)nucleation::Nbt::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};
    
    nb::class_<nucleation::Nbt> opaque(mod, "Nbt", nb::type_slots(nucleation_Nbt_slots));
    opaque
        .def_static("chest_build", &nucleation::Nbt::chest_build, "items_json"_a, "name"_a)
        .def_static("sign_build", &nucleation::Nbt::sign_build, "front_json"_a, "back_json"_a, "color"_a, "glowing"_a, "waxed"_a)
        .def_static("text_build", &nucleation::Nbt::text_build, "s"_a, "color"_a, "bold"_a, "italic"_a);
}

} 