#include "diplomat_nanobind_common.hpp"


#include "Design.hpp"
#include "Schematic.hpp"

namespace nucleation {
void add_Design_binding(nb::module_ mod) {
    PyType_Slot nucleation_Design_slots[] = {
        {Py_tp_free, (void *)nucleation::Design::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};

    nb::class_<nucleation::Design> opaque(mod, "Design", nb::type_slots(nucleation_Design_slots));
    opaque
        .def("add_cell", &nucleation::Design::add_cell, "name"_a, "cell"_a)
        .def("add_gate", &nucleation::Design::add_gate, "bus"_a, "gate"_a, "x"_a, "y"_a, "z"_a, "sx"_a, "sy"_a, "sz"_a)
        .def("bake", std::move(maybe_op_unwrap(&nucleation::Design::bake)), "budget"_a)
        .def("bus_skew", &nucleation::Design::bus_skew, "name"_a)
        .def("bus_state", &nucleation::Design::bus_state, "name"_a)
        .def("check", &nucleation::Design::check)
        .def_static("create", std::move(maybe_op_unwrap(&nucleation::Design::create)), "name"_a)
        .def("declare_input", &nucleation::Design::declare_input, "name"_a, "ax"_a, "ay"_a, "az"_a, "sx"_a, "sy"_a, "sz"_a, "width"_a, "ty"_a)
        .def("declare_output", &nucleation::Design::declare_output, "name"_a, "ax"_a, "ay"_a, "az"_a, "sx"_a, "sy"_a, "sz"_a, "width"_a, "ty"_a)
        .def("export_litematic", &nucleation::Design::export_litematic, "path"_a)
        .def("flatten", std::move(maybe_op_unwrap(&nucleation::Design::flatten)))
        .def_static("for_schematic", std::move(maybe_op_unwrap(&nucleation::Design::for_schematic)), "name"_a, "base"_a)
        .def_static("from_litematic", std::move(maybe_op_unwrap(&nucleation::Design::from_litematic)), "data"_a)
        .def_static("from_nucm", std::move(maybe_op_unwrap(&nucleation::Design::from_nucm)), "data"_a)
        .def_static("import_litematic", std::move(maybe_op_unwrap(&nucleation::Design::import_litematic)), "path"_a)
        .def_static("load_nucm", std::move(maybe_op_unwrap(&nucleation::Design::load_nucm)), "path"_a)
        .def("move_gate", &nucleation::Design::move_gate, "bus"_a, "gate"_a, "x"_a, "y"_a, "z"_a)
        .def("move_instance", &nucleation::Design::move_instance, "name"_a, "x"_a, "y"_a, "z"_a, "rot_y"_a)
        .def("place", &nucleation::Design::place, "name"_a, "cell"_a, "x"_a, "y"_a, "z"_a, "rot_y"_a)
        .def("rip", &nucleation::Design::rip, "name"_a)
        .def("route_bus", &nucleation::Design::route_bus, "name"_a, "driver"_a, "sinks_json"_a, "gates_json"_a, "style_json"_a)
        .def("route_bus_or", &nucleation::Design::route_bus_or, "name"_a, "drivers_json"_a, "sinks_json"_a, "gates_json"_a, "style_json"_a)
        .def("save_nucm", &nucleation::Design::save_nucm, "path"_a)
        .def("set_block", &nucleation::Design::set_block, "x"_a, "y"_a, "z"_a, "block"_a)
        .def("set_bus_rule", &nucleation::Design::set_bus_rule, "bus"_a, "rule_json"_a)
        .def("to_litematic_b64", &nucleation::Design::to_litematic_b64)
        .def("to_nucm_b64", &nucleation::Design::to_nucm_b64);
}

}
