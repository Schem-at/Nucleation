#include "diplomat_nanobind_common.hpp"


#include "IoLayoutBuilder.hpp"
#include "IoType.hpp"
#include "LayoutFunction.hpp"

namespace nucleation {
void add_IoLayoutBuilder_binding(nb::module_ mod) {
    PyType_Slot nucleation_IoLayoutBuilder_slots[] = {
        {Py_tp_free, (void *)nucleation::IoLayoutBuilder::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};
    
    nb::class_<nucleation::IoLayoutBuilder> opaque(mod, "IoLayoutBuilder", nb::type_slots(nucleation_IoLayoutBuilder_slots));
    opaque
        .def("add_input", &nucleation::IoLayoutBuilder::add_input, "name"_a, "io_type"_a, "layout"_a, "positions"_a)
        .def("add_input_auto", &nucleation::IoLayoutBuilder::add_input_auto, "name"_a, "io_type"_a, "positions"_a)
        .def("add_input_from_region", &nucleation::IoLayoutBuilder::add_input_from_region, "name"_a, "io_type"_a, "layout"_a, "region_positions"_a)
        .def("add_input_from_region_auto", &nucleation::IoLayoutBuilder::add_input_from_region_auto, "name"_a, "io_type"_a, "region_positions"_a)
        .def("add_output", &nucleation::IoLayoutBuilder::add_output, "name"_a, "io_type"_a, "layout"_a, "positions"_a)
        .def("add_output_auto", &nucleation::IoLayoutBuilder::add_output_auto, "name"_a, "io_type"_a, "positions"_a)
        .def("add_output_from_region", &nucleation::IoLayoutBuilder::add_output_from_region, "name"_a, "io_type"_a, "layout"_a, "region_positions"_a)
        .def("add_output_from_region_auto", &nucleation::IoLayoutBuilder::add_output_from_region_auto, "name"_a, "io_type"_a, "region_positions"_a)
        .def("build", std::move(maybe_op_unwrap(&nucleation::IoLayoutBuilder::build)))
        .def_static("create", std::move(maybe_op_unwrap(&nucleation::IoLayoutBuilder::create)));
}

} 