#ifndef NUCLEATION_ScatterFace_D_HPP
#define NUCLEATION_ScatterFace_D_HPP

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
    enum ScatterFace {
      ScatterFace_Top = 0,
      ScatterFace_Bottom = 1,
      ScatterFace_North = 2,
      ScatterFace_South = 3,
      ScatterFace_East = 4,
      ScatterFace_West = 5,
    };

    typedef struct ScatterFace_option {union { ScatterFace ok; }; bool is_ok; } ScatterFace_option;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * Face direction for surface scatter rules.
 */
class ScatterFace {
public:
    enum Value {
        Top = 0,
        Bottom = 1,
        North = 2,
        South = 3,
        East = 4,
        West = 5,
    };

    ScatterFace(): value(Value::Top) {}

    // Implicit conversions between enum and ::Value
    constexpr ScatterFace(Value v) : value(v) {}
    constexpr operator Value() const { return value; }
    // Prevent usage as boolean value
    explicit operator bool() const = delete;

    inline nucleation::capi::ScatterFace AsFFI() const;
    inline static nucleation::ScatterFace FromFFI(nucleation::capi::ScatterFace c_enum);
private:
    Value value;
};

} // namespace
#endif // NUCLEATION_ScatterFace_D_HPP
