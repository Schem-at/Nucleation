#include "diplomat_nanobind_common.hpp"


#include "Routing.hpp"
#include "Schematic.hpp"

namespace nucleation {
void add_Routing_binding(nb::module_ mod) {
    PyType_Slot nucleation_Routing_slots[] = {
        {Py_tp_free, (void *)nucleation::Routing::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};

    nb::class_<nucleation::Routing> opaque(mod, "Routing", nb::type_slots(nucleation_Routing_slots));
    opaque
        .def_static("drc", &nucleation::Routing::drc, "schematic"_a, "check_decay"_a)
        .def_static("lvs", &nucleation::Routing::lvs, "schematic"_a, "intent_json"_a)
        .def_static("route_all", &nucleation::Routing::route_all, "schematic"_a, "nets_json"_a)
        .def_static("route_net", &nucleation::Routing::route_net, "schematic"_a, "sx"_a, "sy"_a, "sz"_a, "dx"_a, "dy"_a, "dz"_a, "label"_a)
        .def_static("sta", &nucleation::Routing::sta, "schematic"_a, "netlist_json"_a);
}

}
