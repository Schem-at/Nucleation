#ifndef NUCLEATION_MeshConfig_D_HPP
#define NUCLEATION_MeshConfig_D_HPP

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
namespace capi { struct MeshConfig; }
class MeshConfig;
class NucleationError;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct MeshConfig;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * Mesh generation configuration. Wraps {@link crate::meshing::MeshConfig}.
 */
class MeshConfig {
public:

  inline static std::unique_ptr<nucleation::MeshConfig> create();

  inline void set_cull_hidden_faces(bool val);

  inline bool cull_hidden_faces() const;

  inline void set_ambient_occlusion(bool val);

  inline bool ambient_occlusion() const;

  inline void set_ao_intensity(float val);

  inline float ao_intensity() const;

  /**
   * Set the biome used for tinting (e.g. "plains", "swamp").
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> set_biome(std::string_view biome);

  /**
   * Clear the biome (old ABI: `meshconfig_set_biome(NULL)`).
   */
  inline void clear_biome();

  /**
   * The configured biome; `NotFound` if none is set.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> biome() const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> biome_write(W& writeable_output) const;

  inline void set_atlas_max_size(uint32_t size);

  inline uint32_t atlas_max_size() const;

  inline void set_cull_occluded_blocks(bool val);

  inline bool cull_occluded_blocks() const;

  inline void set_greedy_meshing(bool val);

  inline bool greedy_meshing() const;

    inline const nucleation::capi::MeshConfig* AsFFI() const;
    inline nucleation::capi::MeshConfig* AsFFI();
    inline static const nucleation::MeshConfig* FromFFI(const nucleation::capi::MeshConfig* ptr);
    inline static nucleation::MeshConfig* FromFFI(nucleation::capi::MeshConfig* ptr);
    inline static void operator delete(void* ptr);
private:
    MeshConfig() = delete;
    MeshConfig(const nucleation::MeshConfig&) = delete;
    MeshConfig(nucleation::MeshConfig&&) noexcept = delete;
    MeshConfig operator=(const nucleation::MeshConfig&) = delete;
    MeshConfig operator=(nucleation::MeshConfig&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_MeshConfig_D_HPP
