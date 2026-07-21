#ifndef BuildAnimation_HPP
#define BuildAnimation_HPP

#include "BuildAnimation.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "AnimationEffect.hpp"
#include "Brush.hpp"
#include "NucleationError.hpp"
#include "RenderConfig.hpp"
#include "Shape.hpp"
#include "diplomat_runtime.hpp"


namespace diplomat {
namespace capi {
    extern "C" {

    diplomat::capi::BuildAnimation* BuildAnimation_create(diplomat::capi::DiplomatStringView name);

    void BuildAnimation_set_default_effect(diplomat::capi::BuildAnimation* self, const diplomat::capi::AnimationEffect* effect);

    diplomat::capi::BuildAnimation* BuildAnimation_with_effect(diplomat::capi::BuildAnimation* self, const diplomat::capi::AnimationEffect* effect);

    void BuildAnimation_set_step_ms(diplomat::capi::BuildAnimation* self, float step_ms);

    void BuildAnimation_set_stagger_total_ms(diplomat::capi::BuildAnimation* self, float total_ms);

    void BuildAnimation_clear_stagger(diplomat::capi::BuildAnimation* self);

    void BuildAnimation_set_stagger_offset_ms(diplomat::capi::BuildAnimation* self, float offset_ms);

    typedef struct BuildAnimation_set_loop_period_ms_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} BuildAnimation_set_loop_period_ms_result;
    BuildAnimation_set_loop_period_ms_result BuildAnimation_set_loop_period_ms(diplomat::capi::BuildAnimation* self, float period_ms);

    void BuildAnimation_clear_loop_period(diplomat::capi::BuildAnimation* self);

    typedef struct BuildAnimation_begin_group_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} BuildAnimation_begin_group_result;
    BuildAnimation_begin_group_result BuildAnimation_begin_group(diplomat::capi::BuildAnimation* self);

