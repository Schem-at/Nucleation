#include "diplomat_nanobind_common.hpp"


#include "Field.hpp"
#include "Material.hpp"
#include "ScatterRule.hpp"
#include "Schematic.hpp"
#include "VoxelPipeline.hpp"

namespace nucleation {
void add_VoxelPipeline_binding(nb::module_ mod) {
    PyType_Slot nucleation_VoxelPipeline_slots[] = {
        {Py_tp_free, (void *)nucleation::VoxelPipeline::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};

    nb::class_<nucleation::VoxelPipeline> opaque(mod, "VoxelPipeline", nb::type_slots(nucleation_VoxelPipeline_slots));
    opaque
        .def("add_fluid", &nucleation::VoxelPipeline::add_fluid, "field"_a, "block"_a)
        .def("add_fluid_level", &nucleation::VoxelPipeline::add_fluid_level, "y_level"_a, "block"_a)
        .def("add_scatter", &nucleation::VoxelPipeline::add_scatter, "rule"_a)
        .def("apply_to_schematic", &nucleation::VoxelPipeline::apply_to_schematic, "schematic"_a)
        .def("apply_to_schematic_bounded", &nucleation::VoxelPipeline::apply_to_schematic_bounded, "schematic"_a, "min_x"_a, "min_y"_a, "min_z"_a, "max_x"_a, "max_y"_a, "max_z"_a)
        .def_static("create", std::move(maybe_op_unwrap(&nucleation::VoxelPipeline::create)), "geometry"_a, "material"_a)
        .def("set_fill_keep_existing", &nucleation::VoxelPipeline::set_fill_keep_existing)
        .def("set_fill_replace", &nucleation::VoxelPipeline::set_fill_replace)
        .def("set_fill_replacing", &nucleation::VoxelPipeline::set_fill_replacing, "targets_json"_a)
        .def("to_schematic", std::move(maybe_op_unwrap(&nucleation::VoxelPipeline::to_schematic)), "name"_a)
        .def("to_schematic_bounded", std::move(maybe_op_unwrap(&nucleation::VoxelPipeline::to_schematic_bounded)), "name"_a, "min_x"_a, "min_y"_a, "min_z"_a, "max_x"_a, "max_y"_a, "max_z"_a);
}

}
