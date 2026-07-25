#ifndef NUCLEATION_CellularSdfConfig_HPP
#define NUCLEATION_CellularSdfConfig_HPP

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


namespace nucleation {
namespace capi {
    extern "C" {

    typedef struct CellularSdfConfig_create_result {union {nucleation::capi::CellularSdfConfig* ok; nucleation::capi::NucleationError err;}; bool is_ok;} CellularSdfConfig_create_result;
    CellularSdfConfig_create_result CellularSdfConfig_create(int32_t cell_size_x, int32_t cell_size_z, uint64_t seed, float max_jitter_x, float max_jitter_z, float max_yaw_degrees, float min_scale, float max_scale, int32_t min_y_offset, int32_t max_y_offset, uint32_t presence_numerator, uint32_t presence_denominator, uint64_t feature_salt);

    void CellularSdfConfig_destroy(CellularSdfConfig* self);

    } // extern "C"
} // namespace capi
} // namespace

inline nucleation::diplomat::result<std::unique_ptr<nucleation::CellularSdfConfig>, nucleation::NucleationError> nucleation::CellularSdfConfig::create(int32_t cell_size_x, int32_t cell_size_z, uint64_t seed, float max_jitter_x, float max_jitter_z, float max_yaw_degrees, float min_scale, float max_scale, int32_t min_y_offset, int32_t max_y_offset, uint32_t presence_numerator, uint32_t presence_denominator, uint64_t feature_salt) {
    auto result = nucleation::capi::CellularSdfConfig_create(cell_size_x,
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
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::CellularSdfConfig>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::CellularSdfConfig>>(std::unique_ptr<nucleation::CellularSdfConfig>(nucleation::CellularSdfConfig::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::CellularSdfConfig>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline const nucleation::capi::CellularSdfConfig* nucleation::CellularSdfConfig::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::CellularSdfConfig*>(this);
}

inline nucleation::capi::CellularSdfConfig* nucleation::CellularSdfConfig::AsFFI() {
    return reinterpret_cast<nucleation::capi::CellularSdfConfig*>(this);
}

inline const nucleation::CellularSdfConfig* nucleation::CellularSdfConfig::FromFFI(const nucleation::capi::CellularSdfConfig* ptr) {
    return reinterpret_cast<const nucleation::CellularSdfConfig*>(ptr);
}

inline nucleation::CellularSdfConfig* nucleation::CellularSdfConfig::FromFFI(nucleation::capi::CellularSdfConfig* ptr) {
    return reinterpret_cast<nucleation::CellularSdfConfig*>(ptr);
}

inline void nucleation::CellularSdfConfig::operator delete(void* ptr) {
    nucleation::capi::CellularSdfConfig_destroy(reinterpret_cast<nucleation::capi::CellularSdfConfig*>(ptr));
}


#endif // NUCLEATION_CellularSdfConfig_HPP
