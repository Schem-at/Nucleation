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
#include "Brush.hpp"
#include "NucleationError.hpp"
#include "RenderConfig.hpp"
#include "ResourcePack.hpp"
#include "Schematic.hpp"
#include "Shape.hpp"
#include "VideoConfig.hpp"
#include "diplomat_runtime.hpp"


namespace nucleation {
namespace capi {
    extern "C" {

    nucleation::capi::BuildAnimation* BuildAnimation_create(nucleation::diplomat::capi::DiplomatStringView name);

    typedef struct BuildAnimation_from_schematic_result {union {nucleation::capi::BuildAnimation* ok; nucleation::capi::NucleationError err;}; bool is_ok;} BuildAnimation_from_schematic_result;
    BuildAnimation_from_schematic_result BuildAnimation_from_schematic(const nucleation::capi::Schematic* schematic);

    void BuildAnimation_animate_all(nucleation::capi::BuildAnimation* self, const nucleation::capi::AnimationEffect* effect);

    void BuildAnimation_set_default_effect(nucleation::capi::BuildAnimation* self, const nucleation::capi::AnimationEffect* effect);

    nucleation::capi::BuildAnimation* BuildAnimation_with_effect(nucleation::capi::BuildAnimation* self, const nucleation::capi::AnimationEffect* effect);

    void BuildAnimation_set_step_ms(nucleation::capi::BuildAnimation* self, float step_ms);

    void BuildAnimation_set_stagger_total_ms(nucleation::capi::BuildAnimation* self, float total_ms);

    void BuildAnimation_clear_stagger(nucleation::capi::BuildAnimation* self);

    void BuildAnimation_set_stagger_offset_ms(nucleation::capi::BuildAnimation* self, float offset_ms);

    typedef struct BuildAnimation_set_loop_period_ms_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} BuildAnimation_set_loop_period_ms_result;
    BuildAnimation_set_loop_period_ms_result BuildAnimation_set_loop_period_ms(nucleation::capi::BuildAnimation* self, float period_ms);

    void BuildAnimation_clear_loop_period(nucleation::capi::BuildAnimation* self);

    typedef struct BuildAnimation_begin_group_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} BuildAnimation_begin_group_result;
    BuildAnimation_begin_group_result BuildAnimation_begin_group(nucleation::capi::BuildAnimation* self);

