#ifndef AnimationEffect_HPP
#define AnimationEffect_HPP

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


namespace diplomat {
namespace capi {
    extern "C" {

    diplomat::capi::AnimationEffect* AnimationEffect_create(float duration_ms);

    diplomat::capi::AnimationEffect* AnimationEffect_instant(void);

    diplomat::capi::AnimationEffect* AnimationEffect_pop_in(float duration_ms);

    diplomat::capi::AnimationEffect* AnimationEffect_drop_in(float duration_ms, float height);

    diplomat::capi::AnimationEffect* AnimationEffect_drop_and_pop(float duration_ms, float height);

    diplomat::capi::AnimationEffect* AnimationEffect_spin_in(float duration_ms, float turns);

    diplomat::capi::AnimationEffect* AnimationEffect_turntable(float duration_ms);

    typedef struct AnimationEffect_add_tween_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} AnimationEffect_add_tween_result;
    AnimationEffect_add_tween_result AnimationEffect_add_tween(diplomat::capi::AnimationEffect* self, diplomat::capi::DiplomatStringView property_name, float from, float to, diplomat::capi::DiplomatStringView easing_name);

    typedef struct AnimationEffect_add_keyframe_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} AnimationEffect_add_keyframe_result;
    AnimationEffect_add_keyframe_result AnimationEffect_add_keyframe(diplomat::capi::AnimationEffect* self, diplomat::capi::DiplomatStringView property_name, float at, float value, diplomat::capi::DiplomatStringView easing_name);

    void AnimationEffect_set_repeat_forever(diplomat::capi::AnimationEffect* self);

    void AnimationEffect_destroy(AnimationEffect* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::unique_ptr<AnimationEffect> AnimationEffect::create(float duration_ms) {
    auto result = diplomat::capi::AnimationEffect_create(duration_ms);
    return std::unique_ptr<AnimationEffect>(AnimationEffect::FromFFI(result));
}

inline std::unique_ptr<AnimationEffect> AnimationEffect::instant() {
    auto result = diplomat::capi::AnimationEffect_instant();
    return std::unique_ptr<AnimationEffect>(AnimationEffect::FromFFI(result));
}

inline std::unique_ptr<AnimationEffect> AnimationEffect::pop_in(float duration_ms) {
    auto result = diplomat::capi::AnimationEffect_pop_in(duration_ms);
    return std::unique_ptr<AnimationEffect>(AnimationEffect::FromFFI(result));
}

inline std::unique_ptr<AnimationEffect> AnimationEffect::drop_in(float duration_ms, float height) {
    auto result = diplomat::capi::AnimationEffect_drop_in(duration_ms,
        height);
    return std::unique_ptr<AnimationEffect>(AnimationEffect::FromFFI(result));
}

inline std::unique_ptr<AnimationEffect> AnimationEffect::drop_and_pop(float duration_ms, float height) {
    auto result = diplomat::capi::AnimationEffect_drop_and_pop(duration_ms,
        height);
    return std::unique_ptr<AnimationEffect>(AnimationEffect::FromFFI(result));
}

inline std::unique_ptr<AnimationEffect> AnimationEffect::spin_in(float duration_ms, float turns) {
    auto result = diplomat::capi::AnimationEffect_spin_in(duration_ms,
        turns);
    return std::unique_ptr<AnimationEffect>(AnimationEffect::FromFFI(result));
}

inline std::unique_ptr<AnimationEffect> AnimationEffect::turntable(float duration_ms) {
    auto result = diplomat::capi::AnimationEffect_turntable(duration_ms);
    return std::unique_ptr<AnimationEffect>(AnimationEffect::FromFFI(result));
}

inline diplomat::result<std::monostate, NucleationError> AnimationEffect::add_tween(std::string_view property_name, float from, float to, std::string_view easing_name) {
    auto result = diplomat::capi::AnimationEffect_add_tween(this->AsFFI(),
        {property_name.data(), property_name.size()},
        from,
        to,
        {easing_name.data(), easing_name.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> AnimationEffect::add_keyframe(std::string_view property_name, float at, float value, std::string_view easing_name) {
    auto result = diplomat::capi::AnimationEffect_add_keyframe(this->AsFFI(),
        {property_name.data(), property_name.size()},
        at,
        value,
        {easing_name.data(), easing_name.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline void AnimationEffect::set_repeat_forever() {
    diplomat::capi::AnimationEffect_set_repeat_forever(this->AsFFI());
}

inline const diplomat::capi::AnimationEffect* AnimationEffect::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::AnimationEffect*>(this);
}

inline diplomat::capi::AnimationEffect* AnimationEffect::AsFFI() {
    return reinterpret_cast<diplomat::capi::AnimationEffect*>(this);
}

inline const AnimationEffect* AnimationEffect::FromFFI(const diplomat::capi::AnimationEffect* ptr) {
    return reinterpret_cast<const AnimationEffect*>(ptr);
}

inline AnimationEffect* AnimationEffect::FromFFI(diplomat::capi::AnimationEffect* ptr) {
    return reinterpret_cast<AnimationEffect*>(ptr);
}

inline void AnimationEffect::operator delete(void* ptr) {
    diplomat::capi::AnimationEffect_destroy(reinterpret_cast<diplomat::capi::AnimationEffect*>(ptr));
}


#endif // AnimationEffect_HPP
