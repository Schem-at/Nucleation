#include "diplomat_nanobind_common.hpp"


#include "Schematic.hpp"
#include "TickSettleMode.hpp"
#include "TickSimulation.hpp"

namespace nucleation {
void add_TickSimulation_binding(nb::module_ mod) {
    PyType_Slot nucleation_TickSimulation_slots[] = {
        {Py_tp_free, (void *)nucleation::TickSimulation::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};

    nb::class_<nucleation::TickSimulation> opaque(mod, "TickSimulation", nb::type_slots(nucleation_TickSimulation_slots));
    opaque
        .def("changes_count", &nucleation::TickSimulation::changes_count)
        .def("changes_json", &nucleation::TickSimulation::changes_json)
        .def("checkpoint", &nucleation::TickSimulation::checkpoint)
        .def("events_summary_json", &nucleation::TickSimulation::events_summary_json)
        .def_static("from_schematic", std::move(maybe_op_unwrap(&nucleation::TickSimulation::from_schematic)), "schematic"_a, "settle"_a, "origin_x"_a, "origin_y"_a, "origin_z"_a, "extra_states"_a)
        .def_static("from_snbt", std::move(maybe_op_unwrap(&nucleation::TickSimulation::from_snbt)), "snbt"_a, "settle"_a, "origin_x"_a, "origin_y"_a, "origin_z"_a, "extra_states"_a)
        .def_static("gametest_snbt", &nucleation::TickSimulation::gametest_snbt, "schematic"_a)
        .def("get_block", &nucleation::TickSimulation::get_block, "x"_a, "y"_a, "z"_a)
        .def("is_quiescent", &nucleation::TickSimulation::is_quiescent)
        .def("item_entities_json", &nucleation::TickSimulation::item_entities_json)
        .def("non_air_center_x", &nucleation::TickSimulation::non_air_center_x)
        .def("non_air_count", &nucleation::TickSimulation::non_air_count)
        .def("non_air_max_x", &nucleation::TickSimulation::non_air_max_x)
        .def("non_air_min_x", &nucleation::TickSimulation::non_air_min_x)
        .def("place_block", &nucleation::TickSimulation::place_block, "x"_a, "y"_a, "z"_a, "state"_a)
        .def("restore", &nucleation::TickSimulation::restore, "id"_a)
        .def("run", &nucleation::TickSimulation::run, "ticks"_a)
        .def("run_until_quiescent", &nucleation::TickSimulation::run_until_quiescent, "budget"_a)
        .def("set_rng_seed", &nucleation::TickSimulation::set_rng_seed, "seed"_a)
        .def("step", &nucleation::TickSimulation::step)
        .def("tick_count", &nucleation::TickSimulation::tick_count)
        .def("use_block", &nucleation::TickSimulation::use_block, "x"_a, "y"_a, "z"_a)
        .def("world_snapshot_json", &nucleation::TickSimulation::world_snapshot_json);
}

}
