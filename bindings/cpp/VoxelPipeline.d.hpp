#ifndef VoxelPipeline_D_HPP
#define VoxelPipeline_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"

namespace diplomat::capi { struct Field; }
class Field;
namespace diplomat::capi { struct Material; }
class Material;
namespace diplomat::capi { struct ScatterRule; }
class ScatterRule;
namespace diplomat::capi { struct Schematic; }
class Schematic;
class NucleationError;




namespace diplomat {
namespace capi {
    struct VoxelPipeline;
} // namespace capi
} // namespace

/**
 * Unified brick-pruned, parallel voxelization pipeline.
 */
class VoxelPipeline {
public:

  inline static std::unique_ptr<VoxelPipeline> create(const Field& geometry, const Material& material);

  inline diplomat::result<std::monostate, NucleationError> add_fluid(const Field& field, std::string_view block);

  inline diplomat::result<std::monostate, NucleationError> add_fluid_level(float y_level, std::string_view block);

  inline void add_scatter(const ScatterRule& rule);

  inline void set_fill_replace();

  inline void set_fill_keep_existing();

  inline diplomat::result<std::monostate, NucleationError> set_fill_replacing(std::string_view targets_json);

  inline diplomat::result<std::unique_ptr<Schematic>, NucleationError> to_schematic(std::string_view name) const;

  inline diplomat::result<std::unique_ptr<Schematic>, NucleationError> to_schematic_bounded(std::string_view name, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z) const;

  inline diplomat::result<std::monostate, NucleationError> apply_to_schematic(Schematic& schematic) const;

  inline diplomat::result<std::monostate, NucleationError> apply_to_schematic_bounded(Schematic& schematic, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z) const;

    inline const diplomat::capi::VoxelPipeline* AsFFI() const;
    inline diplomat::capi::VoxelPipeline* AsFFI();
    inline static const VoxelPipeline* FromFFI(const diplomat::capi::VoxelPipeline* ptr);
    inline static VoxelPipeline* FromFFI(diplomat::capi::VoxelPipeline* ptr);
    inline static void operator delete(void* ptr);
private:
    VoxelPipeline() = delete;
    VoxelPipeline(const VoxelPipeline&) = delete;
    VoxelPipeline(VoxelPipeline&&) noexcept = delete;
    VoxelPipeline operator=(const VoxelPipeline&) = delete;
    VoxelPipeline operator=(VoxelPipeline&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // VoxelPipeline_D_HPP
