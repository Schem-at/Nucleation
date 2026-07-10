#ifndef NUCLEATION_MeshResult_D_HPP
#define NUCLEATION_MeshResult_D_HPP

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
namespace capi { struct MeshResult; }
class MeshResult;
namespace capi { struct ResourcePack; }
class ResourcePack;
namespace capi { struct Schematic; }
class Schematic;
struct MeshBounds;
class NucleationError;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct MeshResult;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * A single mesh output. Wraps {@link crate::meshing::MeshResult}.
 */
class MeshResult {
public:

  /**
   * Mesh an entire schematic in one pass (old ABI: `schematic_to_mesh`).
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::MeshResult>, nucleation::NucleationError> create(const nucleation::Schematic& schematic, const nucleation::ResourcePack& pack, const nucleation::MeshConfig& config);

  /**
   * Mesh a schematic with USDZ-compatible output (old ABI: `schematic_to_usdz`).
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::MeshResult>, nucleation::NucleationError> create_usdz(const nucleation::Schematic& schematic, const nucleation::ResourcePack& pack, const nucleation::MeshConfig& config);

  /**
   * The mesh as a binary GLB, base64-encoded.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> glb_data_b64() const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> glb_data_b64_write(W& writeable_output) const;

  /**
   * The mesh serialized in the NUCM cache format, base64-encoded.
   */
  inline std::string nucm_data_b64() const;
  template<typename W>
  inline void nucm_data_b64_write(W& writeable_output) const;

  inline uint32_t vertex_count() const;

  inline uint32_t triangle_count() const;

  inline bool has_transparency() const;

  inline nucleation::MeshBounds bounds() const;

    inline const nucleation::capi::MeshResult* AsFFI() const;
    inline nucleation::capi::MeshResult* AsFFI();
    inline static const nucleation::MeshResult* FromFFI(const nucleation::capi::MeshResult* ptr);
    inline static nucleation::MeshResult* FromFFI(nucleation::capi::MeshResult* ptr);
    inline static void operator delete(void* ptr);
private:
    MeshResult() = delete;
    MeshResult(const nucleation::MeshResult&) = delete;
    MeshResult(nucleation::MeshResult&&) noexcept = delete;
    MeshResult operator=(const nucleation::MeshResult&) = delete;
    MeshResult operator=(nucleation::MeshResult&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_MeshResult_D_HPP
