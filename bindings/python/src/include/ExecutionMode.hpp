#ifndef NUCLEATION_ExecutionMode_HPP
#define NUCLEATION_ExecutionMode_HPP

#include "ExecutionMode.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "NucleationError.hpp"
#include "OutputCondition.hpp"
#include "diplomat_runtime.hpp"


namespace nucleation {
namespace capi {
    extern "C" {

    nucleation::capi::ExecutionMode* ExecutionMode_fixed_ticks(uint32_t ticks);

    typedef struct ExecutionMode_until_condition_result {union {nucleation::capi::ExecutionMode* ok; nucleation::capi::NucleationError err;}; bool is_ok;} ExecutionMode_until_condition_result;
    ExecutionMode_until_condition_result ExecutionMode_until_condition(nucleation::diplomat::capi::DiplomatStringView output_name, const nucleation::capi::OutputCondition* condition, uint32_t max_ticks, uint32_t check_interval);

    nucleation::capi::ExecutionMode* ExecutionMode_until_change(uint32_t max_ticks, uint32_t check_interval);

    nucleation::capi::ExecutionMode* ExecutionMode_until_stable(uint32_t stable_ticks, uint32_t max_ticks);

    void ExecutionMode_destroy(ExecutionMode* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::unique_ptr<nucleation::ExecutionMode> nucleation::ExecutionMode::fixed_ticks(uint32_t ticks) {
    auto result = nucleation::capi::ExecutionMode_fixed_ticks(ticks);
    return std::unique_ptr<nucleation::ExecutionMode>(nucleation::ExecutionMode::FromFFI(result));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::ExecutionMode>, nucleation::NucleationError> nucleation::ExecutionMode::until_condition(std::string_view output_name, const nucleation::OutputCondition& condition, uint32_t max_ticks, uint32_t check_interval) {
    auto result = nucleation::capi::ExecutionMode_until_condition({output_name.data(), output_name.size()},
        condition.AsFFI(),
        max_ticks,
        check_interval);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::ExecutionMode>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::ExecutionMode>>(std::unique_ptr<nucleation::ExecutionMode>(nucleation::ExecutionMode::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::ExecutionMode>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline std::unique_ptr<nucleation::ExecutionMode> nucleation::ExecutionMode::until_change(uint32_t max_ticks, uint32_t check_interval) {
    auto result = nucleation::capi::ExecutionMode_until_change(max_ticks,
        check_interval);
    return std::unique_ptr<nucleation::ExecutionMode>(nucleation::ExecutionMode::FromFFI(result));
}

inline std::unique_ptr<nucleation::ExecutionMode> nucleation::ExecutionMode::until_stable(uint32_t stable_ticks, uint32_t max_ticks) {
    auto result = nucleation::capi::ExecutionMode_until_stable(stable_ticks,
        max_ticks);
    return std::unique_ptr<nucleation::ExecutionMode>(nucleation::ExecutionMode::FromFFI(result));
}

inline const nucleation::capi::ExecutionMode* nucleation::ExecutionMode::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::ExecutionMode*>(this);
}

inline nucleation::capi::ExecutionMode* nucleation::ExecutionMode::AsFFI() {
    return reinterpret_cast<nucleation::capi::ExecutionMode*>(this);
}

inline const nucleation::ExecutionMode* nucleation::ExecutionMode::FromFFI(const nucleation::capi::ExecutionMode* ptr) {
    return reinterpret_cast<const nucleation::ExecutionMode*>(ptr);
}

inline nucleation::ExecutionMode* nucleation::ExecutionMode::FromFFI(nucleation::capi::ExecutionMode* ptr) {
    return reinterpret_cast<nucleation::ExecutionMode*>(ptr);
}

inline void nucleation::ExecutionMode::operator delete(void* ptr) {
    nucleation::capi::ExecutionMode_destroy(reinterpret_cast<nucleation::capi::ExecutionMode*>(ptr));
}


#endif // NUCLEATION_ExecutionMode_HPP
