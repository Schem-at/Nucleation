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
#include "ResourcePack.hpp"
#include "Schematic.hpp"
#include "Shape.hpp"
#include "VideoConfig.hpp"
#include "diplomat_runtime.hpp"


namespace diplomat {
namespace capi {
    extern "C" {

    diplomat::capi::BuildAnimation* BuildAnimation_create(diplomat::capi::DiplomatStringView name);

    typedef struct BuildAnimation_from_schematic_result {union {diplomat::capi::BuildAnimation* ok; diplomat::capi::NucleationError err;}; bool is_ok;} BuildAnimation_from_schematic_result;
    BuildAnimation_from_schematic_result BuildAnimation_from_schematic(const diplomat::capi::Schematic* schematic);

    void BuildAnimation_animate_all(diplomat::capi::BuildAnimation* self, const diplomat::capi::AnimationEffect* effect);

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

    typedef struct BuildAnimation_create_region_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} BuildAnimation_create_region_result;
    BuildAnimation_create_region_result BuildAnimation_create_region(diplomat::capi::BuildAnimation* self, diplomat::capi::DiplomatStringView name, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

    typedef struct BuildAnimation_set_block_in_region_result {union {uint32_t ok; diplomat::capi::NucleationError err;}; bool is_ok;} BuildAnimation_set_block_in_region_result;
    BuildAnimation_set_block_in_region_result BuildAnimation_set_block_in_region(diplomat::capi::BuildAnimation* self, diplomat::capi::DiplomatStringView region, int32_t x, int32_t y, int32_t z, diplomat::capi::DiplomatStringView block);

    typedef struct BuildAnimation_translate_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} BuildAnimation_translate_result;
    BuildAnimation_translate_result BuildAnimation_translate(diplomat::capi::BuildAnimation* self, int32_t x, int32_t y, int32_t z, float duration_ms);

    typedef struct BuildAnimation_translate_region_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} BuildAnimation_translate_region_result;
    BuildAnimation_translate_region_result BuildAnimation_translate_region(diplomat::capi::BuildAnimation* self, diplomat::capi::DiplomatStringView region, int32_t x, int32_t y, int32_t z, float duration_ms);