    typedef struct BuildAnimation_begin_keyed_group_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} BuildAnimation_begin_keyed_group_result;
    BuildAnimation_begin_keyed_group_result BuildAnimation_begin_keyed_group(diplomat::capi::BuildAnimation* self, float key);

    typedef struct BuildAnimation_end_group_result {union {uint32_t ok; diplomat::capi::NucleationError err;}; bool is_ok;} BuildAnimation_end_group_result;
    BuildAnimation_end_group_result BuildAnimation_end_group(diplomat::capi::BuildAnimation* self);

    typedef struct BuildAnimation_set_block_result {union {uint32_t ok; diplomat::capi::NucleationError err;}; bool is_ok;} BuildAnimation_set_block_result;
    BuildAnimation_set_block_result BuildAnimation_set_block(diplomat::capi::BuildAnimation* self, int32_t x, int32_t y, int32_t z, diplomat::capi::DiplomatStringView block);

    typedef struct BuildAnimation_fill_along_parameter_result {union {uint32_t ok; diplomat::capi::NucleationError err;}; bool is_ok;} BuildAnimation_fill_along_parameter_result;
    BuildAnimation_fill_along_parameter_result BuildAnimation_fill_along_parameter(diplomat::capi::BuildAnimation* self, const diplomat::capi::Shape* shape, const diplomat::capi::Brush* brush, uint32_t group_count);

    typedef struct BuildAnimation_add_armor_stand_result {union {uint32_t ok; diplomat::capi::NucleationError err;}; bool is_ok;} BuildAnimation_add_armor_stand_result;
    BuildAnimation_add_armor_stand_result BuildAnimation_add_armor_stand(diplomat::capi::BuildAnimation* self, double x, double y, double z, float yaw, diplomat::capi::DiplomatStringView armor_material);

    void BuildAnimation_animate_camera(diplomat::capi::BuildAnimation* self, const diplomat::capi::AnimationEffect* effect, float offset_ms);

    uint32_t BuildAnimation_frame_count(const diplomat::capi::BuildAnimation* self, double fps, float hold_ms);

    typedef struct BuildAnimation_render_gif_result {union {uint32_t ok; diplomat::capi::NucleationError err;}; bool is_ok;} BuildAnimation_render_gif_result;
    BuildAnimation_render_gif_result BuildAnimation_render_gif(const diplomat::capi::BuildAnimation* self, diplomat::capi::DiplomatU8View pack_zip, const diplomat::capi::RenderConfig* config, diplomat::capi::DiplomatStringView path, double fps, float hold_ms);

    typedef struct BuildAnimation_render_frames_result {union {uint32_t ok; diplomat::capi::NucleationError err;}; bool is_ok;} BuildAnimation_render_frames_result;
    BuildAnimation_render_frames_result BuildAnimation_render_frames(const diplomat::capi::BuildAnimation* self, diplomat::capi::DiplomatU8View pack_zip, const diplomat::capi::RenderConfig* config, diplomat::capi::DiplomatStringView prefix, double fps, float hold_ms);

    typedef struct BuildAnimation_save_to_file_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} BuildAnimation_save_to_file_result;
    BuildAnimation_save_to_file_result BuildAnimation_save_to_file(const diplomat::capi::BuildAnimation* self, diplomat::capi::DiplomatStringView path);

    uint32_t BuildAnimation_group_count(const diplomat::capi::BuildAnimation* self);

    float BuildAnimation_duration_ms(const diplomat::capi::BuildAnimation* self);

    void BuildAnimation_destroy(BuildAnimation* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::unique_ptr<BuildAnimation> BuildAnimation::create(std::string_view name) {
    auto result = diplomat::capi::BuildAnimation_create({name.data(), name.size()});
    return std::unique_ptr<BuildAnimation>(BuildAnimation::FromFFI(result));
}

inline void BuildAnimation::set_default_effect(const AnimationEffect& effect) {
    diplomat::capi::BuildAnimation_set_default_effect(this->AsFFI(),
        effect.AsFFI());
}

inline BuildAnimation& BuildAnimation::with_effect(const AnimationEffect& effect) {
    auto result = diplomat::capi::BuildAnimation_with_effect(this->AsFFI(),
        effect.AsFFI());
    return *BuildAnimation::FromFFI(result);
}

inline void BuildAnimation::set_step_ms(float step_ms) {
    diplomat::capi::BuildAnimation_set_step_ms(this->AsFFI(),
        step_ms);
}

inline void BuildAnimation::set_stagger_total_ms(float total_ms) {
    diplomat::capi::BuildAnimation_set_stagger_total_ms(this->AsFFI(),
        total_ms);
}

inline void BuildAnimation::clear_stagger() {
    diplomat::capi::BuildAnimation_clear_stagger(this->AsFFI());
}

inline void BuildAnimation::set_stagger_offset_ms(float offset_ms) {
    diplomat::capi::BuildAnimation_set_stagger_offset_ms(this->AsFFI(),
        offset_ms);
}

inline diplomat::result<std::monostate, NucleationError> BuildAnimation::set_loop_period_ms(float period_ms) {
    auto result = diplomat::capi::BuildAnimation_set_loop_period_ms(this->AsFFI(),
        period_ms);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline void BuildAnimation::clear_loop_period() {
    diplomat::capi::BuildAnimation_clear_loop_period(this->AsFFI());
}

inline diplomat::result<std::monostate, NucleationError> BuildAnimation::begin_group() {
    auto result = diplomat::capi::BuildAnimation_begin_group(this->AsFFI());
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> BuildAnimation::begin_keyed_group(float key) {
    auto result = diplomat::capi::BuildAnimation_begin_keyed_group(this->AsFFI(),
        key);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<uint32_t, NucleationError> BuildAnimation::end_group() {
    auto result = diplomat::capi::BuildAnimation_end_group(this->AsFFI());
    return result.is_ok ? diplomat::result<uint32_t, NucleationError>(diplomat::Ok<uint32_t>(result.ok)) : diplomat::result<uint32_t, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<uint32_t, NucleationError> BuildAnimation::set_block(int32_t x, int32_t y, int32_t z, std::string_view block) {
    auto result = diplomat::capi::BuildAnimation_set_block(this->AsFFI(),
        x,
        y,
        z,
        {block.data(), block.size()});
    return result.is_ok ? diplomat::result<uint32_t, NucleationError>(diplomat::Ok<uint32_t>(result.ok)) : diplomat::result<uint32_t, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<uint32_t, NucleationError> BuildAnimation::fill_along_parameter(const Shape& shape, const Brush& brush, uint32_t group_count) {
    auto result = diplomat::capi::BuildAnimation_fill_along_parameter(this->AsFFI(),
        shape.AsFFI(),
        brush.AsFFI(),
        group_count);
    return result.is_ok ? diplomat::result<uint32_t, NucleationError>(diplomat::Ok<uint32_t>(result.ok)) : diplomat::result<uint32_t, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<uint32_t, NucleationError> BuildAnimation::add_armor_stand(double x, double y, double z, float yaw, std::string_view armor_material) {
    auto result = diplomat::capi::BuildAnimation_add_armor_stand(this->AsFFI(),
        x,
        y,
        z,
        yaw,
        {armor_material.data(), armor_material.size()});
    return result.is_ok ? diplomat::result<uint32_t, NucleationError>(diplomat::Ok<uint32_t>(result.ok)) : diplomat::result<uint32_t, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline void BuildAnimation::animate_camera(const AnimationEffect& effect, float offset_ms) {
    diplomat::capi::BuildAnimation_animate_camera(this->AsFFI(),
        effect.AsFFI(),
        offset_ms);
}

inline uint32_t BuildAnimation::frame_count(double fps, float hold_ms) const {
    auto result = diplomat::capi::BuildAnimation_frame_count(this->AsFFI(),
        fps,
        hold_ms);
    return result;
}

inline diplomat::result<uint32_t, NucleationError> BuildAnimation::render_gif(diplomat::span<const uint8_t> pack_zip, const RenderConfig& config, std::string_view path, double fps, float hold_ms) const {
    auto result = diplomat::capi::BuildAnimation_render_gif(this->AsFFI(),
        {pack_zip.data(), pack_zip.size()},
        config.AsFFI(),
        {path.data(), path.size()},
        fps,
        hold_ms);
    return result.is_ok ? diplomat::result<uint32_t, NucleationError>(diplomat::Ok<uint32_t>(result.ok)) : diplomat::result<uint32_t, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<uint32_t, NucleationError> BuildAnimation::render_frames(diplomat::span<const uint8_t> pack_zip, const RenderConfig& config, std::string_view prefix, double fps, float hold_ms) const {
    auto result = diplomat::capi::BuildAnimation_render_frames(this->AsFFI(),
        {pack_zip.data(), pack_zip.size()},
        config.AsFFI(),
        {prefix.data(), prefix.size()},
        fps,
        hold_ms);
    return result.is_ok ? diplomat::result<uint32_t, NucleationError>(diplomat::Ok<uint32_t>(result.ok)) : diplomat::result<uint32_t, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> BuildAnimation::save_to_file(std::string_view path) const {
    auto result = diplomat::capi::BuildAnimation_save_to_file(this->AsFFI(),
        {path.data(), path.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline uint32_t BuildAnimation::group_count() const {
    auto result = diplomat::capi::BuildAnimation_group_count(this->AsFFI());
    return result;
}

inline float BuildAnimation::duration_ms() const {
    auto result = diplomat::capi::BuildAnimation_duration_ms(this->AsFFI());
    return result;
}

inline const diplomat::capi::BuildAnimation* BuildAnimation::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::BuildAnimation*>(this);
}

inline diplomat::capi::BuildAnimation* BuildAnimation::AsFFI() {
    return reinterpret_cast<diplomat::capi::BuildAnimation*>(this);
}

inline const BuildAnimation* BuildAnimation::FromFFI(const diplomat::capi::BuildAnimation* ptr) {
    return reinterpret_cast<const BuildAnimation*>(ptr);
}

inline BuildAnimation* BuildAnimation::FromFFI(diplomat::capi::BuildAnimation* ptr) {
    return reinterpret_cast<BuildAnimation*>(ptr);
}

inline void BuildAnimation::operator delete(void* ptr) {
    diplomat::capi::BuildAnimation_destroy(reinterpret_cast<diplomat::capi::BuildAnimation*>(ptr));
}


#endif // BuildAnimation_HPP
