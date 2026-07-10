#include "diplomat_nanobind_common.hpp"


#include "Diff.hpp"
#include "Schematic.hpp"

namespace nucleation {
void add_Diff_binding(nb::module_ mod) {
    PyType_Slot nucleation_Diff_slots[] = {
        {Py_tp_free, (void *)nucleation::Diff::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};
    
    nb::class_<nucleation::Diff> opaque(mod, "Diff", nb::type_slots(nucleation_Diff_slots));
    opaque
        .def("added", std::move(maybe_op_unwrap(&nucleation::Diff::added)))
        .def("changed", std::move(maybe_op_unwrap(&nucleation::Diff::changed)))
        .def_static("compute", std::move(maybe_op_unwrap(&nucleation::Diff::compute)), "a"_a, "b"_a, "preset"_a)
        .def_static("compute_with_opts", std::move(maybe_op_unwrap(&nucleation::Diff::compute_with_opts)), "a"_a, "b"_a, "preset"_a, "cost_add"_a, "cost_delete"_a, "cost_change"_a, "cost_swap"_a, "symmetry"_a)
        .def("distance", &nucleation::Diff::distance)
        .def_static("from_json", std::move(maybe_op_unwrap(&nucleation::Diff::from_json)), "json"_a)
        .def("markers", std::move(maybe_op_unwrap(&nucleation::Diff::markers)))
        .def("removed", std::move(maybe_op_unwrap(&nucleation::Diff::removed)))
        .def("summary_json", &nucleation::Diff::summary_json)
        .def("support", &nucleation::Diff::support)
        .def("swapped", std::move(maybe_op_unwrap(&nucleation::Diff::swapped)))
        .def("to_json", &nucleation::Diff::to_json)
        .def("to_overlay_glb_b64", &nucleation::Diff::to_overlay_glb_b64, "after_glb"_a);
}

} 