    typedef struct BuildAnimation_translate_all_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} BuildAnimation_translate_all_result;
    BuildAnimation_translate_all_result BuildAnimation_translate_all(diplomat::capi::BuildAnimation* self, int32_t x, int32_t y, int32_t z, float duration_ms);

    typedef struct BuildAnimation_rotate_x_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} BuildAnimation_rotate_x_result;
    BuildAnimation_rotate_x_result BuildAnimation_rotate_x(diplomat::capi::BuildAnimation* self, int32_t degrees, float duration_ms);

    typedef struct BuildAnimation_rotate_y_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} BuildAnimation_rotate_y_result;
    BuildAnimation_rotate_y_result BuildAnimation_rotate_y(diplomat::capi::BuildAnimation* self, int32_t degrees, float duration_ms);

    typedef struct BuildAnimation_rotate_z_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} BuildAnimation_rotate_z_result;
    BuildAnimation_rotate_z_result BuildAnimation_rotate_z(diplomat::capi::BuildAnimation* self, int32_t degrees, float duration_ms);

    typedef struct BuildAnimation_rotate_region_x_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} BuildAnimation_rotate_region_x_result;
    BuildAnimation_rotate_region_x_result BuildAnimation_rotate_region_x(diplomat::capi::BuildAnimation* self, diplomat::capi::DiplomatStringView region, int32_t degrees, float duration_ms);

    typedef struct BuildAnimation_rotate_region_y_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} BuildAnimation_rotate_region_y_result;
    BuildAnimation_rotate_region_y_result BuildAnimation_rotate_region_y(diplomat::capi::BuildAnimation* self, diplomat::capi::DiplomatStringView region, int32_t degrees, float duration_ms);

    typedef struct BuildAnimation_rotate_region_z_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} BuildAnimation_rotate_region_z_result;
    BuildAnimation_rotate_region_z_result BuildAnimation_rotate_region_z(diplomat::capi::BuildAnimation* self, diplomat::capi::DiplomatStringView region, int32_t degrees, float duration_ms);

    typedef struct BuildAnimation_rotate_all_x_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} BuildAnimation_rotate_all_x_result;
    BuildAnimation_rotate_all_x_result BuildAnimation_rotate_all_x(diplomat::capi::BuildAnimation* self, int32_t degrees, float duration_ms);

    typedef struct BuildAnimation_rotate_all_y_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} BuildAnimation_rotate_all_y_result;
    BuildAnimation_rotate_all_y_result BuildAnimation_rotate_all_y(diplomat::capi::BuildAnimation* self, int32_t degrees, float duration_ms);

    typedef struct BuildAnimation_rotate_all_z_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} BuildAnimation_rotate_all_z_result;
    BuildAnimation_rotate_all_z_result BuildAnimation_rotate_all_z(diplomat::capi::BuildAnimation* self, int32_t degrees, float duration_ms);

    typedef struct BuildAnimation_flip_x_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} BuildAnimation_flip_x_result;
    BuildAnimation_flip_x_result BuildAnimation_flip_x(diplomat::capi::BuildAnimation* self, float duration_ms);

    typedef struct BuildAnimation_flip_y_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} BuildAnimation_flip_y_result;
    BuildAnimation_flip_y_result BuildAnimation_flip_y(diplomat::capi::BuildAnimation* self, float duration_ms);

    typedef struct BuildAnimation_flip_z_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} BuildAnimation_flip_z_result;
    BuildAnimation_flip_z_result BuildAnimation_flip_z(diplomat::capi::BuildAnimation* self, float duration_ms);

    typedef struct BuildAnimation_flip_region_x_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} BuildAnimation_flip_region_x_result;
    BuildAnimation_flip_region_x_result BuildAnimation_flip_region_x(diplomat::capi::BuildAnimation* self, diplomat::capi::DiplomatStringView region, float duration_ms);

    typedef struct BuildAnimation_flip_region_y_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} BuildAnimation_flip_region_y_result;
    BuildAnimation_flip_region_y_result BuildAnimation_flip_region_y(diplomat::capi::BuildAnimation* self, diplomat::capi::DiplomatStringView region, float duration_ms);

    typedef struct BuildAnimation_flip_region_z_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} BuildAnimation_flip_region_z_result;
    BuildAnimation_flip_region_z_result BuildAnimation_flip_region_z(diplomat::capi::BuildAnimation* self, diplomat::capi::DiplomatStringView region, float duration_ms);

    typedef struct BuildAnimation_flip_all_x_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} BuildAnimation_flip_all_x_result;
    BuildAnimation_flip_all_x_result BuildAnimation_flip_all_x(diplomat::capi::BuildAnimation* self, float duration_ms);

    typedef struct BuildAnimation_flip_all_y_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} BuildAnimation_flip_all_y_result;
    BuildAnimation_flip_all_y_result BuildAnimation_flip_all_y(diplomat::capi::BuildAnimation* self, float duration_ms);

    typedef struct BuildAnimation_flip_all_z_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} BuildAnimation_flip_all_z_result;
    BuildAnimation_flip_all_z_result BuildAnimation_flip_all_z(diplomat::capi::BuildAnimation* self, float duration_ms);

    typedef struct BuildAnimation_stamp_region_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} BuildAnimation_stamp_region_result;
    BuildAnimation_stamp_region_result BuildAnimation_stamp_region(diplomat::capi::BuildAnimation* self, const diplomat::capi::Schematic* source, diplomat::capi::DiplomatStringView region, int32_t x, int32_t y, int32_t z, diplomat::capi::DiplomatStringView exclusions, float duration_ms);

    typedef struct BuildAnimation_stamp_box_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} BuildAnimation_stamp_box_result;
    BuildAnimation_stamp_box_result BuildAnimation_stamp_box(diplomat::capi::BuildAnimation* self, const diplomat::capi::Schematic* source, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z, int32_t x, int32_t y, int32_t z, diplomat::capi::DiplomatStringView exclusions, float duration_ms);

    void BuildAnimation_set_operation_gizmos(diplomat::capi::BuildAnimation* self, bool enabled);

    typedef struct BuildAnimation_operations_json_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} BuildAnimation_operations_json_result;
    BuildAnimation_operations_json_result BuildAnimation_operations_json(const diplomat::capi::BuildAnimation* self, diplomat::capi::DiplomatWrite* write);

    typedef struct BuildAnimation_frame_json_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} BuildAnimation_frame_json_result;
    BuildAnimation_frame_json_result BuildAnimation_frame_json(const diplomat::capi::BuildAnimation* self, float time_ms, diplomat::capi::DiplomatWrite* write);

    typedef struct BuildAnimation_fill_along_parameter_result {union {uint32_t ok; diplomat::capi::NucleationError err;}; bool is_ok;} BuildAnimation_fill_along_parameter_result;
    BuildAnimation_fill_along_parameter_result BuildAnimation_fill_along_parameter(diplomat::capi::BuildAnimation* self, const diplomat::capi::Shape* shape, const diplomat::capi::Brush* brush, uint32_t group_count);

    typedef struct BuildAnimation_add_armor_stand_result {union {uint32_t ok; diplomat::capi::NucleationError err;}; bool is_ok;} BuildAnimation_add_armor_stand_result;
    BuildAnimation_add_armor_stand_result BuildAnimation_add_armor_stand(diplomat::capi::BuildAnimation* self, double x, double y, double z, float yaw, diplomat::capi::DiplomatStringView armor_material);

    void BuildAnimation_animate_camera(diplomat::capi::BuildAnimation* self, const diplomat::capi::AnimationEffect* effect, float offset_ms);

    uint32_t BuildAnimation_frame_count(const diplomat::capi::BuildAnimation* self, double fps, float hold_ms);

    typedef struct BuildAnimation_render_gif_result {union {uint32_t ok; diplomat::capi::NucleationError err;}; bool is_ok;} BuildAnimation_render_gif_result;
    BuildAnimation_render_gif_result BuildAnimation_render_gif(const diplomat::capi::BuildAnimation* self, diplomat::capi::DiplomatU8View pack_zip, const diplomat::capi::RenderConfig* config, diplomat::capi::DiplomatStringView path, double fps, float hold_ms);

    typedef struct BuildAnimation_render_video_with_pack_result {union {uint32_t ok; diplomat::capi::NucleationError err;}; bool is_ok;} BuildAnimation_render_video_with_pack_result;
    BuildAnimation_render_video_with_pack_result BuildAnimation_render_video_with_pack(const diplomat::capi::BuildAnimation* self, const diplomat::capi::ResourcePack* pack, const diplomat::capi::RenderConfig* config, const diplomat::capi::VideoConfig* video, diplomat::capi::DiplomatStringView path, float hold_ms);

    typedef struct BuildAnimation_render_video_result {union {uint32_t ok; diplomat::capi::NucleationError err;}; bool is_ok;} BuildAnimation_render_video_result;
    BuildAnimation_render_video_result BuildAnimation_render_video(const diplomat::capi::BuildAnimation* self, diplomat::capi::DiplomatU8View pack_zip, const diplomat::capi::RenderConfig* config, const diplomat::capi::VideoConfig* video, diplomat::capi::DiplomatStringView path, float hold_ms);

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

