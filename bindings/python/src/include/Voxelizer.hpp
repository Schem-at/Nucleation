#ifndef NUCLEATION_Voxelizer_HPP
#define NUCLEATION_Voxelizer_HPP

#include "Voxelizer.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "NucleationError.hpp"
#include "Palette.hpp"
#include "Schematic.hpp"
#include "Shape.hpp"
#include "VoxelModel.hpp"
#include "diplomat_runtime.hpp"


namespace nucleation {
namespace capi {
    extern "C" {

    void Voxelizer_last_error_detail(nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct Voxelizer_load_glb_base64_result {union {nucleation::capi::VoxelModel* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Voxelizer_load_glb_base64_result;
    Voxelizer_load_glb_base64_result Voxelizer_load_glb_base64(nucleation::diplomat::capi::DiplomatStringView data);

    typedef struct Voxelizer_load_glb_result {union {nucleation::capi::VoxelModel* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Voxelizer_load_glb_result;
    Voxelizer_load_glb_result Voxelizer_load_glb(nucleation::diplomat::capi::DiplomatU8View data);

    typedef struct Voxelizer_load_obj_result {union {nucleation::capi::VoxelModel* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Voxelizer_load_obj_result;
    Voxelizer_load_obj_result Voxelizer_load_obj(nucleation::diplomat::capi::DiplomatStringView text);

    typedef struct Voxelizer_shape_from_glb_result {union {nucleation::capi::Shape* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Voxelizer_shape_from_glb_result;
    Voxelizer_shape_from_glb_result Voxelizer_shape_from_glb(nucleation::diplomat::capi::DiplomatU8View data, float target_size, float shell);

    typedef struct Voxelizer_shape_from_obj_result {union {nucleation::capi::Shape* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Voxelizer_shape_from_obj_result;
    Voxelizer_shape_from_obj_result Voxelizer_shape_from_obj(nucleation::diplomat::capi::DiplomatStringView text, float target_size, float shell);

    typedef struct Voxelizer_schematic_from_glb_textured_result {union {nucleation::capi::Schematic* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Voxelizer_schematic_from_glb_textured_result;
    Voxelizer_schematic_from_glb_textured_result Voxelizer_schematic_from_glb_textured(nucleation::diplomat::capi::DiplomatU8View data, float target_size, float shell, const nucleation::capi::Palette* palette, nucleation::diplomat::capi::DiplomatStringView name);

    void Voxelizer_destroy(Voxelizer* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::string nucleation::Voxelizer::last_error_detail() {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::Voxelizer_last_error_detail(&write);
    return output;
}
template<typename W>
inline void nucleation::Voxelizer::last_error_detail_write(W& writeable) {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::Voxelizer_last_error_detail(&write);
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::VoxelModel>, nucleation::NucleationError> nucleation::Voxelizer::load_glb_base64(std::string_view data) {
    auto result = nucleation::capi::Voxelizer_load_glb_base64({data.data(), data.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::VoxelModel>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::VoxelModel>>(std::unique_ptr<nucleation::VoxelModel>(nucleation::VoxelModel::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::VoxelModel>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::VoxelModel>, nucleation::NucleationError> nucleation::Voxelizer::load_glb(nucleation::diplomat::span<const uint8_t> data) {
    auto result = nucleation::capi::Voxelizer_load_glb({data.data(), data.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::VoxelModel>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::VoxelModel>>(std::unique_ptr<nucleation::VoxelModel>(nucleation::VoxelModel::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::VoxelModel>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::VoxelModel>, nucleation::NucleationError> nucleation::Voxelizer::load_obj(std::string_view text) {
    auto result = nucleation::capi::Voxelizer_load_obj({text.data(), text.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::VoxelModel>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::VoxelModel>>(std::unique_ptr<nucleation::VoxelModel>(nucleation::VoxelModel::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::VoxelModel>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Shape>, nucleation::NucleationError> nucleation::Voxelizer::shape_from_glb(nucleation::diplomat::span<const uint8_t> data, float target_size, float shell) {
    auto result = nucleation::capi::Voxelizer_shape_from_glb({data.data(), data.size()},
        target_size,
        shell);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Shape>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Shape>>(std::unique_ptr<nucleation::Shape>(nucleation::Shape::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Shape>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Shape>, nucleation::NucleationError> nucleation::Voxelizer::shape_from_obj(std::string_view text, float target_size, float shell) {
    auto result = nucleation::capi::Voxelizer_shape_from_obj({text.data(), text.size()},
        target_size,
        shell);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Shape>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Shape>>(std::unique_ptr<nucleation::Shape>(nucleation::Shape::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Shape>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> nucleation::Voxelizer::schematic_from_glb_textured(nucleation::diplomat::span<const uint8_t> data, float target_size, float shell, const nucleation::Palette& palette, std::string_view name) {
    auto result = nucleation::capi::Voxelizer_schematic_from_glb_textured({data.data(), data.size()},
        target_size,
        shell,
        palette.AsFFI(),
        {name.data(), name.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Schematic>>(std::unique_ptr<nucleation::Schematic>(nucleation::Schematic::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline const nucleation::capi::Voxelizer* nucleation::Voxelizer::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::Voxelizer*>(this);
}

inline nucleation::capi::Voxelizer* nucleation::Voxelizer::AsFFI() {
    return reinterpret_cast<nucleation::capi::Voxelizer*>(this);
}

inline const nucleation::Voxelizer* nucleation::Voxelizer::FromFFI(const nucleation::capi::Voxelizer* ptr) {
    return reinterpret_cast<const nucleation::Voxelizer*>(ptr);
}

inline nucleation::Voxelizer* nucleation::Voxelizer::FromFFI(nucleation::capi::Voxelizer* ptr) {
    return reinterpret_cast<nucleation::Voxelizer*>(ptr);
}

inline void nucleation::Voxelizer::operator delete(void* ptr) {
    nucleation::capi::Voxelizer_destroy(reinterpret_cast<nucleation::capi::Voxelizer*>(ptr));
}


#endif // NUCLEATION_Voxelizer_HPP
