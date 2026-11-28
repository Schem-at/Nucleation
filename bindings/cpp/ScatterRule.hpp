#ifndef ScatterRule_HPP
#define ScatterRule_HPP

#include "ScatterRule.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "MaterialCondition.hpp"
#include "NucleationError.hpp"
#include "ScatterFace.hpp"
#include "diplomat_runtime.hpp"


namespace diplomat {
namespace capi {
    extern "C" {

    typedef struct ScatterRule_create_result {union {diplomat::capi::ScatterRule* ok; diplomat::capi::NucleationError err;}; bool is_ok;} ScatterRule_create_result;
    ScatterRule_create_result ScatterRule_create(diplomat::capi::ScatterFace face, float density, diplomat::capi::DiplomatStringView blocks_json, diplomat::capi::DiplomatF32View weights, int32_t seed);

    void ScatterRule_set_condition(diplomat::capi::ScatterRule* self, const diplomat::capi::MaterialCondition* condition);

    typedef struct ScatterRule_set_on_block_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} ScatterRule_set_on_block_result;
    ScatterRule_set_on_block_result ScatterRule_set_on_block(diplomat::capi::ScatterRule* self, diplomat::capi::DiplomatStringView prefix);

    typedef struct ScatterRule_set_height_range_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} ScatterRule_set_height_range_result;
    ScatterRule_set_height_range_result ScatterRule_set_height_range(diplomat::capi::ScatterRule* self, uint32_t min, uint32_t max);

    void ScatterRule_destroy(ScatterRule* self);

    } // extern "C"
} // namespace capi
} // namespace

inline diplomat::result<std::unique_ptr<ScatterRule>, NucleationError> ScatterRule::create(ScatterFace face, float density, std::string_view blocks_json, diplomat::span<const float> weights, int32_t seed) {
    auto result = diplomat::capi::ScatterRule_create(face.AsFFI(),
        density,
        {blocks_json.data(), blocks_json.size()},
        {weights.data(), weights.size()},
        seed);
    return result.is_ok ? diplomat::result<std::unique_ptr<ScatterRule>, NucleationError>(diplomat::Ok<std::unique_ptr<ScatterRule>>(std::unique_ptr<ScatterRule>(ScatterRule::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<ScatterRule>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline void ScatterRule::set_condition(const MaterialCondition& condition) {
    diplomat::capi::ScatterRule_set_condition(this->AsFFI(),
        condition.AsFFI());
}

inline diplomat::result<std::monostate, NucleationError> ScatterRule::set_on_block(std::string_view prefix) {
    auto result = diplomat::capi::ScatterRule_set_on_block(this->AsFFI(),
        {prefix.data(), prefix.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> ScatterRule::set_height_range(uint32_t min, uint32_t max) {
    auto result = diplomat::capi::ScatterRule_set_height_range(this->AsFFI(),
        min,
        max);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline const diplomat::capi::ScatterRule* ScatterRule::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::ScatterRule*>(this);
}

inline diplomat::capi::ScatterRule* ScatterRule::AsFFI() {
    return reinterpret_cast<diplomat::capi::ScatterRule*>(this);
}

inline const ScatterRule* ScatterRule::FromFFI(const diplomat::capi::ScatterRule* ptr) {
    return reinterpret_cast<const ScatterRule*>(ptr);
}

inline ScatterRule* ScatterRule::FromFFI(diplomat::capi::ScatterRule* ptr) {
    return reinterpret_cast<ScatterRule*>(ptr);
}

inline void ScatterRule::operator delete(void* ptr) {
    diplomat::capi::ScatterRule_destroy(reinterpret_cast<diplomat::capi::ScatterRule*>(ptr));
}


#endif // ScatterRule_HPP
