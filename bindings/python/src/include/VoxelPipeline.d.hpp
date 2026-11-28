#ifndef NUCLEATION_VoxelPipeline_D_HPP
#define NUCLEATION_VoxelPipeline_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"
namespace nucleation {
namespace capi { struct Field; }
class Field;
namespace capi { struct Material; }
class Material;
namespace capi { struct ScatterRule; }
class ScatterRule;
namespace capi { struct Schematic; }
class Schematic;
namespace capi { struct VoxelPipeline; }
class VoxelPipeline;
class NucleationError;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct VoxelPipeline;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * Unified brick-pruned, parallel voxelization pipeline.
 */
class VoxelPipeline {
public:

  inline static std::unique_ptr<nucleation::VoxelPipeline> create(const nucleation::Field& geometry, const nucleation::Material& material);

  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> add_fluid(const nucleation::Field& field, std::string_view block);

  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> add_fluid_level(float y_level, std::string_view block);

  inline void add_scatter(const nucleation::ScatterRule& rule);

  inline void set_fill_replace();

  inline void set_fill_keep_existing();

  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> set_fill_replacing(std::string_view targets_json);

  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> to_schematic(std::string_view name) const;

  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> to_schematic_bounded(std::string_view name, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z) const;

  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> apply_to_schematic(nucleation::Schematic& schematic) const;

  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> apply_to_schematic_bounded(nucleation::Schematic& schematic, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z) const;

    inline const nucleation::capi::VoxelPipeline* AsFFI() const;
    inline nucleation::capi::VoxelPipeline* AsFFI();
    inline static const nucleation::VoxelPipeline* FromFFI(const nucleation::capi::VoxelPipeline* ptr);
    inline static nucleation::VoxelPipeline* FromFFI(nucleation::capi::VoxelPipeline* ptr);
    inline static void operator delete(void* ptr);
private:
    VoxelPipeline() = delete;
    VoxelPipeline(const nucleation::VoxelPipeline&) = delete;
    VoxelPipeline(nucleation::VoxelPipeline&&) noexcept = delete;
    VoxelPipeline operator=(const nucleation::VoxelPipeline&) = delete;
    VoxelPipeline operator=(nucleation::VoxelPipeline&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_VoxelPipeline_D_HPP