inline diplomat::result<std::unique_ptr<BuildAnimation>, NucleationError> BuildAnimation::from_schematic(const Schematic& schematic) {
    auto result = diplomat::capi::BuildAnimation_from_schematic(schematic.AsFFI());
    return result.is_ok ? diplomat::result<std::unique_ptr<BuildAnimation>, NucleationError>(diplomat::Ok<std::unique_ptr<BuildAnimation>>(std::unique_ptr<BuildAnimation>(BuildAnimation::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<BuildAnimation>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline void BuildAnimation::animate_all(const AnimationEffect& effect) {
    diplomat::capi::BuildAnimation_animate_all(this->AsFFI(),
        effect.AsFFI());
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

inline diplomat::result<std::monostate, NucleationError> BuildAnimation::create_region(std::string_view name, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z) {
    auto result = diplomat::capi::BuildAnimation_create_region(this->AsFFI(),
        {name.data(), name.size()},
        min_x,
        min_y,
        min_z,
        max_x,
        max_y,
        max_z);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<uint32_t, NucleationError> BuildAnimation::set_block_in_region(std::string_view region, int32_t x, int32_t y, int32_t z, std::string_view block) {
    auto result = diplomat::capi::BuildAnimation_set_block_in_region(this->AsFFI(),
        {region.data(), region.size()},
        x,
        y,
        z,
        {block.data(), block.size()});
    return result.is_ok ? diplomat::result<uint32_t, NucleationError>(diplomat::Ok<uint32_t>(result.ok)) : diplomat::result<uint32_t, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> BuildAnimation::translate(int32_t x, int32_t y, int32_t z, float duration_ms) {
    auto result = diplomat::capi::BuildAnimation_translate(this->AsFFI(),
        x,
        y,
        z,
        duration_ms);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> BuildAnimation::translate_region(std::string_view region, int32_t x, int32_t y, int32_t z, float duration_ms) {
    auto result = diplomat::capi::BuildAnimation_translate_region(this->AsFFI(),
        {region.data(), region.size()},
        x,
        y,
        z,
        duration_ms);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> BuildAnimation::translate_all(int32_t x, int32_t y, int32_t z, float duration_ms) {
    auto result = diplomat::capi::BuildAnimation_translate_all(this->AsFFI(),
        x,
        y,
        z,
        duration_ms);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> BuildAnimation::rotate_x(int32_t degrees, float duration_ms) {
    auto result = diplomat::capi::BuildAnimation_rotate_x(this->AsFFI(),
        degrees,
        duration_ms);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> BuildAnimation::rotate_y(int32_t degrees, float duration_ms) {
    auto result = diplomat::capi::BuildAnimation_rotate_y(this->AsFFI(),
        degrees,
        duration_ms);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> BuildAnimation::rotate_z(int32_t degrees, float duration_ms) {
    auto result = diplomat::capi::BuildAnimation_rotate_z(this->AsFFI(),
        degrees,
        duration_ms);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> BuildAnimation::rotate_region_x(std::string_view region, int32_t degrees, float duration_ms) {
    auto result = diplomat::capi::BuildAnimation_rotate_region_x(this->AsFFI(),
        {region.data(), region.size()},
        degrees,
        duration_ms);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> BuildAnimation::rotate_region_y(std::string_view region, int32_t degrees, float duration_ms) {
    auto result = diplomat::capi::BuildAnimation_rotate_region_y(this->AsFFI(),
        {region.data(), region.size()},
        degrees,
        duration_ms);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> BuildAnimation::rotate_region_z(std::string_view region, int32_t degrees, float duration_ms) {
    auto result = diplomat::capi::BuildAnimation_rotate_region_z(this->AsFFI(),
        {region.data(), region.size()},
        degrees,
        duration_ms);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> BuildAnimation::rotate_all_x(int32_t degrees, float duration_ms) {
    auto result = diplomat::capi::BuildAnimation_rotate_all_x(this->AsFFI(),
        degrees,
        duration_ms);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> BuildAnimation::rotate_all_y(int32_t degrees, float duration_ms) {
    auto result = diplomat::capi::BuildAnimation_rotate_all_y(this->AsFFI(),
        degrees,
        duration_ms);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> BuildAnimation::rotate_all_z(int32_t degrees, float duration_ms) {
    auto result = diplomat::capi::BuildAnimation_rotate_all_z(this->AsFFI(),
        degrees,
        duration_ms);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> BuildAnimation::flip_x(float duration_ms) {
    auto result = diplomat::capi::BuildAnimation_flip_x(this->AsFFI(),
        duration_ms);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> BuildAnimation::flip_y(float duration_ms) {
    auto result = diplomat::capi::BuildAnimation_flip_y(this->AsFFI(),
        duration_ms);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> BuildAnimation::flip_z(float duration_ms) {
    auto result = diplomat::capi::BuildAnimation_flip_z(this->AsFFI(),
        duration_ms);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> BuildAnimation::flip_region_x(std::string_view region, float duration_ms) {
    auto result = diplomat::capi::BuildAnimation_flip_region_x(this->AsFFI(),
        {region.data(), region.size()},
        duration_ms);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> BuildAnimation::flip_region_y(std::string_view region, float duration_ms) {
    auto result = diplomat::capi::BuildAnimation_flip_region_y(this->AsFFI(),
        {region.data(), region.size()},
        duration_ms);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> BuildAnimation::flip_region_z(std::string_view region, float duration_ms) {
    auto result = diplomat::capi::BuildAnimation_flip_region_z(this->AsFFI(),
        {region.data(), region.size()},
        duration_ms);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> BuildAnimation::flip_all_x(float duration_ms) {
    auto result = diplomat::capi::BuildAnimation_flip_all_x(this->AsFFI(),
        duration_ms);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> BuildAnimation::flip_all_y(float duration_ms) {
    auto result = diplomat::capi::BuildAnimation_flip_all_y(this->AsFFI(),
        duration_ms);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> BuildAnimation::flip_all_z(float duration_ms) {
    auto result = diplomat::capi::BuildAnimation_flip_all_z(this->AsFFI(),
        duration_ms);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> BuildAnimation::stamp_region(const Schematic& source, std::string_view region, int32_t x, int32_t y, int32_t z, std::string_view exclusions, float duration_ms) {
    auto result = diplomat::capi::BuildAnimation_stamp_region(this->AsFFI(),
        source.AsFFI(),
        {region.data(), region.size()},
        x,
        y,
        z,
        {exclusions.data(), exclusions.size()},
        duration_ms);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> BuildAnimation::stamp_box(const Schematic& source, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z, int32_t x, int32_t y, int32_t z, std::string_view exclusions, float duration_ms) {
    auto result = diplomat::capi::BuildAnimation_stamp_box(this->AsFFI(),
        source.AsFFI(),
        min_x,
        min_y,
        min_z,
        max_x,
        max_y,
        max_z,
        x,
        y,
        z,
        {exclusions.data(), exclusions.size()},
        duration_ms);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline void BuildAnimation::set_operation_gizmos(bool enabled) {
    diplomat::capi::BuildAnimation_set_operation_gizmos(this->AsFFI(),
        enabled);
}

inline diplomat::result<std::string, NucleationError> BuildAnimation::operations_json() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::BuildAnimation_operations_json(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> BuildAnimation::operations_json_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::BuildAnimation_operations_json(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> BuildAnimation::frame_json(float time_ms) const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::BuildAnimation_frame_json(this->AsFFI(),
        time_ms,
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> BuildAnimation::frame_json_write(float time_ms, W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::BuildAnimation_frame_json(this->AsFFI(),
        time_ms,
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
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

inline diplomat::result<uint32_t, NucleationError> BuildAnimation::render_video_with_pack(const ResourcePack& pack, const RenderConfig& config, const VideoConfig& video, std::string_view path, float hold_ms) const {
    auto result = diplomat::capi::BuildAnimation_render_video_with_pack(this->AsFFI(),
        pack.AsFFI(),
        config.AsFFI(),
        video.AsFFI(),
        {path.data(), path.size()},
        hold_ms);
    return result.is_ok ? diplomat::result<uint32_t, NucleationError>(diplomat::Ok<uint32_t>(result.ok)) : diplomat::result<uint32_t, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<uint32_t, NucleationError> BuildAnimation::render_video(diplomat::span<const uint8_t> pack_zip, const RenderConfig& config, const VideoConfig& video, std::string_view path, float hold_ms) const {
    auto result = diplomat::capi::BuildAnimation_render_video(this->AsFFI(),
        {pack_zip.data(), pack_zip.size()},
        config.AsFFI(),
        video.AsFFI(),
        {path.data(), path.size()},
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
