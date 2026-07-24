#ifndef Renderer_HPP
#define Renderer_HPP

#include "Renderer.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "NucleationError.hpp"
#include "RenderConfig.hpp"
#include "ResourcePack.hpp"
#include "Schematic.hpp"
#include "diplomat_runtime.hpp"


namespace diplomat {
namespace capi {
    extern "C" {

    typedef struct Renderer_render_pixels_b64_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Renderer_render_pixels_b64_result;
    Renderer_render_pixels_b64_result Renderer_render_pixels_b64(const diplomat::capi::Schematic* schematic, diplomat::capi::DiplomatU8View pack_zip, const diplomat::capi::RenderConfig* config, diplomat::capi::DiplomatWrite* write);

    typedef struct Renderer_render_png_b64_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Renderer_render_png_b64_result;
    Renderer_render_png_b64_result Renderer_render_png_b64(const diplomat::capi::Schematic* schematic, diplomat::capi::DiplomatU8View pack_zip, const diplomat::capi::RenderConfig* config, diplomat::capi::DiplomatWrite* write);

    typedef struct Renderer_render_to_file_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Renderer_render_to_file_result;
    Renderer_render_to_file_result Renderer_render_to_file(const diplomat::capi::Schematic* schematic, diplomat::capi::DiplomatU8View pack_zip, const diplomat::capi::RenderConfig* config, diplomat::capi::DiplomatStringView path);

    typedef struct Renderer_render_to_file_with_pack_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Renderer_render_to_file_with_pack_result;
    Renderer_render_to_file_with_pack_result Renderer_render_to_file_with_pack(const diplomat::capi::Schematic* schematic, const diplomat::capi::ResourcePack* pack, const diplomat::capi::RenderConfig* config, diplomat::capi::DiplomatStringView path);

    typedef struct Renderer_render_pixels_b64_with_pack_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Renderer_render_pixels_b64_with_pack_result;
    Renderer_render_pixels_b64_with_pack_result Renderer_render_pixels_b64_with_pack(const diplomat::capi::Schematic* schematic, const diplomat::capi::ResourcePack* pack, const diplomat::capi::RenderConfig* config, diplomat::capi::DiplomatWrite* write);

    typedef struct Renderer_render_png_b64_with_pack_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Renderer_render_png_b64_with_pack_result;
    Renderer_render_png_b64_with_pack_result Renderer_render_png_b64_with_pack(const diplomat::capi::Schematic* schematic, const diplomat::capi::ResourcePack* pack, const diplomat::capi::RenderConfig* config, diplomat::capi::DiplomatWrite* write);

    void Renderer_destroy(Renderer* self);

    } // extern "C"
} // namespace capi
} // namespace

inline diplomat::result<std::string, NucleationError> Renderer::render_pixels_b64(const Schematic& schematic, diplomat::span<const uint8_t> pack_zip, const RenderConfig& config) {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::Renderer_render_pixels_b64(schematic.AsFFI(),
        {pack_zip.data(), pack_zip.size()},
        config.AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> Renderer::render_pixels_b64_write(const Schematic& schematic, diplomat::span<const uint8_t> pack_zip, const RenderConfig& config, W& writeable) {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::Renderer_render_pixels_b64(schematic.AsFFI(),
        {pack_zip.data(), pack_zip.size()},
        config.AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> Renderer::render_png_b64(const Schematic& schematic, diplomat::span<const uint8_t> pack_zip, const RenderConfig& config) {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::Renderer_render_png_b64(schematic.AsFFI(),
        {pack_zip.data(), pack_zip.size()},
        config.AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> Renderer::render_png_b64_write(const Schematic& schematic, diplomat::span<const uint8_t> pack_zip, const RenderConfig& config, W& writeable) {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::Renderer_render_png_b64(schematic.AsFFI(),
        {pack_zip.data(), pack_zip.size()},
        config.AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> Renderer::render_to_file(const Schematic& schematic, diplomat::span<const uint8_t> pack_zip, const RenderConfig& config, std::string_view path) {
    auto result = diplomat::capi::Renderer_render_to_file(schematic.AsFFI(),
        {pack_zip.data(), pack_zip.size()},
        config.AsFFI(),
        {path.data(), path.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> Renderer::render_to_file_with_pack(const Schematic& schematic, const ResourcePack& pack, const RenderConfig& config, std::string_view path) {
    auto result = diplomat::capi::Renderer_render_to_file_with_pack(schematic.AsFFI(),
        pack.AsFFI(),
        config.AsFFI(),
        {path.data(), path.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> Renderer::render_pixels_b64_with_pack(const Schematic& schematic, const ResourcePack& pack, const RenderConfig& config) {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::Renderer_render_pixels_b64_with_pack(schematic.AsFFI(),
        pack.AsFFI(),
        config.AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> Renderer::render_pixels_b64_with_pack_write(const Schematic& schematic, const ResourcePack& pack, const RenderConfig& config, W& writeable) {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::Renderer_render_pixels_b64_with_pack(schematic.AsFFI(),
        pack.AsFFI(),
        config.AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> Renderer::render_png_b64_with_pack(const Schematic& schematic, const ResourcePack& pack, const RenderConfig& config) {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::Renderer_render_png_b64_with_pack(schematic.AsFFI(),
        pack.AsFFI(),
        config.AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> Renderer::render_png_b64_with_pack_write(const Schematic& schematic, const ResourcePack& pack, const RenderConfig& config, W& writeable) {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::Renderer_render_png_b64_with_pack(schematic.AsFFI(),
        pack.AsFFI(),
        config.AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline const diplomat::capi::Renderer* Renderer::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::Renderer*>(this);
}

inline diplomat::capi::Renderer* Renderer::AsFFI() {
    return reinterpret_cast<diplomat::capi::Renderer*>(this);
}

inline const Renderer* Renderer::FromFFI(const diplomat::capi::Renderer* ptr) {
    return reinterpret_cast<const Renderer*>(ptr);
}

inline Renderer* Renderer::FromFFI(diplomat::capi::Renderer* ptr) {
    return reinterpret_cast<Renderer*>(ptr);
}

inline void Renderer::operator delete(void* ptr) {
    diplomat::capi::Renderer_destroy(reinterpret_cast<diplomat::capi::Renderer*>(ptr));
}


#endif // Renderer_HPP
