#ifndef AnimationEffect_H
#define AnimationEffect_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "NucleationError.d.h"

#include "AnimationEffect.d.h"






AnimationEffect* AnimationEffect_create(float duration_ms);

AnimationEffect* AnimationEffect_instant(void);

AnimationEffect* AnimationEffect_pop_in(float duration_ms);

AnimationEffect* AnimationEffect_drop_in(float duration_ms, float height);

AnimationEffect* AnimationEffect_drop_and_pop(float duration_ms, float height);

AnimationEffect* AnimationEffect_spin_in(float duration_ms, float turns);

AnimationEffect* AnimationEffect_turntable(float duration_ms);

typedef struct AnimationEffect_add_tween_result {union { NucleationError err;}; bool is_ok;} AnimationEffect_add_tween_result;
AnimationEffect_add_tween_result AnimationEffect_add_tween(AnimationEffect* self, DiplomatStringView property_name, float from, float to, DiplomatStringView easing_name);

typedef struct AnimationEffect_add_keyframe_result {union { NucleationError err;}; bool is_ok;} AnimationEffect_add_keyframe_result;
AnimationEffect_add_keyframe_result AnimationEffect_add_keyframe(AnimationEffect* self, DiplomatStringView property_name, float at, float value, DiplomatStringView easing_name);

void AnimationEffect_set_repeat_forever(AnimationEffect* self);

void AnimationEffect_destroy(AnimationEffect* self);





#endif // AnimationEffect_H
