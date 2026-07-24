#ifndef NUCLEATION_AnimationEffect_HPP
#define NUCLEATION_AnimationEffect_HPP

#include "AnimationEffect.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "NucleationError.hpp"
#include "diplomat_runtime.hpp"


namespace nucleation {
namespace capi {
    extern "C" {

    nucleation::capi::AnimationEffect* AnimationEffect_create(float duration_ms);

    nucleation::capi::AnimationEffect* AnimationEffect_instant(void);

    nucleation::capi::AnimationEffect* AnimationEffect_pop_in(float duration_ms);

    nucleation::capi::AnimationEffect* AnimationEffect_drop_in(float duration_ms, float height);

    nucleation::capi::AnimationEffect* AnimationEffect_drop_and_pop(float duration_ms, float height);

    nucleation::capi::AnimationEffect* AnimationEffect_spin_in(float duration_ms, float turns);

    nucleation::capi::AnimationEffect* AnimationEffect_turntable(float duration_ms);

    typedef struct AnimationEffect_add_tween_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} AnimationEffect_add_tween_result;
    AnimationEffect_add_tween_result AnimationEffect_add_tween(nucleation::capi::AnimationEffect* self, nucleation::diplomat::capi::DiplomatStringView property_name, float from, float to, nucleation::diplomat::capi::DiplomatStringView easing_name);

    typedef struct AnimationEffect_add_keyframe_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} AnimationEffect_add_keyframe_result;
    AnimationEffect_add_keyframe_result AnimationEffect_add_keyframe(nucleation::capi::AnimationEffect* self, nucleation::diplomat::capi::DiplomatStringView property_name, float at, float value, nucleation::diplomat::capi::DiplomatStringView easing_name);

    void AnimationEffect_set_repeat_forever(nucleation::capi::AnimationEffect* self);

    void AnimationEffect_destroy(AnimationEffect* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::unique_ptr<nucleation::AnimationEffect> nucleation::AnimationEffect::create(float duration_ms) {
    auto result = nucleation::capi::AnimationEffect_create(duration_ms);
    return std::unique_ptr<nucleation::AnimationEffect>(nucleation::AnimationEffect::FromFFI(result));
}

inline std::unique_ptr<nucleation::AnimationEffect> nucleation::AnimationEffect::instant() {
    auto result = nucleation::capi::AnimationEffect_instant();
    return std::unique_ptr<nucleation::AnimationEffect>(nucleation::AnimationEffect::FromFFI(result));
}

inline std::unique_ptr<nucleation::AnimationEffect> nucleation::AnimationEffect::pop_in(float duration_ms) {
    auto result = nucleation::capi::AnimationEffect_pop_in(duration_ms);
    return std::unique_ptr<nucleation::AnimationEffect>(nucleation::AnimationEffect::FromFFI(result));
}

inline std::unique_ptr<nucleation::AnimationEffect> nucleation::AnimationEffect::drop_in(float duration_ms, float height) {
    auto result = nucleation::capi::AnimationEffect_drop_in(duration_ms,
        height);
    return std::unique_ptr<nucleation::AnimationEffect>(nucleation::AnimationEffect::FromFFI(result));
}

inline std::unique_ptr<nucleation::AnimationEffect> nucleation::AnimationEffect::drop_and_pop(float duration_ms, float height) {
    auto result = nucleation::capi::AnimationEffect_drop_and_pop(duration_ms,
        height);
    return std::unique_ptr<nucleation::AnimationEffect>(nucleation::AnimationEffect::FromFFI(result));
}

inline std::unique_ptr<nucleation::AnimationEffect> nucleation::AnimationEffect::spin_in(float duration_ms, float turns) {
    auto result = nucleation::capi::AnimationEffect_spin_in(duration_ms,
        turns);
    return std::unique_ptr<nucleation::AnimationEffect>(nucleation::AnimationEffect::FromFFI(result));
}

inline std::unique_ptr<nucleation::AnimationEffect> nucleation::AnimationEffect::turntable(float duration_ms) {
    auto result = nucleation::capi::AnimationEffect_turntable(duration_ms);
    return std::unique_ptr<nucleation::AnimationEffect>(nucleation::AnimationEffect::FromFFI(result));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::AnimationEffect::add_tween(std::string_view property_name, float from, float to, std::string_view easing_name) {
    auto result = nucleation::capi::AnimationEffect_add_tween(this->AsFFI(),
        {property_name.data(), property_name.size()},
        from,
        to,
        {easing_name.data(), easing_name.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::AnimationEffect::add_keyframe(std::string_view property_name, float at, float value, std::string_view easing_name) {
    auto result = nucleation::capi::AnimationEffect_add_keyframe(this->AsFFI(),
        {property_name.data(), property_name.size()},
        at,
        value,
        {easing_name.data(), easing_name.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline void nucleation::AnimationEffect::set_repeat_forever() {
    nucleation::capi::AnimationEffect_set_repeat_forever(this->AsFFI());
}

inline const nucleation::capi::AnimationEffect* nucleation::AnimationEffect::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::AnimationEffect*>(this);
}

inline nucleation::capi::AnimationEffect* nucleation::AnimationEffect::AsFFI() {
    return reinterpret_cast<nucleation::capi::AnimationEffect*>(this);
}

inline const nucleation::AnimationEffect* nucleation::AnimationEffect::FromFFI(const nucleation::capi::AnimationEffect* ptr) {
    return reinterpret_cast<const nucleation::AnimationEffect*>(ptr);
}

inline nucleation::AnimationEffect* nucleation::AnimationEffect::FromFFI(nucleation::capi::AnimationEffect* ptr) {
    return reinterpret_cast<nucleation::AnimationEffect*>(ptr);
}

inline void nucleation::AnimationEffect::operator delete(void* ptr) {
    nucleation::capi::AnimationEffect_destroy(reinterpret_cast<nucleation::capi::AnimationEffect*>(ptr));
}


#endif // NUCLEATION_AnimationEffect_HPP
