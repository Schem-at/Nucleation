#ifndef TickSettleMode_D_HPP
#define TickSettleMode_D_HPP

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
    enum TickSettleMode {
      TickSettleMode_Placement = 0,
      TickSettleMode_Quiet = 1,
      TickSettleMode_InWorld = 2,
    };

    typedef struct TickSettleMode_option {union { TickSettleMode ok; }; bool is_ok; } TickSettleMode_option;
} // namespace capi
} // namespace

/**
 * How the loaded structure is settled before tick 0.
 */
class TickSettleMode {
public:
    enum Value {
        /**
         * Vanilla placement pass + ordered settle — a build saved at rest.
         */
        Placement = 0,
        /**
         * `onPlace` only, no settle — a knownShape capture.
         */
        Quiet = 1,
        /**
         * Neither — a build recorded mid-state in the world it stood in.
         */
        InWorld = 2,
    };

    TickSettleMode(): value(Value::Placement) {}

    // Implicit conversions between enum and ::Value
    constexpr TickSettleMode(Value v) : value(v) {}
    constexpr operator Value() const { return value; }
    // Prevent usage as boolean value
    explicit operator bool() const = delete;

    inline diplomat::capi::TickSettleMode AsFFI() const;
    inline static TickSettleMode FromFFI(diplomat::capi::TickSettleMode c_enum);
private:
    Value value;
};


#endif // TickSettleMode_D_HPP
