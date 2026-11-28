#ifndef NUCLEATION_ScatterRule_HPP
#define NUCLEATION_ScatterRule_HPP

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


namespace nucleation {
namespace capi {
    extern "C" {

    typedef struct ScatterRule_create_result {union {nucleation::capi::ScatterRule* ok; nucleation::capi::NucleationError err;}; bool is_ok;} ScatterRule_create_result;
    ScatterRule_create_result ScatterRule_create(nucleation::capi::ScatterFace face, float density, nucleation::diplomat::capi::DiplomatStringView blocks_json, nucleation::diplomat::capi::DiplomatF32View weights, int32_t seed);

    void ScatterRule_set_condition(nucleation::capi::ScatterRule* self, const nucleation::capi::MaterialCondition* condition);

    typedef struct ScatterRule_set_on_block_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} ScatterRule_set_on_block_result;
    ScatterRule_set_on_block_result ScatterRule_set_on_block(nucleation::capi::ScatterRule* self, nucleation::diplomat::capi::DiplomatStringView prefix);

    typedef struct ScatterRule_set_height_range_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} ScatterRule_set_height_range_result;
    ScatterRule_set_height_range_result ScatterRule_set_height_range(nucleation::capi::ScatterRule* self, uint32_t min, uint32_t max);

    void ScatterRule_destroy(ScatterRule* self);

    } // extern "C"
} // namespace capi
} // namespace

inline nucleation::diplomat::result<std::unique_ptr<nucleation::ScatterRule>, nucleation::NucleationError> nucleation::ScatterRule::create(nucleation::ScatterFace face, float density, std::string_view blocks_json, nucleation::diplomat::span<const float> weights, int32_t seed) {
    auto result = nucleation::capi::ScatterRule_create(face.AsFFI(),
        density,
        {blocks_json.data(), blocks_json.size()},
        {weights.data(), weights.size()},
        seed);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::ScatterRule>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::ScatterRule>>(std::unique_ptr<nucleation::ScatterRule>(nucleation::ScatterRule::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::ScatterRule>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline void nucleation::ScatterRule::set_condition(const nucleation::MaterialCondition& condition) {
    nucleation::capi::ScatterRule_set_condition(this->AsFFI(),
        condition.AsFFI());
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::ScatterRule::set_on_block(std::string_view prefix) {
    auto result = nucleation::capi::ScatterRule_set_on_block(this->AsFFI(),
        {prefix.data(), prefix.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::ScatterRule::set_height_range(uint32_t min, uint32_t max) {
    auto result = nucleation::capi::ScatterRule_set_height_range(this->AsFFI(),
        min,
        max);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline const nucleation::capi::ScatterRule* nucleation::ScatterRule::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::ScatterRule*>(this);
}

inline nucleation::capi::ScatterRule* nucleation::ScatterRule::AsFFI() {
    return reinterpret_cast<nucleation::capi::ScatterRule*>(this);
}

inline const nucleation::ScatterRule* nucleation::ScatterRule::FromFFI(const nucleation::capi::ScatterRule* ptr) {
    return reinterpret_cast<const nucleation::ScatterRule*>(ptr);
}

inline nucleation::ScatterRule* nucleation::ScatterRule::FromFFI(nucleation::capi::ScatterRule* ptr) {
    return reinterpret_cast<nucleation::ScatterRule*>(ptr);
}

inline void nucleation::ScatterRule::operator delete(void* ptr) {
    nucleation::capi::ScatterRule_destroy(reinterpret_cast<nucleation::capi::ScatterRule*>(ptr));
}


#endif // NUCLEATION_ScatterRule_HPP
