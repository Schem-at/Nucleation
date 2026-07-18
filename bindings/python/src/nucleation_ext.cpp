#include "diplomat_nanobind_common.hpp"

// Forward declarations for binding add functions
namespace nucleation{
  
void add_RegionBounds_binding(nb::module_);
void add_ItemScale_binding(nb::module_);
void add_MeshBounds_binding(nb::module_);
void add_MeshProgress_binding(nb::module_);
void add_TextureInfo_binding(nb::module_);
void add_BlockPos_binding(nb::module_);
void add_Dimensions_binding(nb::module_);
void add_Autostack_binding(nb::module_);
void add_Blocks_binding(nb::module_);
void add_Brush_binding(nb::module_);
void add_BuildingTool_binding(nb::module_);
void add_Palette_binding(nb::module_);
void add_PaletteBuilder_binding(nb::module_);
void add_Shape_binding(nb::module_);
void add_DefinitionRegion_binding(nb::module_);
void add_SchematicRegions_binding(nb::module_);
void add_Diff_binding(nb::module_);
void add_Fingerprint_binding(nb::module_);
void add_ChunkMeshResult_binding(nb::module_);
void add_ItemModelConfig_binding(nb::module_);
void add_ItemModelPackBuilder_binding(nb::module_);
void add_ItemModelResult_binding(nb::module_);
void add_MeshConfig_binding(nb::module_);
void add_MeshJob_binding(nb::module_);
void add_MeshResult_binding(nb::module_);
void add_MultiMeshResult_binding(nb::module_);
void add_RawMeshExport_binding(nb::module_);
void add_ResourcePack_binding(nb::module_);
void add_ResourcePackList_binding(nb::module_);
void add_TextureAtlas_binding(nb::module_);
void add_Nbt_binding(nb::module_);
void add_RenderConfig_binding(nb::module_);
void add_Renderer_binding(nb::module_);
void add_BlockState_binding(nb::module_);
void add_Schematic_binding(nb::module_);
void add_SchematicBuilder_binding(nb::module_);
void add_Scripting_binding(nb::module_);
void add_Sdf_binding(nb::module_);
void add_CircuitBuilder_binding(nb::module_);
void add_ExecutionMode_binding(nb::module_);
void add_IoLayout_binding(nb::module_);
void add_IoLayoutBuilder_binding(nb::module_);
void add_IoType_binding(nb::module_);
void add_LayoutFunction_binding(nb::module_);
void add_MchprsWorld_binding(nb::module_);
void add_OutputCondition_binding(nb::module_);
void add_RedstoneGraph_binding(nb::module_);
void add_SortStrategy_binding(nb::module_);
void add_TypedCircuitExecutor_binding(nb::module_);
void add_Value_binding(nb::module_);
void add_Store_binding(nb::module_);
void add_StoreIo_binding(nb::module_);
void add_Voxelizer_binding(nb::module_);
void add_WorldChunkView_binding(nb::module_);
void add_WorldSink_binding(nb::module_);
void add_WorldStream_binding(nb::module_);
void add_InterpolationSpace_binding(nb::module_);
void add_MeshPhase_binding(nb::module_);
void add_NucleationError_binding(nb::module_);
}

// Nanobind does not usually support custom deleters, so we're shimming some of the machinery to add that ability.
// On module init, the dummy type will have the normal nanobind inst_dealloc function in the tp_dealloc slot, so we
// pull it out, store it here, and then call it in the tp_dealloc function we are shimming in to all our types.
// Our custom tp_dealloc function will call the tp_free function instead of `delete`, allowing us effectively to override
// the delete operator. Everything below goes through nanobind's public low-level instance API
// (inst_state/inst_destruct/inst_set_state/inst_ptr), so it does not depend on the private nb_inst layout
// (which changed in nanobind 2.13) and works on nanobind >= 2.12.
// See https://nanobind.readthedocs.io/en/latest/lowlevel.html#customizing-type-creation and
// https://github.com/wjakob/nanobind/discussions/932
void (*nb_tp_dealloc)(void *) = nullptr;

