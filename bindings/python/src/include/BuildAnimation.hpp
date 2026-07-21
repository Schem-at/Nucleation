#ifndef NUCLEATION_BuildAnimation_HPP
#define NUCLEATION_BuildAnimation_HPP

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
#include "NucleationError.hpp"
#include "RenderConfig.hpp"
#include "diplomat_runtime.hpp"


namespace nucleation {
namespace capi {
    extern "C" {

    nucleation::capi::BuildAnimation* BuildAnimation_create(nucleation::diplomat::capi::DiplomatStringView name);

    void BuildAnimation_set_default_effect(nucleation::capi::BuildAnimation* self, const nucleation::capi::AnimationEffect* effect);

    nucleation::capi::BuildAnimation* BuildAnimation_with_effect(nucleation::capi::BuildAnimation* self, const nucleation::capi::AnimationEffect* effect);

    void BuildAnimation_set_step_ms(nucleation::capi::BuildAnimation* self, float step_ms);

    void BuildAnimation_set_stagger_total_ms(nucleation::capi::BuildAnimation* self, float total_ms);

    void BuildAnimation_clear_stagger(nucleation::capi::BuildAnimation* self);

    typedef struct BuildAnimation_begin_group_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} BuildAnimation_begin_group_result;
    BuildAnimation_begin_group_result BuildAnimation_begin_group(nucleation::capi::BuildAnimation* self);

    typedef struct BuildAnimation_begin_keyed_group_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} BuildAnimation_begin_keyed_group_result;
    BuildAnimation_begin_keyed_group_result BuildAnimation_begin_keyed_group(nucleation::capi::BuildAnimation* self, float key);

    typedef struct BuildAnimation_end_group_result {union {uint32_t ok; nucleation::capi::NucleationError err;}; bool is_ok;} BuildAnimation_end_group_result;
    BuildAnimation_end_group_result BuildAnimation_end_group(nucleation::capi::BuildAnimation* self);

    typedef struct BuildAnimation_set_block_result {union {uint32_t ok; nucleation::capi::NucleationError err;}; bool is_ok;} BuildAnimation_set_block_result;
    BuildAnimation_set_block_result BuildAnimation_set_block(nucleation::capi::BuildAnimation* self, int32_t x, int32_t y, int32_t z, nucleation::diplomat::capi::DiplomatStringView block);

    typedef struct BuildAnimation_add_armor_stand_result {union {uint32_t ok; nucleation::capi::NucleationError err;}; bool is_ok;} BuildAnimation_add_armor_stand_result;
    BuildAnimation_add_armor_stand_result BuildAnimation_add_armor_stand(nucleation::capi::BuildAnimation* self, double x, double y, double z, float yaw, nucleation::diplomat::capi::DiplomatStringView armor_material);

    void BuildAnimation_animate_camera(nucleation::capi::BuildAnimation* self, const nucleation::capi::AnimationEffect* effect, float offset_ms);

    uint32_t BuildAnimation_frame_count(const nucleation::capi::BuildAnimation* self, double fps, float hold_ms);

    typedef struct BuildAnimation_render_gif_result {union {uint32_t ok; nucleation::capi::NucleationError err;}; bool is_ok;} BuildAnimation_render_gif_result;
    BuildAnimation_render_gif_result BuildAnimation_render_gif(const nucleation::capi::BuildAnimation* self, nucleation::diplomat::capi::DiplomatU8View pack_zip, const nucleation::capi::RenderConfig* config, nucleation::diplomat::capi::DiplomatStringView path, double fps, float hold_ms);

    typedef struct BuildAnimation_render_frames_result {union {uint32_t ok; nucleation::capi::NucleationError err;}; bool is_ok;} BuildAnimation_render_frames_result;
    BuildAnimation_render_frames_result BuildAnimation_render_frames(const nucleation::capi::BuildAnimation* self, nucleation::diplomat::capi::DiplomatU8View pack_zip, const nucleation::capi::RenderConfig* config, nucleation::diplomat::capi::DiplomatStringView prefix, double fps, float hold_ms);

