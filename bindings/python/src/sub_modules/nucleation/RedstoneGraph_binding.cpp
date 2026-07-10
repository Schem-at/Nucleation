#include "diplomat_nanobind_common.hpp"


#include "RedstoneGraph.hpp"

namespace nucleation {
void add_RedstoneGraph_binding(nb::module_ mod) {
    PyType_Slot nucleation_RedstoneGraph_slots[] = {
        {Py_tp_free, (void *)nucleation::RedstoneGraph::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};
    
    nb::class_<nucleation::RedstoneGraph> opaque(mod, "RedstoneGraph", nb::type_slots(nucleation_RedstoneGraph_slots));
    opaque
        .def("edge_count", &nucleation::RedstoneGraph::edge_count)
        .def("edges_json", &nucleation::RedstoneGraph::edges_json)
        .def("features_json", &nucleation::RedstoneGraph::features_json)
        .def("fingerprint", &nucleation::RedstoneGraph::fingerprint, "preset"_a)
        .def_static("from_json", std::move(maybe_op_unwrap(&nucleation::RedstoneGraph::from_json)), "json"_a)
        .def("node_count", &nucleation::RedstoneGraph::node_count)
        .def("nodes_json", &nucleation::RedstoneGraph::nodes_json)
        .def("to_json", &nucleation::RedstoneGraph::to_json);
}

} 