#ifndef NUCLEATION_FieldNoiseBasis_D_HPP
#define NUCLEATION_FieldNoiseBasis_D_HPP

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
    enum FieldNoiseBasis {
      FieldNoiseBasis_ValueNoise = 0,
      FieldNoiseBasis_Perlin = 1,
    };

    typedef struct FieldNoiseBasis_option {union { FieldNoiseBasis ok; }; bool is_ok; } FieldNoiseBasis_option;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * Noise basis for procedural 2D/3D noise and FBM.
 */
class FieldNoiseBasis {
public:
    enum Value {
        ValueNoise = 0,
        Perlin = 1,
    };

    FieldNoiseBasis(): value(Value::ValueNoise) {}

    // Implicit conversions between enum and ::Value
    constexpr FieldNoiseBasis(Value v) : value(v) {}
    constexpr operator Value() const { return value; }
    // Prevent usage as boolean value
    explicit operator bool() const = delete;

    inline nucleation::capi::FieldNoiseBasis AsFFI() const;
    inline static nucleation::FieldNoiseBasis FromFFI(nucleation::capi::FieldNoiseBasis c_enum);
private:
    Value value;
};

} // namespace
#endif // NUCLEATION_FieldNoiseBasis_D_HPP