    typedef struct BuildAnimation_save_to_file_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} BuildAnimation_save_to_file_result;
    BuildAnimation_save_to_file_result BuildAnimation_save_to_file(const nucleation::capi::BuildAnimation* self, nucleation::diplomat::capi::DiplomatStringView path);

    uint32_t BuildAnimation_group_count(const nucleation::capi::BuildAnimation* self);

    float BuildAnimation_duration_ms(const nucleation::capi::BuildAnimation* self);

    void BuildAnimation_destroy(BuildAnimation* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::unique_ptr<nucleation::BuildAnimation> nucleation::BuildAnimation::create(std::string_view name) {
    auto result = nucleation::capi::BuildAnimation_create({name.data(), name.size()});
    return std::unique_ptr<nucleation::BuildAnimation>(nucleation::BuildAnimation::FromFFI(result));
}

inline void nucleation::BuildAnimation::set_default_effect(const nucleation::AnimationEffect& effect) {
    nucleation::capi::BuildAnimation_set_default_effect(this->AsFFI(),
        effect.AsFFI());
}

inline nucleation::BuildAnimation& nucleation::BuildAnimation::with_effect(const nucleation::AnimationEffect& effect) {
    auto result = nucleation::capi::BuildAnimation_with_effect(this->AsFFI(),
        effect.AsFFI());
    return *nucleation::BuildAnimation::FromFFI(result);
}

inline void nucleation::BuildAnimation::set_step_ms(float step_ms) {
    nucleation::capi::BuildAnimation_set_step_ms(this->AsFFI(),
        step_ms);
}

inline void nucleation::BuildAnimation::set_stagger_total_ms(float total_ms) {
    nucleation::capi::BuildAnimation_set_stagger_total_ms(this->AsFFI(),
        total_ms);
}

inline void nucleation::BuildAnimation::clear_stagger() {
    nucleation::capi::BuildAnimation_clear_stagger(this->AsFFI());
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::BuildAnimation::begin_group() {
    auto result = nucleation::capi::BuildAnimation_begin_group(this->AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::BuildAnimation::begin_keyed_group(float key) {
    auto result = nucleation::capi::BuildAnimation_begin_keyed_group(this->AsFFI(),
        key);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<uint32_t, nucleation::NucleationError> nucleation::BuildAnimation::end_group() {
    auto result = nucleation::capi::BuildAnimation_end_group(this->AsFFI());
    return result.is_ok ? nucleation::diplomat::result<uint32_t, nucleation::NucleationError>(nucleation::diplomat::Ok<uint32_t>(result.ok)) : nucleation::diplomat::result<uint32_t, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<uint32_t, nucleation::NucleationError> nucleation::BuildAnimation::set_block(int32_t x, int32_t y, int32_t z, std::string_view block) {
    auto result = nucleation::capi::BuildAnimation_set_block(this->AsFFI(),
        x,
        y,
        z,
        {block.data(), block.size()});
    return result.is_ok ? nucleation::diplomat::result<uint32_t, nucleation::NucleationError>(nucleation::diplomat::Ok<uint32_t>(result.ok)) : nucleation::diplomat::result<uint32_t, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<uint32_t, nucleation::NucleationError> nucleation::BuildAnimation::add_armor_stand(double x, double y, double z, float yaw, std::string_view armor_material) {
    auto result = nucleation::capi::BuildAnimation_add_armor_stand(this->AsFFI(),
        x,
        y,
        z,
        yaw,
        {armor_material.data(), armor_material.size()});
    return result.is_ok ? nucleation::diplomat::result<uint32_t, nucleation::NucleationError>(nucleation::diplomat::Ok<uint32_t>(result.ok)) : nucleation::diplomat::result<uint32_t, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline void nucleation::BuildAnimation::animate_camera(const nucleation::AnimationEffect& effect, float offset_ms) {
    nucleation::capi::BuildAnimation_animate_camera(this->AsFFI(),
        effect.AsFFI(),
        offset_ms);
}

inline uint32_t nucleation::BuildAnimation::frame_count(double fps, float hold_ms) const {
    auto result = nucleation::capi::BuildAnimation_frame_count(this->AsFFI(),
        fps,
        hold_ms);
    return result;
}

inline nucleation::diplomat::result<uint32_t, nucleation::NucleationError> nucleation::BuildAnimation::render_gif(nucleation::diplomat::span<const uint8_t> pack_zip, const nucleation::RenderConfig& config, std::string_view path, double fps, float hold_ms) const {
    auto result = nucleation::capi::BuildAnimation_render_gif(this->AsFFI(),
        {pack_zip.data(), pack_zip.size()},
        config.AsFFI(),
        {path.data(), path.size()},
        fps,
        hold_ms);
    return result.is_ok ? nucleation::diplomat::result<uint32_t, nucleation::NucleationError>(nucleation::diplomat::Ok<uint32_t>(result.ok)) : nucleation::diplomat::result<uint32_t, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<uint32_t, nucleation::NucleationError> nucleation::BuildAnimation::render_frames(nucleation::diplomat::span<const uint8_t> pack_zip, const nucleation::RenderConfig& config, std::string_view prefix, double fps, float hold_ms) const {
    auto result = nucleation::capi::BuildAnimation_render_frames(this->AsFFI(),
        {pack_zip.data(), pack_zip.size()},
        config.AsFFI(),
        {prefix.data(), prefix.size()},
        fps,
        hold_ms);
    return result.is_ok ? nucleation::diplomat::result<uint32_t, nucleation::NucleationError>(nucleation::diplomat::Ok<uint32_t>(result.ok)) : nucleation::diplomat::result<uint32_t, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::BuildAnimation::save_to_file(std::string_view path) const {
    auto result = nucleation::capi::BuildAnimation_save_to_file(this->AsFFI(),
        {path.data(), path.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline uint32_t nucleation::BuildAnimation::group_count() const {
    auto result = nucleation::capi::BuildAnimation_group_count(this->AsFFI());
    return result;
}

inline float nucleation::BuildAnimation::duration_ms() const {
    auto result = nucleation::capi::BuildAnimation_duration_ms(this->AsFFI());
    return result;
}

inline const nucleation::capi::BuildAnimation* nucleation::BuildAnimation::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::BuildAnimation*>(this);
}

inline nucleation::capi::BuildAnimation* nucleation::BuildAnimation::AsFFI() {
    return reinterpret_cast<nucleation::capi::BuildAnimation*>(this);
}

inline const nucleation::BuildAnimation* nucleation::BuildAnimation::FromFFI(const nucleation::capi::BuildAnimation* ptr) {
    return reinterpret_cast<const nucleation::BuildAnimation*>(ptr);
}

inline nucleation::BuildAnimation* nucleation::BuildAnimation::FromFFI(nucleation::capi::BuildAnimation* ptr) {
    return reinterpret_cast<nucleation::BuildAnimation*>(ptr);
}

inline void nucleation::BuildAnimation::operator delete(void* ptr) {
    nucleation::capi::BuildAnimation_destroy(reinterpret_cast<nucleation::capi::BuildAnimation*>(ptr));
}


#endif // NUCLEATION_BuildAnimation_HPP
