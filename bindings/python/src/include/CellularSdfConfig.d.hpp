#ifndef NUCLEATION_CellularSdfConfig_D_HPP
#define NUCLEATION_CellularSdfConfig_D_HPP

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
namespace capi { struct CellularSdfConfig; }
class CellularSdfConfig;
class NucleationError;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct CellularSdfConfig;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * Immutable hashed-cell variation shared by coordinated SDF source layers.
 */
class CellularSdfConfig {
public:

  /**
   * Validates every field up front, so a config that constructs here is
   * never rejected for its own values by a later `cellular_sdf` call.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::CellularSdfConfig>, nucleation::NucleationError> create(int32_t cell_size_x, int32_t cell_size_z, uint64_t seed, float max_jitter_x, float max_jitter_z, float max_yaw_degrees, float min_scale, float max_scale, int32_t min_y_offset, int32_t max_y_offset, uint32_t presence_numerator, uint32_t presence_denominator, uint64_t feature_salt);

    inline const nucleation::capi::CellularSdfConfig* AsFFI() const;
    inline nucleation::capi::CellularSdfConfig* AsFFI();
    inline static const nucleation::CellularSdfConfig* FromFFI(const nucleation::capi::CellularSdfConfig* ptr);
    inline static nucleation::CellularSdfConfig* FromFFI(nucleation::capi::CellularSdfConfig* ptr);
    inline static void operator delete(void* ptr);
private:
    CellularSdfConfig() = delete;
    CellularSdfConfig(const nucleation::CellularSdfConfig&) = delete;
    CellularSdfConfig(nucleation::CellularSdfConfig&&) noexcept = delete;
    CellularSdfConfig operator=(const nucleation::CellularSdfConfig&) = delete;
    CellularSdfConfig operator=(nucleation::CellularSdfConfig&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_CellularSdfConfig_D_HPP
