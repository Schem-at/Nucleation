#ifndef NUCLEATION_GeneratedChunkCoverage_HPP
#define NUCLEATION_GeneratedChunkCoverage_HPP

#include "GeneratedChunkCoverage.d.hpp"

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
namespace capi {

} // namespace capi
} // namespace

inline nucleation::capi::GeneratedChunkCoverage nucleation::GeneratedChunkCoverage::AsFFI() const {
    return static_cast<nucleation::capi::GeneratedChunkCoverage>(value);
}

inline nucleation::GeneratedChunkCoverage nucleation::GeneratedChunkCoverage::FromFFI(nucleation::capi::GeneratedChunkCoverage c_enum) {
    switch (c_enum) {
        case nucleation::capi::GeneratedChunkCoverage_Complete:
        case nucleation::capi::GeneratedChunkCoverage_Partial:
        case nucleation::capi::GeneratedChunkCoverage_Outside:
            return static_cast<nucleation::GeneratedChunkCoverage::Value>(c_enum);
        default:
            std::abort();
    }
}
#endif // NUCLEATION_GeneratedChunkCoverage_HPP
