#ifndef NUCLEATION_GeneratedChunkCoverage_D_HPP
#define NUCLEATION_GeneratedChunkCoverage_D_HPP

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
    enum GeneratedChunkCoverage {
      GeneratedChunkCoverage_Complete = 0,
      GeneratedChunkCoverage_Partial = 1,
      GeneratedChunkCoverage_Outside = 2,
    };

    typedef struct GeneratedChunkCoverage_option {union { GeneratedChunkCoverage ok; }; bool is_ok; } GeneratedChunkCoverage_option;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * Coverage of a generated chunk by its source graph.
 */
class GeneratedChunkCoverage {
public:
    enum Value {
        Complete = 0,
        Partial = 1,
        Outside = 2,
    };

    GeneratedChunkCoverage(): value(Value::Complete) {}

    // Implicit conversions between enum and ::Value
    constexpr GeneratedChunkCoverage(Value v) : value(v) {}
    constexpr operator Value() const { return value; }
    // Prevent usage as boolean value
    explicit operator bool() const = delete;

    inline nucleation::capi::GeneratedChunkCoverage AsFFI() const;
    inline static nucleation::GeneratedChunkCoverage FromFFI(nucleation::capi::GeneratedChunkCoverage c_enum);
private:
    Value value;
};

} // namespace
#endif // NUCLEATION_GeneratedChunkCoverage_D_HPP
