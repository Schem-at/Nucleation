#ifndef GeneratedChunkCoverage_D_HPP
#define GeneratedChunkCoverage_D_HPP

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
    enum GeneratedChunkCoverage {
      GeneratedChunkCoverage_Complete = 0,
      GeneratedChunkCoverage_Partial = 1,
      GeneratedChunkCoverage_Outside = 2,
    };

    typedef struct GeneratedChunkCoverage_option {union { GeneratedChunkCoverage ok; }; bool is_ok; } GeneratedChunkCoverage_option;
} // namespace capi
} // namespace

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

    inline diplomat::capi::GeneratedChunkCoverage AsFFI() const;
    inline static GeneratedChunkCoverage FromFFI(diplomat::capi::GeneratedChunkCoverage c_enum);
private:
    Value value;
};


#endif // GeneratedChunkCoverage_D_HPP
