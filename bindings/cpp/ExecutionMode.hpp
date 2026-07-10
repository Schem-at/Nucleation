#ifndef ExecutionMode_HPP
#define ExecutionMode_HPP

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


namespace diplomat {
namespace capi {
    extern "C" {

    diplomat::capi::ExecutionMode* ExecutionMode_fixed_ticks(uint32_t ticks);

    typedef struct ExecutionMode_until_condition_result {union {diplomat::capi::ExecutionMode* ok; diplomat::capi::NucleationError err;}; bool is_ok;} ExecutionMode_until_condition_result;
    ExecutionMode_until_condition_result ExecutionMode_until_condition(diplomat::capi::DiplomatStringView output_name, const diplomat::capi::OutputCondition* condition, uint32_t max_ticks, uint32_t check_interval);

    diplomat::capi::ExecutionMode* ExecutionMode_until_change(uint32_t max_ticks, uint32_t check_interval);

    diplomat::capi::ExecutionMode* ExecutionMode_until_stable(uint32_t stable_ticks, uint32_t max_ticks);

    void ExecutionMode_destroy(ExecutionMode* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::unique_ptr<ExecutionMode> ExecutionMode::fixed_ticks(uint32_t ticks) {
    auto result = diplomat::capi::ExecutionMode_fixed_ticks(ticks);
    return std::unique_ptr<ExecutionMode>(ExecutionMode::FromFFI(result));
}

inline diplomat::result<std::unique_ptr<ExecutionMode>, NucleationError> ExecutionMode::until_condition(std::string_view output_name, const OutputCondition& condition, uint32_t max_ticks, uint32_t check_interval) {
    auto result = diplomat::capi::ExecutionMode_until_condition({output_name.data(), output_name.size()},
        condition.AsFFI(),
        max_ticks,
        check_interval);
    return result.is_ok ? diplomat::result<std::unique_ptr<ExecutionMode>, NucleationError>(diplomat::Ok<std::unique_ptr<ExecutionMode>>(std::unique_ptr<ExecutionMode>(ExecutionMode::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<ExecutionMode>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline std::unique_ptr<ExecutionMode> ExecutionMode::until_change(uint32_t max_ticks, uint32_t check_interval) {
    auto result = diplomat::capi::ExecutionMode_until_change(max_ticks,
        check_interval);
    return std::unique_ptr<ExecutionMode>(ExecutionMode::FromFFI(result));
}

inline std::unique_ptr<ExecutionMode> ExecutionMode::until_stable(uint32_t stable_ticks, uint32_t max_ticks) {
    auto result = diplomat::capi::ExecutionMode_until_stable(stable_ticks,
        max_ticks);
    return std::unique_ptr<ExecutionMode>(ExecutionMode::FromFFI(result));
}

inline const diplomat::capi::ExecutionMode* ExecutionMode::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::ExecutionMode*>(this);
}

inline diplomat::capi::ExecutionMode* ExecutionMode::AsFFI() {
    return reinterpret_cast<diplomat::capi::ExecutionMode*>(this);
}

inline const ExecutionMode* ExecutionMode::FromFFI(const diplomat::capi::ExecutionMode* ptr) {
    return reinterpret_cast<const ExecutionMode*>(ptr);
}

inline ExecutionMode* ExecutionMode::FromFFI(diplomat::capi::ExecutionMode* ptr) {
    return reinterpret_cast<ExecutionMode*>(ptr);
}

inline void ExecutionMode::operator delete(void* ptr) {
    diplomat::capi::ExecutionMode_destroy(reinterpret_cast<diplomat::capi::ExecutionMode*>(ptr));
}


#endif // ExecutionMode_HPP
