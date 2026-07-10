#include "diplomat_nanobind_common.hpp"


#include "CircuitBuilder.hpp"
#include "IoType.hpp"
#include "LayoutFunction.hpp"
#include "Schematic.hpp"
#include "SortStrategy.hpp"

namespace nucleation {
void add_CircuitBuilder_binding(nb::module_ mod) {
    PyType_Slot nucleation_CircuitBuilder_slots[] = {
        {Py_tp_free, (void *)nucleation::CircuitBuilder::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};
    
    nb::class_<nucleation::CircuitBuilder> opaque(mod, "CircuitBuilder", nb::type_slots(nucleation_CircuitBuilder_slots));
    opaque
        .def("build", std::move(maybe_op_unwrap(&nucleation::CircuitBuilder::build)))
        .def("build_validated", std::move(maybe_op_unwrap(&nucleation::CircuitBuilder::build_validated)))
        .def_static("create", std::move(maybe_op_unwrap(&nucleation::CircuitBuilder::create)), "schematic"_a)
        .def_static("from_insign", std::move(maybe_op_unwrap(&nucleation::CircuitBuilder::from_insign)), "schematic"_a)
        .def("input_count", &nucleation::CircuitBuilder::input_count)
        .def("input_names_json", &nucleation::CircuitBuilder::input_names_json)
        .def("output_count", &nucleation::CircuitBuilder::output_count)
        .def("output_names_json", &nucleation::CircuitBuilder::output_names_json)
        .def("validate", &nucleation::CircuitBuilder::validate)
        .def("with_input", &nucleation::CircuitBuilder::with_input, "name"_a, "io_type"_a, "layout"_a, "region_positions"_a)
        .def("with_input_auto", &nucleation::CircuitBuilder::with_input_auto, "name"_a, "io_type"_a, "region_positions"_a)
        .def("with_input_auto_sorted", &nucleation::CircuitBuilder::with_input_auto_sorted, "name"_a, "io_type"_a, "region_positions"_a, "sort"_a)
        .def("with_input_sorted", &nucleation::CircuitBuilder::with_input_sorted, "name"_a, "io_type"_a, "layout"_a, "region_positions"_a, "sort"_a)
        .def("with_options", &nucleation::CircuitBuilder::with_options, "optimize"_a, "io_only"_a)
        .def("with_output", &nucleation::CircuitBuilder::with_output, "name"_a, "io_type"_a, "layout"_a, "region_positions"_a)
        .def("with_output_auto", &nucleation::CircuitBuilder::with_output_auto, "name"_a, "io_type"_a, "region_positions"_a)
        .def("with_output_auto_sorted", &nucleation::CircuitBuilder::with_output_auto_sorted, "name"_a, "io_type"_a, "region_positions"_a, "sort"_a)
        .def("with_output_sorted", &nucleation::CircuitBuilder::with_output_sorted, "name"_a, "io_type"_a, "layout"_a, "region_positions"_a, "sort"_a)
        .def("with_state_mode", &nucleation::CircuitBuilder::with_state_mode, "mode"_a);
}

} 