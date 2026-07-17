#include "diplomat_nanobind_common.hpp"


#include "Palette.hpp"

namespace nucleation {
void add_Palette_binding(nb::module_ mod) {
    PyType_Slot nucleation_Palette_slots[] = {
        {Py_tp_free, (void *)nucleation::Palette::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};
    
    nb::class_<nucleation::Palette> opaque(mod, "Palette", nb::type_slots(nucleation_Palette_slots));
    opaque
        .def_static("all", std::move(maybe_op_unwrap(&nucleation::Palette::all)))
        .def("block_ids_json", &nucleation::Palette::block_ids_json)
        .def("closest_block", &nucleation::Palette::closest_block, "r"_a, "g"_a, "b"_a)
        .def_static("concrete", std::move(maybe_op_unwrap(&nucleation::Palette::concrete)))
        .def_static("decorative", std::move(maybe_op_unwrap(&nucleation::Palette::decorative)))
        .def_static("from_block_ids", std::move(maybe_op_unwrap(&nucleation::Palette::from_block_ids)), "ids_json"_a)
        .def_static("grayscale", std::move(maybe_op_unwrap(&nucleation::Palette::grayscale)))
        .def("len", &nucleation::Palette::len)
        .def_static("solid", std::move(maybe_op_unwrap(&nucleation::Palette::solid)))
        .def_static("structural", std::move(maybe_op_unwrap(&nucleation::Palette::structural)))
        .def_static("terracotta", std::move(maybe_op_unwrap(&nucleation::Palette::terracotta)))
        .def_static("wool", std::move(maybe_op_unwrap(&nucleation::Palette::wool)));
}

} 