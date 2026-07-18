#ifndef Voxelizer_HPP
#define Voxelizer_HPP

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
#include "diplomat_runtime.hpp"


namespace diplomat {
namespace capi {
    extern "C" {

    typedef struct Voxelizer_shape_from_glb_result {union {diplomat::capi::Shape* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Voxelizer_shape_from_glb_result;
    Voxelizer_shape_from_glb_result Voxelizer_shape_from_glb(diplomat::capi::DiplomatU8View data, float target_size, float shell);

    typedef struct Voxelizer_shape_from_obj_result {union {diplomat::capi::Shape* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Voxelizer_shape_from_obj_result;
    Voxelizer_shape_from_obj_result Voxelizer_shape_from_obj(diplomat::capi::DiplomatStringView text, float target_size, float shell);

    typedef struct Voxelizer_schematic_from_glb_textured_result {union {diplomat::capi::Schematic* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Voxelizer_schematic_from_glb_textured_result;
    Voxelizer_schematic_from_glb_textured_result Voxelizer_schematic_from_glb_textured(diplomat::capi::DiplomatU8View data, float target_size, float shell, const diplomat::capi::Palette* palette, diplomat::capi::DiplomatStringView name);

    void Voxelizer_destroy(Voxelizer* self);

    } // extern "C"
} // namespace capi
} // namespace

inline diplomat::result<std::unique_ptr<Shape>, NucleationError> Voxelizer::shape_from_glb(diplomat::span<const uint8_t> data, float target_size, float shell) {
    auto result = diplomat::capi::Voxelizer_shape_from_glb({data.data(), data.size()},
        target_size,
        shell);
    return result.is_ok ? diplomat::result<std::unique_ptr<Shape>, NucleationError>(diplomat::Ok<std::unique_ptr<Shape>>(std::unique_ptr<Shape>(Shape::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Shape>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Shape>, NucleationError> Voxelizer::shape_from_obj(std::string_view text, float target_size, float shell) {
    auto result = diplomat::capi::Voxelizer_shape_from_obj({text.data(), text.size()},
        target_size,
        shell);
    return result.is_ok ? diplomat::result<std::unique_ptr<Shape>, NucleationError>(diplomat::Ok<std::unique_ptr<Shape>>(std::unique_ptr<Shape>(Shape::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Shape>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Schematic>, NucleationError> Voxelizer::schematic_from_glb_textured(diplomat::span<const uint8_t> data, float target_size, float shell, const Palette& palette, std::string_view name) {
    auto result = diplomat::capi::Voxelizer_schematic_from_glb_textured({data.data(), data.size()},
        target_size,
        shell,
        palette.AsFFI(),
        {name.data(), name.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Ok<std::unique_ptr<Schematic>>(std::unique_ptr<Schematic>(Schematic::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline const diplomat::capi::Voxelizer* Voxelizer::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::Voxelizer*>(this);
}

inline diplomat::capi::Voxelizer* Voxelizer::AsFFI() {
    return reinterpret_cast<diplomat::capi::Voxelizer*>(this);
}

inline const Voxelizer* Voxelizer::FromFFI(const diplomat::capi::Voxelizer* ptr) {
    return reinterpret_cast<const Voxelizer*>(ptr);
}

inline Voxelizer* Voxelizer::FromFFI(diplomat::capi::Voxelizer* ptr) {
    return reinterpret_cast<Voxelizer*>(ptr);
}

inline void Voxelizer::operator delete(void* ptr) {
    diplomat::capi::Voxelizer_destroy(reinterpret_cast<diplomat::capi::Voxelizer*>(ptr));
}


#endif // Voxelizer_HPP
