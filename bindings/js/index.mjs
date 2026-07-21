

export { RegionBounds } from "./RegionBounds.mjs"

export { ItemScale } from "./ItemScale.mjs"

export { MeshBounds } from "./MeshBounds.mjs"

export { MeshProgress } from "./MeshProgress.mjs"

export { TextureInfo } from "./TextureInfo.mjs"

export { BlockPos } from "./BlockPos.mjs"

export { Dimensions } from "./Dimensions.mjs"

export { AnimationEffect } from "./AnimationEffect.mjs"

export { BuildAnimation } from "./BuildAnimation.mjs"

export { Autostack } from "./Autostack.mjs"

export { Blocks } from "./Blocks.mjs"

export { Brush } from "./Brush.mjs"

export { BuildingTool } from "./BuildingTool.mjs"

export { Curve3D } from "./Curve3D.mjs"

export { Palette } from "./Palette.mjs"

export { PaletteBuilder } from "./PaletteBuilder.mjs"

export { Shape } from "./Shape.mjs"

export { DefinitionRegion } from "./DefinitionRegion.mjs"

export { SchematicRegions } from "./SchematicRegions.mjs"

export { Diff } from "./Diff.mjs"

export { Fingerprint } from "./Fingerprint.mjs"

export { DistanceField } from "./DistanceField.mjs"

export { Geo } from "./Geo.mjs"

export { ChunkMeshResult } from "./ChunkMeshResult.mjs"

export { ItemModelConfig } from "./ItemModelConfig.mjs"

export { ItemModelPackBuilder } from "./ItemModelPackBuilder.mjs"

export { ItemModelResult } from "./ItemModelResult.mjs"

export { MeshConfig } from "./MeshConfig.mjs"

export { MeshJob } from "./MeshJob.mjs"

export { MeshResult } from "./MeshResult.mjs"

export { MultiMeshResult } from "./MultiMeshResult.mjs"

export { RawMeshExport } from "./RawMeshExport.mjs"

export { ResourcePack } from "./ResourcePack.mjs"

export { ResourcePackList } from "./ResourcePackList.mjs"

export { TextureAtlas } from "./TextureAtlas.mjs"

export { Nbt } from "./Nbt.mjs"

export { RenderConfig } from "./RenderConfig.mjs"

export { Renderer } from "./Renderer.mjs"

export { BlockState } from "./BlockState.mjs"

export { Schematic } from "./Schematic.mjs"

export { SchematicBuilder } from "./SchematicBuilder.mjs"

export { Scripting } from "./Scripting.mjs"

export { Sdf } from "./Sdf.mjs"

export { CircuitBuilder } from "./CircuitBuilder.mjs"

export { ExecutionMode } from "./ExecutionMode.mjs"

export { IoLayout } from "./IoLayout.mjs"

export { IoLayoutBuilder } from "./IoLayoutBuilder.mjs"

export { IoType } from "./IoType.mjs"

export { LayoutFunction } from "./LayoutFunction.mjs"

export { MchprsWorld } from "./MchprsWorld.mjs"

export { OutputCondition } from "./OutputCondition.mjs"

export { RedstoneGraph } from "./RedstoneGraph.mjs"

export { SortStrategy } from "./SortStrategy.mjs"

export { TypedCircuitExecutor } from "./TypedCircuitExecutor.mjs"

export { Value } from "./Value.mjs"

export { Store } from "./Store.mjs"

export { StoreIo } from "./StoreIo.mjs"

export { Voxelizer } from "./Voxelizer.mjs"

export { WorldChunkView } from "./WorldChunkView.mjs"

export { WorldSink } from "./WorldSink.mjs"

export { WorldStream } from "./WorldStream.mjs"

export { InterpolationSpace } from "./InterpolationSpace.mjs"

export { MeshPhase } from "./MeshPhase.mjs"

export { NucleationError } from "./NucleationError.mjs"

import wasm from "./diplomat-wasm.mjs";
import {FUNCTION_PARAM_ALLOC, internalConstructor} from "./diplomat-runtime.mjs";

FUNCTION_PARAM_ALLOC.reserve(internalConstructor, wasm, 52);
