#ifndef NUCLEATION_OutputCondition_HPP
#define NUCLEATION_OutputCondition_HPP

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


namespace nucleation {
namespace capi {
    extern "C" {

    nucleation::capi::OutputCondition* OutputCondition_equals(const nucleation::capi::Value* value);

    nucleation::capi::OutputCondition* OutputCondition_not_equals(const nucleation::capi::Value* value);

    nucleation::capi::OutputCondition* OutputCondition_greater_than(const nucleation::capi::Value* value);

    nucleation::capi::OutputCondition* OutputCondition_less_than(const nucleation::capi::Value* value);

    nucleation::capi::OutputCondition* OutputCondition_bitwise_and(uint32_t mask);

    void OutputCondition_destroy(OutputCondition* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::unique_ptr<nucleation::OutputCondition> nucleation::OutputCondition::equals(const nucleation::Value& value) {
    auto result = nucleation::capi::OutputCondition_equals(value.AsFFI());
    return std::unique_ptr<nucleation::OutputCondition>(nucleation::OutputCondition::FromFFI(result));
}

inline std::unique_ptr<nucleation::OutputCondition> nucleation::OutputCondition::not_equals(const nucleation::Value& value) {
    auto result = nucleation::capi::OutputCondition_not_equals(value.AsFFI());
    return std::unique_ptr<nucleation::OutputCondition>(nucleation::OutputCondition::FromFFI(result));
}

inline std::unique_ptr<nucleation::OutputCondition> nucleation::OutputCondition::greater_than(const nucleation::Value& value) {
    auto result = nucleation::capi::OutputCondition_greater_than(value.AsFFI());
    return std::unique_ptr<nucleation::OutputCondition>(nucleation::OutputCondition::FromFFI(result));
}

inline std::unique_ptr<nucleation::OutputCondition> nucleation::OutputCondition::less_than(const nucleation::Value& value) {
    auto result = nucleation::capi::OutputCondition_less_than(value.AsFFI());
    return std::unique_ptr<nucleation::OutputCondition>(nucleation::OutputCondition::FromFFI(result));
}

inline std::unique_ptr<nucleation::OutputCondition> nucleation::OutputCondition::bitwise_and(uint32_t mask) {
    auto result = nucleation::capi::OutputCondition_bitwise_and(mask);
    return std::unique_ptr<nucleation::OutputCondition>(nucleation::OutputCondition::FromFFI(result));
}

inline const nucleation::capi::OutputCondition* nucleation::OutputCondition::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::OutputCondition*>(this);
}

inline nucleation::capi::OutputCondition* nucleation::OutputCondition::AsFFI() {
    return reinterpret_cast<nucleation::capi::OutputCondition*>(this);
}

inline const nucleation::OutputCondition* nucleation::OutputCondition::FromFFI(const nucleation::capi::OutputCondition* ptr) {
    return reinterpret_cast<const nucleation::OutputCondition*>(ptr);
}

inline nucleation::OutputCondition* nucleation::OutputCondition::FromFFI(nucleation::capi::OutputCondition* ptr) {
    return reinterpret_cast<nucleation::OutputCondition*>(ptr);
}

inline void nucleation::OutputCondition::operator delete(void* ptr) {
    nucleation::capi::OutputCondition_destroy(reinterpret_cast<nucleation::capi::OutputCondition*>(ptr));
}


#endif // NUCLEATION_OutputCondition_HPP
