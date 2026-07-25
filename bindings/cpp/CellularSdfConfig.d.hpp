#ifndef CellularSdfConfig_D_HPP
#define CellularSdfConfig_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"

class NucleationError;




namespace diplomat {
namespace capi {
    struct CellularSdfConfig;
} // namespace capi
} // namespace

/**
 * Immutable hashed-cell variation shared by coordinated SDF source layers.
 */
class CellularSdfConfig {
public:

  /**
   * Validates every field up front, so a config that constructs here is
   * never rejected for its own values by a later `cellular_sdf` call.
   */
  inline static diplomat::result<std::unique_ptr<CellularSdfConfig>, NucleationError> create(int32_t cell_size_x, int32_t cell_size_z, uint64_t seed, float max_jitter_x, float max_jitter_z, float max_yaw_degrees, float min_scale, float max_scale, int32_t min_y_offset, int32_t max_y_offset, uint32_t presence_numerator, uint32_t presence_denominator, uint64_t feature_salt);

    inline const diplomat::capi::CellularSdfConfig* AsFFI() const;
    inline diplomat::capi::CellularSdfConfig* AsFFI();
    inline static const CellularSdfConfig* FromFFI(const diplomat::capi::CellularSdfConfig* ptr);
    inline static CellularSdfConfig* FromFFI(diplomat::capi::CellularSdfConfig* ptr);
    inline static void operator delete(void* ptr);
private:
    CellularSdfConfig() = delete;
    CellularSdfConfig(const CellularSdfConfig&) = delete;
    CellularSdfConfig(CellularSdfConfig&&) noexcept = delete;
    CellularSdfConfig operator=(const CellularSdfConfig&) = delete;
    CellularSdfConfig operator=(CellularSdfConfig&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // CellularSdfConfig_D_HPP
