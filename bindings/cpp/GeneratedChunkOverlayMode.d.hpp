#ifndef GeneratedChunkOverlayMode_D_HPP
#define GeneratedChunkOverlayMode_D_HPP

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
    enum GeneratedChunkOverlayMode {
      GeneratedChunkOverlayMode_Replace = 0,
      GeneratedChunkOverlayMode_KeepExisting = 1,
    };

    typedef struct GeneratedChunkOverlayMode_option {union { GeneratedChunkOverlayMode ok; }; bool is_ok; } GeneratedChunkOverlayMode_option;
} // namespace capi
} // namespace

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

    inline diplomat::capi::GeneratedChunkOverlayMode AsFFI() const;
    inline static GeneratedChunkOverlayMode FromFFI(diplomat::capi::GeneratedChunkOverlayMode c_enum);
private:
    Value value;
};


#endif // GeneratedChunkOverlayMode_D_HPP
