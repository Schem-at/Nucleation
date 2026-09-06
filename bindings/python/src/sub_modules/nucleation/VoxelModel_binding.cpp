#include "diplomat_nanobind_common.hpp"


#include "Palette.hpp"
#include "VoxelModel.hpp"

namespace nucleation {
void add_VoxelModel_binding(nb::module_ mod) {
    PyType_Slot nucleation_VoxelModel_slots[] = {
        {Py_tp_free, (void *)nucleation::VoxelModel::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};

    nb::class_<nucleation::VoxelModel> opaque(mod, "VoxelModel", nb::type_slots(nucleation_VoxelModel_slots));
    opaque
        .def("plan_json", &nucleation::VoxelModel::plan_json, "options_json"_a)
        .def("to_schematic", std::move(maybe_op_unwrap(&nucleation::VoxelModel::to_schematic)), "options_json"_a, "palette"_a, "name"_a);
}

}