void diplomat_tp_dealloc(PyObject *self)
{
    nb::handle h(self);

    // inst_state() returns {ready, destruct}. Diplomat opaques always live in
    // Rust-allocated memory (the payload is never co-located with the Python
    // object), so `destruct` is set exactly when nanobind owns the instance
    // and would call `operator delete` on the payload during its dealloc.
    const bool owned = nb::inst_state(h).second;
    if (owned)
    {
        void *p = nb::inst_ptr<void>(h);

        // Run the C++ destructor (if the type has a non-trivial one) and
        // clear the `destruct` flag through the supported API.
        nb::inst_destruct(h);

        // Clear the remaining ownership state. For instances whose payload is
        // not co-located with the Python object, inst_set_state() derives
        // nanobind's internal "free the payload" flag from `destruct`, so
        // passing destruct=false guarantees the chained nanobind dealloc
        // below neither destroys nor frees the payload again.
        nb::inst_set_state(h, /* ready */ false, /* destruct */ false);

        // Free through the type's Py_tp_free slot, which every diplomat
        // opaque binding points at `Type::operator delete` - i.e. the FFI
        // destroy function that releases the Rust allocation.
        auto tp_free = (freefunc)(PyType_GetSlot(Py_TYPE(self), Py_tp_free));
        (*tp_free)(p);
    }
    (*nb_tp_dealloc)(self);
}

struct _Dummy {};

NB_MODULE(nucleation, mod)
{
    using namespace nucleation;

    {
        nb::class_<_Dummy> dummy(mod, "__dummy__");
        nb_tp_dealloc = (void (*)(void *))nb::type_get_slot(dummy, Py_tp_dealloc);
    }

    nb::class_<std::monostate>(mod, "monostate")
        .def("__repr__", [](const std::monostate &)
             { return ""; })
        .def("__str__", [](const std::monostate &)
             { return ""; });// Module declarations
    // Add bindings
    add_RegionBounds_binding(mod);
    add_ItemScale_binding(mod);
    add_MeshBounds_binding(mod);
    add_MeshProgress_binding(mod);
    add_TextureInfo_binding(mod);
    add_BlockPos_binding(mod);
    add_Dimensions_binding(mod);
    add_Autostack_binding(mod);
    add_Blocks_binding(mod);
    add_Brush_binding(mod);
    add_BuildingTool_binding(mod);
    add_Palette_binding(mod);
    add_PaletteBuilder_binding(mod);
    add_Shape_binding(mod);
    add_DefinitionRegion_binding(mod);
    add_SchematicRegions_binding(mod);
    add_Diff_binding(mod);
    add_Fingerprint_binding(mod);
    add_ChunkMeshResult_binding(mod);
    add_ItemModelConfig_binding(mod);
    add_ItemModelPackBuilder_binding(mod);
    add_ItemModelResult_binding(mod);
    add_MeshConfig_binding(mod);
    add_MeshJob_binding(mod);
    add_MeshResult_binding(mod);
    add_MultiMeshResult_binding(mod);
    add_RawMeshExport_binding(mod);
    add_ResourcePack_binding(mod);
    add_ResourcePackList_binding(mod);
    add_TextureAtlas_binding(mod);
    add_Nbt_binding(mod);
    add_RenderConfig_binding(mod);
    add_Renderer_binding(mod);
    add_BlockState_binding(mod);
    add_Schematic_binding(mod);
    add_SchematicBuilder_binding(mod);
    add_Scripting_binding(mod);
    add_Sdf_binding(mod);
    add_CircuitBuilder_binding(mod);
    add_ExecutionMode_binding(mod);
    add_IoLayout_binding(mod);
    add_IoLayoutBuilder_binding(mod);
    add_IoType_binding(mod);
    add_LayoutFunction_binding(mod);
    add_MchprsWorld_binding(mod);
    add_OutputCondition_binding(mod);
    add_RedstoneGraph_binding(mod);
    add_SortStrategy_binding(mod);
    add_TypedCircuitExecutor_binding(mod);
    add_Value_binding(mod);
    add_Store_binding(mod);
    add_StoreIo_binding(mod);
    add_Voxelizer_binding(mod);
    add_WorldChunkView_binding(mod);
    add_WorldSink_binding(mod);
    add_WorldStream_binding(mod);
    add_InterpolationSpace_binding(mod);
    add_MeshPhase_binding(mod);
    add_NucleationError_binding(mod);
    
    
}