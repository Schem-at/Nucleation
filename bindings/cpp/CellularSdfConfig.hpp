#ifndef CellularSdfConfig_HPP
#define CellularSdfConfig_HPP

#include "CellularSdfConfig.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "NucleationError.hpp"
#include "diplomat_runtime.hpp"


namespace diplomat {
namespace capi {
    extern "C" {

    typedef struct CellularSdfConfig_create_result {union {diplomat::capi::CellularSdfConfig* ok; diplomat::capi::NucleationError err;}; bool is_ok;} CellularSdfConfig_create_result;
    CellularSdfConfig_create_result CellularSdfConfig_create(int32_t cell_size_x, int32_t cell_size_z, uint64_t seed, float max_jitter_x, float max_jitter_z, float max_yaw_degrees, float min_scale, float max_scale, int32_t min_y_offset, int32_t max_y_offset, uint32_t presence_numerator, uint32_t presence_denominator, uint64_t feature_salt);

    void CellularSdfConfig_destroy(CellularSdfConfig* self);

    } // extern "C"
} // namespace capi
} // namespace

inline diplomat::result<std::unique_ptr<CellularSdfConfig>, NucleationError> CellularSdfConfig::create(int32_t cell_size_x, int32_t cell_size_z, uint64_t seed, float max_jitter_x, float max_jitter_z, float max_yaw_degrees, float min_scale, float max_scale, int32_t min_y_offset, int32_t max_y_offset, uint32_t presence_numerator, uint32_t presence_denominator, uint64_t feature_salt) {
    auto result = diplomat::capi::CellularSdfConfig_create(cell_size_x,
        cell_size_z,
        seed,
        max_jitter_x,
        max_jitter_z,
        max_yaw_degrees,
        min_scale,
        max_scale,
        min_y_offset,
        max_y_offset,
        presence_numerator,
        presence_denominator,
        feature_salt);
    return result.is_ok ? diplomat::result<std::unique_ptr<CellularSdfConfig>, NucleationError>(diplomat::Ok<std::unique_ptr<CellularSdfConfig>>(std::unique_ptr<CellularSdfConfig>(CellularSdfConfig::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<CellularSdfConfig>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline const diplomat::capi::CellularSdfConfig* CellularSdfConfig::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::CellularSdfConfig*>(this);
}

inline diplomat::capi::CellularSdfConfig* CellularSdfConfig::AsFFI() {
    return reinterpret_cast<diplomat::capi::CellularSdfConfig*>(this);
}

inline const CellularSdfConfig* CellularSdfConfig::FromFFI(const diplomat::capi::CellularSdfConfig* ptr) {
    return reinterpret_cast<const CellularSdfConfig*>(ptr);
}

inline CellularSdfConfig* CellularSdfConfig::FromFFI(diplomat::capi::CellularSdfConfig* ptr) {
    return reinterpret_cast<CellularSdfConfig*>(ptr);
}

inline void CellularSdfConfig::operator delete(void* ptr) {
    diplomat::capi::CellularSdfConfig_destroy(reinterpret_cast<diplomat::capi::CellularSdfConfig*>(ptr));
}


#endif // CellularSdfConfig_HPP
