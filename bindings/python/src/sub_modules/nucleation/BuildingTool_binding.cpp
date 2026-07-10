#include "diplomat_nanobind_common.hpp"


#include "Brush.hpp"
#include "BuildingTool.hpp"
#include "Schematic.hpp"
#include "Shape.hpp"

namespace nucleation {
void add_BuildingTool_binding(nb::module_ mod) {
    PyType_Slot nucleation_BuildingTool_slots[] = {
        {Py_tp_free, (void *)nucleation::BuildingTool::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};
    
    nb::class_<nucleation::BuildingTool> opaque(mod, "BuildingTool", nb::type_slots(nucleation_BuildingTool_slots));
    opaque
        .def_static("fill", &nucleation::BuildingTool::fill, "schematic"_a, "shape"_a, "brush"_a)
        .def_static("rstack", &nucleation::BuildingTool::rstack, "schematic"_a, "shape"_a, "brush"_a, "count"_a, "offset_x"_a, "offset_y"_a, "offset_z"_a);
}

} 