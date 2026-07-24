#ifndef NUCLEATION_AnimationEffect_D_HPP
#define NUCLEATION_AnimationEffect_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"
namespace nucleation {
namespace capi { struct AnimationEffect; }
class AnimationEffect;
class NucleationError;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct AnimationEffect;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * A reusable set of property tracks, modelled after Anime.js object animations.
 */
class AnimationEffect {
public:

  inline static std::unique_ptr<nucleation::AnimationEffect> create(float duration_ms);

  inline static std::unique_ptr<nucleation::AnimationEffect> instant();

  inline static std::unique_ptr<nucleation::AnimationEffect> pop_in(float duration_ms);

  inline static std::unique_ptr<nucleation::AnimationEffect> drop_in(float duration_ms, float height);

  inline static std::unique_ptr<nucleation::AnimationEffect> drop_and_pop(float duration_ms, float height);

  inline static std::unique_ptr<nucleation::AnimationEffect> spin_in(float duration_ms, float turns);

  inline static std::unique_ptr<nucleation::AnimationEffect> turntable(float duration_ms);

  /**
   * Add a two-key property tween. Property names follow Anime.js/Three.js:
   * `x`, `y`, `z`, `rotateX`, `rotateY`, `rotateZ`, `scale`, `opacity`,
   * `tintR/G/B/A`, and `emissiveR/G/B`.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> add_tween(std::string_view property_name, float from, float to, std::string_view easing_name);

  /**
   * Add a normalised keyframe (`at` in `0..=1`) to a property track.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> add_keyframe(std::string_view property_name, float at, float value, std::string_view easing_name);

  inline void set_repeat_forever();

    inline const nucleation::capi::AnimationEffect* AsFFI() const;
    inline nucleation::capi::AnimationEffect* AsFFI();
    inline static const nucleation::AnimationEffect* FromFFI(const nucleation::capi::AnimationEffect* ptr);
    inline static nucleation::AnimationEffect* FromFFI(nucleation::capi::AnimationEffect* ptr);
    inline static void operator delete(void* ptr);
private:
    AnimationEffect() = delete;
    AnimationEffect(const nucleation::AnimationEffect&) = delete;
    AnimationEffect(nucleation::AnimationEffect&&) noexcept = delete;
    AnimationEffect operator=(const nucleation::AnimationEffect&) = delete;
    AnimationEffect operator=(nucleation::AnimationEffect&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_AnimationEffect_D_HPP
