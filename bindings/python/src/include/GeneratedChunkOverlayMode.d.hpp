#ifndef NUCLEATION_GeneratedChunkOverlayMode_D_HPP
#define NUCLEATION_GeneratedChunkOverlayMode_D_HPP

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
    enum GeneratedChunkOverlayMode {
      GeneratedChunkOverlayMode_Replace = 0,
      GeneratedChunkOverlayMode_KeepExisting = 1,
    };

    typedef struct GeneratedChunkOverlayMode_option {union { GeneratedChunkOverlayMode ok; }; bool is_ok; } GeneratedChunkOverlayMode_option;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * How a composite layer treats non-air blocks already emitted by earlier layers.
 */
class GeneratedChunkOverlayMode {
public:
    enum Value {
        Replace = 0,
        KeepExisting = 1,
    };

    GeneratedChunkOverlayMode(): value(Value::Replace) {}

    // Implicit conversions between enum and ::Value
    constexpr GeneratedChunkOverlayMode(Value v) : value(v) {}
    constexpr operator Value() const { return value; }
    // Prevent usage as boolean value
    explicit operator bool() const = delete;

    inline nucleation::capi::GeneratedChunkOverlayMode AsFFI() const;
    inline static nucleation::GeneratedChunkOverlayMode FromFFI(nucleation::capi::GeneratedChunkOverlayMode c_enum);
private:
    Value value;
};

} // namespace
#endif // NUCLEATION_GeneratedChunkOverlayMode_D_HPP
