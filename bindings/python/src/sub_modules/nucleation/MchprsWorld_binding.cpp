#include "diplomat_nanobind_common.hpp"


#include "MchprsWorld.hpp"
#include "Schematic.hpp"

namespace nucleation {
void add_MchprsWorld_binding(nb::module_ mod) {
    PyType_Slot nucleation_MchprsWorld_slots[] = {
        {Py_tp_free, (void *)nucleation::MchprsWorld::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};
    
    nb::class_<nucleation::MchprsWorld> opaque(mod, "MchprsWorld", nb::type_slots(nucleation_MchprsWorld_slots));
    opaque
        .def("check_custom_io_changes", &nucleation::MchprsWorld::check_custom_io_changes)
        .def("clear_custom_io_changes", &nucleation::MchprsWorld::clear_custom_io_changes)
        .def_static("create", std::move(maybe_op_unwrap(&nucleation::MchprsWorld::create)), "schematic"_a)
        .def_static("create_with_custom_io", std::move(maybe_op_unwrap(&nucleation::MchprsWorld::create_with_custom_io)), "schematic"_a, "optimize"_a, "io_only"_a, "custom_io_positions"_a)
        .def_static("create_with_options", std::move(maybe_op_unwrap(&nucleation::MchprsWorld::create_with_options)), "schematic"_a, "optimize"_a, "io_only"_a)
        .def("export_graph", std::move(maybe_op_unwrap(&nucleation::MchprsWorld::export_graph)))
        .def("export_graph_structural", std::move(maybe_op_unwrap(&nucleation::MchprsWorld::export_graph_structural)))
        .def("flush", &nucleation::MchprsWorld::flush)
        .def("get_lever_power", &nucleation::MchprsWorld::get_lever_power, "x"_a, "y"_a, "z"_a)
        .def("get_redstone_power", &nucleation::MchprsWorld::get_redstone_power, "x"_a, "y"_a, "z"_a)
        .def("get_schematic", std::move(maybe_op_unwrap(&nucleation::MchprsWorld::get_schematic)))
        .def("get_signal_strength", &nucleation::MchprsWorld::get_signal_strength, "x"_a, "y"_a, "z"_a)
        .def("is_lit", &nucleation::MchprsWorld::is_lit, "x"_a, "y"_a, "z"_a)
        .def("on_use_block", &nucleation::MchprsWorld::on_use_block, "x"_a, "y"_a, "z"_a)
        .def("peek_custom_io_changes_json", &nucleation::MchprsWorld::peek_custom_io_changes_json)
        .def("poll_custom_io_changes_json", &nucleation::MchprsWorld::poll_custom_io_changes_json)
        .def("set_lever_power", &nucleation::MchprsWorld::set_lever_power, "x"_a, "y"_a, "z"_a, "powered"_a)
        .def("set_signal_strength", &nucleation::MchprsWorld::set_signal_strength, "x"_a, "y"_a, "z"_a, "strength"_a)
        .def_static("simulate_use_block", std::move(maybe_op_unwrap(&nucleation::MchprsWorld::simulate_use_block)), "schematic"_a, "ticks"_a, "events_xyz"_a)
        .def("sync_to_schematic", &nucleation::MchprsWorld::sync_to_schematic)
        .def("tick", &nucleation::MchprsWorld::tick, "ticks"_a);
}

} 