    typedef struct BuildAnimation_begin_keyed_group_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} BuildAnimation_begin_keyed_group_result;
    BuildAnimation_begin_keyed_group_result BuildAnimation_begin_keyed_group(nucleation::capi::BuildAnimation* self, float key);

    typedef struct BuildAnimation_end_group_result {union {uint32_t ok; nucleation::capi::NucleationError err;}; bool is_ok;} BuildAnimation_end_group_result;
    BuildAnimation_end_group_result BuildAnimation_end_group(nucleation::capi::BuildAnimation* self);

    typedef struct BuildAnimation_set_block_result {union {uint32_t ok; nucleation::capi::NucleationError err;}; bool is_ok;} BuildAnimation_set_block_result;
    BuildAnimation_set_block_result BuildAnimation_set_block(nucleation::capi::BuildAnimation* self, int32_t x, int32_t y, int32_t z, nucleation::diplomat::capi::DiplomatStringView block);

    typedef struct BuildAnimation_create_region_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} BuildAnimation_create_region_result;
    BuildAnimation_create_region_result BuildAnimation_create_region(nucleation::capi::BuildAnimation* self, nucleation::diplomat::capi::DiplomatStringView name, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

    typedef struct BuildAnimation_set_block_in_region_result {union {uint32_t ok; nucleation::capi::NucleationError err;}; bool is_ok;} BuildAnimation_set_block_in_region_result;
    BuildAnimation_set_block_in_region_result BuildAnimation_set_block_in_region(nucleation::capi::BuildAnimation* self, nucleation::diplomat::capi::DiplomatStringView region, int32_t x, int32_t y, int32_t z, nucleation::diplomat::capi::DiplomatStringView block);

    typedef struct BuildAnimation_translate_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} BuildAnimation_translate_result;
    BuildAnimation_translate_result BuildAnimation_translate(nucleation::capi::BuildAnimation* self, int32_t x, int32_t y, int32_t z, float duration_ms);

    typedef struct BuildAnimation_translate_region_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} BuildAnimation_translate_region_result;
    BuildAnimation_translate_region_result BuildAnimation_translate_region(nucleation::capi::BuildAnimation* self, nucleation::diplomat::capi::DiplomatStringView region, int32_t x, int32_t y, int32_t z, float duration_ms);

    typedef struct BuildAnimation_translate_all_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} BuildAnimation_translate_all_result;
    BuildAnimation_translate_all_result BuildAnimation_translate_all(nucleation::capi::BuildAnimation* self, int32_t x, int32_t y, int32_t z, float duration_ms);

    typedef struct BuildAnimation_rotate_x_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} BuildAnimation_rotate_x_result;
    BuildAnimation_rotate_x_result BuildAnimation_rotate_x(nucleation::capi::BuildAnimation* self, int32_t degrees, float duration_ms);

    typedef struct BuildAnimation_rotate_y_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} BuildAnimation_rotate_y_result;
    BuildAnimation_rotate_y_result BuildAnimation_rotate_y(nucleation::capi::BuildAnimation* self, int32_t degrees, float duration_ms);

    typedef struct BuildAnimation_rotate_z_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} BuildAnimation_rotate_z_result;
    BuildAnimation_rotate_z_result BuildAnimation_rotate_z(nucleation::capi::BuildAnimation* self, int32_t degrees, float duration_ms);

    typedef struct BuildAnimation_rotate_region_x_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} BuildAnimation_rotate_region_x_result;
    BuildAnimation_rotate_region_x_result BuildAnimation_rotate_region_x(nucleation::capi::BuildAnimation* self, nucleation::diplomat::capi::DiplomatStringView region, int32_t degrees, float duration_ms);

    typedef struct BuildAnimation_rotate_region_y_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} BuildAnimation_rotate_region_y_result;
    BuildAnimation_rotate_region_y_result BuildAnimation_rotate_region_y(nucleation::capi::BuildAnimation* self, nucleation::diplomat::capi::DiplomatStringView region, int32_t degrees, float duration_ms);

    typedef struct BuildAnimation_rotate_region_z_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} BuildAnimation_rotate_region_z_result;
    BuildAnimation_rotate_region_z_result BuildAnimation_rotate_region_z(nucleation::capi::BuildAnimation* self, nucleation::diplomat::capi::DiplomatStringView region, int32_t degrees, float duration_ms);

    typedef struct BuildAnimation_rotate_all_x_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} BuildAnimation_rotate_all_x_result;
    BuildAnimation_rotate_all_x_result BuildAnimation_rotate_all_x(nucleation::capi::BuildAnimation* self, int32_t degrees, float duration_ms);

    typedef struct BuildAnimation_rotate_all_y_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} BuildAnimation_rotate_all_y_result;
    BuildAnimation_rotate_all_y_result BuildAnimation_rotate_all_y(nucleation::capi::BuildAnimation* self, int32_t degrees, float duration_ms);

    typedef struct BuildAnimation_rotate_all_z_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} BuildAnimation_rotate_all_z_result;
    BuildAnimation_rotate_all_z_result BuildAnimation_rotate_all_z(nucleation::capi::BuildAnimation* self, int32_t degrees, float duration_ms);

    typedef struct BuildAnimation_flip_x_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} BuildAnimation_flip_x_result;
    BuildAnimation_flip_x_result BuildAnimation_flip_x(nucleation::capi::BuildAnimation* self, float duration_ms);

    typedef struct BuildAnimation_flip_y_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} BuildAnimation_flip_y_result;
    BuildAnimation_flip_y_result BuildAnimation_flip_y(nucleation::capi::BuildAnimation* self, float duration_ms);

    typedef struct BuildAnimation_flip_z_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} BuildAnimation_flip_z_result;
    BuildAnimation_flip_z_result BuildAnimation_flip_z(nucleation::capi::BuildAnimation* self, float duration_ms);

    typedef struct BuildAnimation_flip_region_x_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} BuildAnimation_flip_region_x_result;
    BuildAnimation_flip_region_x_result BuildAnimation_flip_region_x(nucleation::capi::BuildAnimation* self, nucleation::diplomat::capi::DiplomatStringView region, float duration_ms);

    typedef struct BuildAnimation_flip_region_y_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} BuildAnimation_flip_region_y_result;
    BuildAnimation_flip_region_y_result BuildAnimation_flip_region_y(nucleation::capi::BuildAnimation* self, nucleation::diplomat::capi::DiplomatStringView region, float duration_ms);

    typedef struct BuildAnimation_flip_region_z_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} BuildAnimation_flip_region_z_result;
    BuildAnimation_flip_region_z_result BuildAnimation_flip_region_z(nucleation::capi::BuildAnimation* self, nucleation::diplomat::capi::DiplomatStringView region, float duration_ms);

    typedef struct BuildAnimation_flip_all_x_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} BuildAnimation_flip_all_x_result;
    BuildAnimation_flip_all_x_result BuildAnimation_flip_all_x(nucleation::capi::BuildAnimation* self, float duration_ms);

    typedef struct BuildAnimation_flip_all_y_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} BuildAnimation_flip_all_y_result;
    BuildAnimation_flip_all_y_result BuildAnimation_flip_all_y(nucleation::capi::BuildAnimation* self, float duration_ms);

    typedef struct BuildAnimation_flip_all_z_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} BuildAnimation_flip_all_z_result;
    BuildAnimation_flip_all_z_result BuildAnimation_flip_all_z(nucleation::capi::BuildAnimation* self, float duration_ms);

    typedef struct BuildAnimation_stamp_region_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} BuildAnimation_stamp_region_result;
    BuildAnimation_stamp_region_result BuildAnimation_stamp_region(nucleation::capi::BuildAnimation* self, const nucleation::capi::Schematic* source, nucleation::diplomat::capi::DiplomatStringView region, int32_t x, int32_t y, int32_t z, nucleation::diplomat::capi::DiplomatStringView exclusions, float duration_ms);

    typedef struct BuildAnimation_stamp_box_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} BuildAnimation_stamp_box_result;
    BuildAnimation_stamp_box_result BuildAnimation_stamp_box(nucleation::capi::BuildAnimation* self, const nucleation::capi::Schematic* source, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z, int32_t x, int32_t y, int32_t z, nucleation::diplomat::capi::DiplomatStringView exclusions, float duration_ms);

    void BuildAnimation_set_operation_gizmos(nucleation::capi::BuildAnimation* self, bool enabled);

    typedef struct BuildAnimation_operations_json_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} BuildAnimation_operations_json_result;
    BuildAnimation_operations_json_result BuildAnimation_operations_json(const nucleation::capi::BuildAnimation* self, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct BuildAnimation_frame_json_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} BuildAnimation_frame_json_result;
    BuildAnimation_frame_json_result BuildAnimation_frame_json(const nucleation::capi::BuildAnimation* self, float time_ms, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct BuildAnimation_fill_along_parameter_result {union {uint32_t ok; nucleation::capi::NucleationError err;}; bool is_ok;} BuildAnimation_fill_along_parameter_result;
    BuildAnimation_fill_along_parameter_result BuildAnimation_fill_along_parameter(nucleation::capi::BuildAnimation* self, const nucleation::capi::Shape* shape, const nucleation::capi::Brush* brush, uint32_t group_count);

    typedef struct BuildAnimation_add_armor_stand_result {union {uint32_t ok; nucleation::capi::NucleationError err;}; bool is_ok;} BuildAnimation_add_armor_stand_result;
    BuildAnimation_add_armor_stand_result BuildAnimation_add_armor_stand(nucleation::capi::BuildAnimation* self, double x, double y, double z, float yaw, nucleation::diplomat::capi::DiplomatStringView armor_material);

    void BuildAnimation_animate_camera(nucleation::capi::BuildAnimation* self, const nucleation::capi::AnimationEffect* effect, float offset_ms);

    uint32_t BuildAnimation_frame_count(const nucleation::capi::BuildAnimation* self, double fps, float hold_ms);

    typedef struct BuildAnimation_render_gif_result {union {uint32_t ok; nucleation::capi::NucleationError err;}; bool is_ok;} BuildAnimation_render_gif_result;
    BuildAnimation_render_gif_result BuildAnimation_render_gif(const nucleation::capi::BuildAnimation* self, nucleation::diplomat::capi::DiplomatU8View pack_zip, const nucleation::capi::RenderConfig* config, nucleation::diplomat::capi::DiplomatStringView path, double fps, float hold_ms);

    typedef struct BuildAnimation_render_video_with_pack_result {union {uint32_t ok; nucleation::capi::NucleationError err;}; bool is_ok;} BuildAnimation_render_video_with_pack_result;
    BuildAnimation_render_video_with_pack_result BuildAnimation_render_video_with_pack(const nucleation::capi::BuildAnimation* self, const nucleation::capi::ResourcePack* pack, const nucleation::capi::RenderConfig* config, const nucleation::capi::VideoConfig* video, nucleation::diplomat::capi::DiplomatStringView path, float hold_ms);

    typedef struct BuildAnimation_render_video_result {union {uint32_t ok; nucleation::capi::NucleationError err;}; bool is_ok;} BuildAnimation_render_video_result;
    BuildAnimation_render_video_result BuildAnimation_render_video(const nucleation::capi::BuildAnimation* self, nucleation::diplomat::capi::DiplomatU8View pack_zip, const nucleation::capi::RenderConfig* config, const nucleation::capi::VideoConfig* video, nucleation::diplomat::capi::DiplomatStringView path, float hold_ms);

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

inline nucleation::diplomat::result<std::unique_ptr<nucleation::BuildAnimation>, nucleation::NucleationError> nucleation::BuildAnimation::from_schematic(const nucleation::Schematic& schematic) {
    auto result = nucleation::capi::BuildAnimation_from_schematic(schematic.AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::BuildAnimation>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::BuildAnimation>>(std::unique_ptr<nucleation::BuildAnimation>(nucleation::BuildAnimation::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::BuildAnimation>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline void nucleation::BuildAnimation::animate_all(const nucleation::AnimationEffect& effect) {
    nucleation::capi::BuildAnimation_animate_all(this->AsFFI(),
        effect.AsFFI());
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

inline void nucleation::BuildAnimation::set_stagger_offset_ms(float offset_ms) {
    nucleation::capi::BuildAnimation_set_stagger_offset_ms(this->AsFFI(),
        offset_ms);
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::BuildAnimation::set_loop_period_ms(float period_ms) {
    auto result = nucleation::capi::BuildAnimation_set_loop_period_ms(this->AsFFI(),
        period_ms);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline void nucleation::BuildAnimation::clear_loop_period() {
    nucleation::capi::BuildAnimation_clear_loop_period(this->AsFFI());
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

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::BuildAnimation::create_region(std::string_view name, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z) {
    auto result = nucleation::capi::BuildAnimation_create_region(this->AsFFI(),
        {name.data(), name.size()},
        min_x,
        min_y,
        min_z,
        max_x,
        max_y,
        max_z);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<uint32_t, nucleation::NucleationError> nucleation::BuildAnimation::set_block_in_region(std::string_view region, int32_t x, int32_t y, int32_t z, std::string_view block) {
    auto result = nucleation::capi::BuildAnimation_set_block_in_region(this->AsFFI(),
        {region.data(), region.size()},
        x,
        y,
        z,
        {block.data(), block.size()});
    return result.is_ok ? nucleation::diplomat::result<uint32_t, nucleation::NucleationError>(nucleation::diplomat::Ok<uint32_t>(result.ok)) : nucleation::diplomat::result<uint32_t, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::BuildAnimation::translate(int32_t x, int32_t y, int32_t z, float duration_ms) {
    auto result = nucleation::capi::BuildAnimation_translate(this->AsFFI(),
        x,
        y,
        z,
        duration_ms);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::BuildAnimation::translate_region(std::string_view region, int32_t x, int32_t y, int32_t z, float duration_ms) {
    auto result = nucleation::capi::BuildAnimation_translate_region(this->AsFFI(),
        {region.data(), region.size()},
        x,
        y,
        z,
        duration_ms);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::BuildAnimation::translate_all(int32_t x, int32_t y, int32_t z, float duration_ms) {
    auto result = nucleation::capi::BuildAnimation_translate_all(this->AsFFI(),
        x,
        y,
        z,
        duration_ms);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::BuildAnimation::rotate_x(int32_t degrees, float duration_ms) {
    auto result = nucleation::capi::BuildAnimation_rotate_x(this->AsFFI(),
        degrees,
        duration_ms);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::BuildAnimation::rotate_y(int32_t degrees, float duration_ms) {
    auto result = nucleation::capi::BuildAnimation_rotate_y(this->AsFFI(),
        degrees,
        duration_ms);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::BuildAnimation::rotate_z(int32_t degrees, float duration_ms) {
    auto result = nucleation::capi::BuildAnimation_rotate_z(this->AsFFI(),
        degrees,
        duration_ms);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::BuildAnimation::rotate_region_x(std::string_view region, int32_t degrees, float duration_ms) {
    auto result = nucleation::capi::BuildAnimation_rotate_region_x(this->AsFFI(),
        {region.data(), region.size()},
        degrees,
        duration_ms);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::BuildAnimation::rotate_region_y(std::string_view region, int32_t degrees, float duration_ms) {
    auto result = nucleation::capi::BuildAnimation_rotate_region_y(this->AsFFI(),
        {region.data(), region.size()},
        degrees,
        duration_ms);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::BuildAnimation::rotate_region_z(std::string_view region, int32_t degrees, float duration_ms) {
    auto result = nucleation::capi::BuildAnimation_rotate_region_z(this->AsFFI(),
        {region.data(), region.size()},
        degrees,
        duration_ms);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::BuildAnimation::rotate_all_x(int32_t degrees, float duration_ms) {
    auto result = nucleation::capi::BuildAnimation_rotate_all_x(this->AsFFI(),
        degrees,
        duration_ms);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::BuildAnimation::rotate_all_y(int32_t degrees, float duration_ms) {
    auto result = nucleation::capi::BuildAnimation_rotate_all_y(this->AsFFI(),
        degrees,
        duration_ms);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::BuildAnimation::rotate_all_z(int32_t degrees, float duration_ms) {
    auto result = nucleation::capi::BuildAnimation_rotate_all_z(this->AsFFI(),
        degrees,
        duration_ms);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::BuildAnimation::flip_x(float duration_ms) {
    auto result = nucleation::capi::BuildAnimation_flip_x(this->AsFFI(),
        duration_ms);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::BuildAnimation::flip_y(float duration_ms) {
    auto result = nucleation::capi::BuildAnimation_flip_y(this->AsFFI(),
        duration_ms);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::BuildAnimation::flip_z(float duration_ms) {
    auto result = nucleation::capi::BuildAnimation_flip_z(this->AsFFI(),
        duration_ms);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::BuildAnimation::flip_region_x(std::string_view region, float duration_ms) {
    auto result = nucleation::capi::BuildAnimation_flip_region_x(this->AsFFI(),
        {region.data(), region.size()},
        duration_ms);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::BuildAnimation::flip_region_y(std::string_view region, float duration_ms) {
    auto result = nucleation::capi::BuildAnimation_flip_region_y(this->AsFFI(),
        {region.data(), region.size()},
        duration_ms);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::BuildAnimation::flip_region_z(std::string_view region, float duration_ms) {
    auto result = nucleation::capi::BuildAnimation_flip_region_z(this->AsFFI(),
        {region.data(), region.size()},
        duration_ms);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::BuildAnimation::flip_all_x(float duration_ms) {
    auto result = nucleation::capi::BuildAnimation_flip_all_x(this->AsFFI(),
        duration_ms);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::BuildAnimation::flip_all_y(float duration_ms) {
    auto result = nucleation::capi::BuildAnimation_flip_all_y(this->AsFFI(),
        duration_ms);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::BuildAnimation::flip_all_z(float duration_ms) {
    auto result = nucleation::capi::BuildAnimation_flip_all_z(this->AsFFI(),
        duration_ms);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::BuildAnimation::stamp_region(const nucleation::Schematic& source, std::string_view region, int32_t x, int32_t y, int32_t z, std::string_view exclusions, float duration_ms) {
    auto result = nucleation::capi::BuildAnimation_stamp_region(this->AsFFI(),
        source.AsFFI(),
        {region.data(), region.size()},
        x,
        y,
        z,
        {exclusions.data(), exclusions.size()},
        duration_ms);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::BuildAnimation::stamp_box(const nucleation::Schematic& source, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z, int32_t x, int32_t y, int32_t z, std::string_view exclusions, float duration_ms) {
    auto result = nucleation::capi::BuildAnimation_stamp_box(this->AsFFI(),
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
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline void nucleation::BuildAnimation::set_operation_gizmos(bool enabled) {
    nucleation::capi::BuildAnimation_set_operation_gizmos(this->AsFFI(),
        enabled);
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::BuildAnimation::operations_json() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::BuildAnimation_operations_json(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::BuildAnimation::operations_json_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::BuildAnimation_operations_json(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::BuildAnimation::frame_json(float time_ms) const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::BuildAnimation_frame_json(this->AsFFI(),
        time_ms,
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::BuildAnimation::frame_json_write(float time_ms, W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::BuildAnimation_frame_json(this->AsFFI(),
        time_ms,
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<uint32_t, nucleation::NucleationError> nucleation::BuildAnimation::fill_along_parameter(const nucleation::Shape& shape, const nucleation::Brush& brush, uint32_t group_count) {
    auto result = nucleation::capi::BuildAnimation_fill_along_parameter(this->AsFFI(),
        shape.AsFFI(),
        brush.AsFFI(),
        group_count);
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

inline nucleation::diplomat::result<uint32_t, nucleation::NucleationError> nucleation::BuildAnimation::render_video_with_pack(const nucleation::ResourcePack& pack, const nucleation::RenderConfig& config, const nucleation::VideoConfig& video, std::string_view path, float hold_ms) const {
    auto result = nucleation::capi::BuildAnimation_render_video_with_pack(this->AsFFI(),
        pack.AsFFI(),
        config.AsFFI(),
        video.AsFFI(),
        {path.data(), path.size()},
        hold_ms);
    return result.is_ok ? nucleation::diplomat::result<uint32_t, nucleation::NucleationError>(nucleation::diplomat::Ok<uint32_t>(result.ok)) : nucleation::diplomat::result<uint32_t, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<uint32_t, nucleation::NucleationError> nucleation::BuildAnimation::render_video(nucleation::diplomat::span<const uint8_t> pack_zip, const nucleation::RenderConfig& config, const nucleation::VideoConfig& video, std::string_view path, float hold_ms) const {
    auto result = nucleation::capi::BuildAnimation_render_video(this->AsFFI(),
        {pack_zip.data(), pack_zip.size()},
        config.AsFFI(),
        video.AsFFI(),
        {path.data(), path.size()},
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
