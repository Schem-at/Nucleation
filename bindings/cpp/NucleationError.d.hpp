#ifndef NucleationError_D_HPP
#define NucleationError_D_HPP

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
    enum NucleationError {
      NucleationError_NullArgument = 0,
      NucleationError_InvalidArgument = 1,
      NucleationError_Parse = 2,
      NucleationError_Serialize = 3,
      NucleationError_Io = 4,
      NucleationError_Lock = 5,
      NucleationError_Store = 6,
      NucleationError_Mesh = 7,
      NucleationError_Render = 8,
      NucleationError_Simulation = 9,
      NucleationError_AlreadyConsumed = 10,
      NucleationError_NotFound = 11,
    };

    typedef struct NucleationError_option {union { NucleationError ok; }; bool is_ok; } NucleationError_option;
} // namespace capi
} // namespace

/**
 * Every fallible method in the bridge returns `Result<T, NucleationError>` —
 * see `stencil/docs/nucleation-error.md` for how these variants were derived from
 * the three error conventions the old hand-written `ffi` module mixed.
 */
class NucleationError {
public:
    enum Value {
        NullArgument = 0,
        InvalidArgument = 1,
        Parse = 2,
        Serialize = 3,
        Io = 4,
        Lock = 5,
        Store = 6,
        Mesh = 7,
        Render = 8,
        Simulation = 9,
        AlreadyConsumed = 10,
        NotFound = 11,
    };

    NucleationError(): value(Value::NullArgument) {}

    // Implicit conversions between enum and ::Value
    constexpr NucleationError(Value v) : value(v) {}
    constexpr operator Value() const { return value; }
    // Prevent usage as boolean value
    explicit operator bool() const = delete;

    inline diplomat::capi::NucleationError AsFFI() const;
    inline static NucleationError FromFFI(diplomat::capi::NucleationError c_enum);
private:
    Value value;
};


#endif // NucleationError_D_HPP
