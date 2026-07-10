#include "diplomat_nanobind_common.hpp"


#include "ExecutionMode.hpp"
#include "IoLayout.hpp"
#include "MchprsWorld.hpp"
#include "Schematic.hpp"
#include "TypedCircuitExecutor.hpp"
#include "Value.hpp"

namespace nucleation {
void add_TypedCircuitExecutor_binding(nb::module_ mod) {
    PyType_Slot nucleation_TypedCircuitExecutor_slots[] = {
        {Py_tp_free, (void *)nucleation::TypedCircuitExecutor::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};
    
    nb::class_<nucleation::TypedCircuitExecutor> opaque(mod, "TypedCircuitExecutor", nb::type_slots(nucleation_TypedCircuitExecutor_slots));
    opaque
        .def("execute", &nucleation::TypedCircuitExecutor::execute, "inputs_json"_a, "mode"_a)
        .def("flush", &nucleation::TypedCircuitExecutor::flush)
        .def_static("from_insign", std::move(maybe_op_unwrap(&nucleation::TypedCircuitExecutor::from_insign)), "schematic"_a)
        .def_static("from_insign_with_options", std::move(maybe_op_unwrap(&nucleation::TypedCircuitExecutor::from_insign_with_options)), "schematic"_a, "optimize"_a, "io_only"_a)
        .def_static("from_layout", std::move(maybe_op_unwrap(&nucleation::TypedCircuitExecutor::from_layout)), "world"_a, "layout"_a)
        .def_static("from_layout_with_options", std::move(maybe_op_unwrap(&nucleation::TypedCircuitExecutor::from_layout_with_options)), "world"_a, "layout"_a, "optimize"_a, "io_only"_a)
        .def("input_names_json", &nucleation::TypedCircuitExecutor::input_names_json)
        .def("layout_info_json", &nucleation::TypedCircuitExecutor::layout_info_json)
        .def("output_names_json", &nucleation::TypedCircuitExecutor::output_names_json)
        .def("read_output", std::move(maybe_op_unwrap(&nucleation::TypedCircuitExecutor::read_output)), "name"_a)
        .def("reset", &nucleation::TypedCircuitExecutor::reset)
        .def("set_input", &nucleation::TypedCircuitExecutor::set_input, "name"_a, "value"_a)
        .def("set_state_mode", &nucleation::TypedCircuitExecutor::set_state_mode, "mode"_a)
        .def("sync_to_schematic", std::move(maybe_op_unwrap(&nucleation::TypedCircuitExecutor::sync_to_schematic)))
        .def("tick", &nucleation::TypedCircuitExecutor::tick, "ticks"_a);
}

} 