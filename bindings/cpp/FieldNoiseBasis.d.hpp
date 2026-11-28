#ifndef FieldNoiseBasis_D_HPP
#define FieldNoiseBasis_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"


namespace diplomat {
namespace capi {
    enum FieldNoiseBasis {
      FieldNoiseBasis_ValueNoise = 0,
      FieldNoiseBasis_Perlin = 1,
    };

    typedef struct FieldNoiseBasis_option {union { FieldNoiseBasis ok; }; bool is_ok; } FieldNoiseBasis_option;
} // namespace capi
} // namespace

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

    inline diplomat::capi::FieldNoiseBasis AsFFI() const;
    inline static FieldNoiseBasis FromFFI(diplomat::capi::FieldNoiseBasis c_enum);
private:
    Value value;
};


#endif // FieldNoiseBasis_D_HPP
