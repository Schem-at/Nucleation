#include "diplomat_nanobind_common.hpp"


#include "Palette.hpp"
#include "Voxelizer.hpp"

namespace nucleation {
void add_Voxelizer_binding(nb::module_ mod) {
    PyType_Slot nucleation_Voxelizer_slots[] = {
        {Py_tp_free, (void *)nucleation::Voxelizer::operator delete },
        {Py_tp_dealloc, (void *)diplomat_tp_dealloc},
        {0, nullptr}};
    
    nb::class_<nucleation::Voxelizer> opaque(mod, "Voxelizer", nb::type_slots(nucleation_Voxelizer_slots));
    opaque
        .def_static("schematic_from_glb_textured", std::move(maybe_op_unwrap(&nucleation::Voxelizer::schematic_from_glb_textured)), "data"_a, "target_size"_a, "shell"_a, "palette"_a, "name"_a)
        .def_static("shape_from_glb", std::move(maybe_op_unwrap(&nucleation::Voxelizer::shape_from_glb)), "data"_a, "target_size"_a, "shell"_a)
        .def_static("shape_from_obj", std::move(maybe_op_unwrap(&nucleation::Voxelizer::shape_from_obj)), "text"_a, "target_size"_a, "shell"_a);
}

} 