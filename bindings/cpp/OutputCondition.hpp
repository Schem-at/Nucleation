#ifndef OutputCondition_HPP
#define OutputCondition_HPP

#include "OutputCondition.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "Value.hpp"
#include "diplomat_runtime.hpp"


namespace diplomat {
namespace capi {
    extern "C" {

    diplomat::capi::OutputCondition* OutputCondition_equals(const diplomat::capi::Value* value);

    diplomat::capi::OutputCondition* OutputCondition_not_equals(const diplomat::capi::Value* value);

    diplomat::capi::OutputCondition* OutputCondition_greater_than(const diplomat::capi::Value* value);

    diplomat::capi::OutputCondition* OutputCondition_less_than(const diplomat::capi::Value* value);

    diplomat::capi::OutputCondition* OutputCondition_bitwise_and(uint32_t mask);

    void OutputCondition_destroy(OutputCondition* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::unique_ptr<OutputCondition> OutputCondition::equals(const Value& value) {
    auto result = diplomat::capi::OutputCondition_equals(value.AsFFI());
    return std::unique_ptr<OutputCondition>(OutputCondition::FromFFI(result));
}

inline std::unique_ptr<OutputCondition> OutputCondition::not_equals(const Value& value) {
    auto result = diplomat::capi::OutputCondition_not_equals(value.AsFFI());
    return std::unique_ptr<OutputCondition>(OutputCondition::FromFFI(result));
}

inline std::unique_ptr<OutputCondition> OutputCondition::greater_than(const Value& value) {
    auto result = diplomat::capi::OutputCondition_greater_than(value.AsFFI());
    return std::unique_ptr<OutputCondition>(OutputCondition::FromFFI(result));
}

inline std::unique_ptr<OutputCondition> OutputCondition::less_than(const Value& value) {
    auto result = diplomat::capi::OutputCondition_less_than(value.AsFFI());
    return std::unique_ptr<OutputCondition>(OutputCondition::FromFFI(result));
}

inline std::unique_ptr<OutputCondition> OutputCondition::bitwise_and(uint32_t mask) {
    auto result = diplomat::capi::OutputCondition_bitwise_and(mask);
    return std::unique_ptr<OutputCondition>(OutputCondition::FromFFI(result));
}

inline const diplomat::capi::OutputCondition* OutputCondition::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::OutputCondition*>(this);
}

inline diplomat::capi::OutputCondition* OutputCondition::AsFFI() {
    return reinterpret_cast<diplomat::capi::OutputCondition*>(this);
}

inline const OutputCondition* OutputCondition::FromFFI(const diplomat::capi::OutputCondition* ptr) {
    return reinterpret_cast<const OutputCondition*>(ptr);
}

inline OutputCondition* OutputCondition::FromFFI(diplomat::capi::OutputCondition* ptr) {
    return reinterpret_cast<OutputCondition*>(ptr);
}

inline void OutputCondition::operator delete(void* ptr) {
    diplomat::capi::OutputCondition_destroy(reinterpret_cast<diplomat::capi::OutputCondition*>(ptr));
}


#endif // OutputCondition_HPP
