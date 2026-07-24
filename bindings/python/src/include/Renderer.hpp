#ifndef NUCLEATION_Renderer_HPP
#define NUCLEATION_Renderer_HPP

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


namespace nucleation {
namespace capi {
    extern "C" {

    typedef struct Renderer_render_pixels_b64_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Renderer_render_pixels_b64_result;
    Renderer_render_pixels_b64_result Renderer_render_pixels_b64(const nucleation::capi::Schematic* schematic, nucleation::diplomat::capi::DiplomatU8View pack_zip, const nucleation::capi::RenderConfig* config, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct Renderer_render_png_b64_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Renderer_render_png_b64_result;
    Renderer_render_png_b64_result Renderer_render_png_b64(const nucleation::capi::Schematic* schematic, nucleation::diplomat::capi::DiplomatU8View pack_zip, const nucleation::capi::RenderConfig* config, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct Renderer_render_to_file_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Renderer_render_to_file_result;
    Renderer_render_to_file_result Renderer_render_to_file(const nucleation::capi::Schematic* schematic, nucleation::diplomat::capi::DiplomatU8View pack_zip, const nucleation::capi::RenderConfig* config, nucleation::diplomat::capi::DiplomatStringView path);

    typedef struct Renderer_render_to_file_with_pack_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Renderer_render_to_file_with_pack_result;
    Renderer_render_to_file_with_pack_result Renderer_render_to_file_with_pack(const nucleation::capi::Schematic* schematic, const nucleation::capi::ResourcePack* pack, const nucleation::capi::RenderConfig* config, nucleation::diplomat::capi::DiplomatStringView path);

    typedef struct Renderer_render_pixels_b64_with_pack_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Renderer_render_pixels_b64_with_pack_result;
    Renderer_render_pixels_b64_with_pack_result Renderer_render_pixels_b64_with_pack(const nucleation::capi::Schematic* schematic, const nucleation::capi::ResourcePack* pack, const nucleation::capi::RenderConfig* config, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct Renderer_render_png_b64_with_pack_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Renderer_render_png_b64_with_pack_result;
    Renderer_render_png_b64_with_pack_result Renderer_render_png_b64_with_pack(const nucleation::capi::Schematic* schematic, const nucleation::capi::ResourcePack* pack, const nucleation::capi::RenderConfig* config, nucleation::diplomat::capi::DiplomatWrite* write);

    void Renderer_destroy(Renderer* self);

    } // extern "C"
} // namespace capi
} // namespace

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::Renderer::render_pixels_b64(const nucleation::Schematic& schematic, nucleation::diplomat::span<const uint8_t> pack_zip, const nucleation::RenderConfig& config) {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::Renderer_render_pixels_b64(schematic.AsFFI(),
        {pack_zip.data(), pack_zip.size()},
        config.AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Renderer::render_pixels_b64_write(const nucleation::Schematic& schematic, nucleation::diplomat::span<const uint8_t> pack_zip, const nucleation::RenderConfig& config, W& writeable) {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::Renderer_render_pixels_b64(schematic.AsFFI(),
        {pack_zip.data(), pack_zip.size()},
        config.AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::Renderer::render_png_b64(const nucleation::Schematic& schematic, nucleation::diplomat::span<const uint8_t> pack_zip, const nucleation::RenderConfig& config) {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::Renderer_render_png_b64(schematic.AsFFI(),
        {pack_zip.data(), pack_zip.size()},
        config.AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Renderer::render_png_b64_write(const nucleation::Schematic& schematic, nucleation::diplomat::span<const uint8_t> pack_zip, const nucleation::RenderConfig& config, W& writeable) {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::Renderer_render_png_b64(schematic.AsFFI(),
        {pack_zip.data(), pack_zip.size()},
        config.AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Renderer::render_to_file(const nucleation::Schematic& schematic, nucleation::diplomat::span<const uint8_t> pack_zip, const nucleation::RenderConfig& config, std::string_view path) {
    auto result = nucleation::capi::Renderer_render_to_file(schematic.AsFFI(),
        {pack_zip.data(), pack_zip.size()},
        config.AsFFI(),
        {path.data(), path.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Renderer::render_to_file_with_pack(const nucleation::Schematic& schematic, const nucleation::ResourcePack& pack, const nucleation::RenderConfig& config, std::string_view path) {
    auto result = nucleation::capi::Renderer_render_to_file_with_pack(schematic.AsFFI(),
        pack.AsFFI(),
        config.AsFFI(),
        {path.data(), path.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::Renderer::render_pixels_b64_with_pack(const nucleation::Schematic& schematic, const nucleation::ResourcePack& pack, const nucleation::RenderConfig& config) {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::Renderer_render_pixels_b64_with_pack(schematic.AsFFI(),
        pack.AsFFI(),
        config.AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Renderer::render_pixels_b64_with_pack_write(const nucleation::Schematic& schematic, const nucleation::ResourcePack& pack, const nucleation::RenderConfig& config, W& writeable) {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::Renderer_render_pixels_b64_with_pack(schematic.AsFFI(),
        pack.AsFFI(),
        config.AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::Renderer::render_png_b64_with_pack(const nucleation::Schematic& schematic, const nucleation::ResourcePack& pack, const nucleation::RenderConfig& config) {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::Renderer_render_png_b64_with_pack(schematic.AsFFI(),
        pack.AsFFI(),
        config.AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Renderer::render_png_b64_with_pack_write(const nucleation::Schematic& schematic, const nucleation::ResourcePack& pack, const nucleation::RenderConfig& config, W& writeable) {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::Renderer_render_png_b64_with_pack(schematic.AsFFI(),
        pack.AsFFI(),
        config.AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline const nucleation::capi::Renderer* nucleation::Renderer::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::Renderer*>(this);
}

inline nucleation::capi::Renderer* nucleation::Renderer::AsFFI() {
    return reinterpret_cast<nucleation::capi::Renderer*>(this);
}

inline const nucleation::Renderer* nucleation::Renderer::FromFFI(const nucleation::capi::Renderer* ptr) {
    return reinterpret_cast<const nucleation::Renderer*>(ptr);
}

inline nucleation::Renderer* nucleation::Renderer::FromFFI(nucleation::capi::Renderer* ptr) {
    return reinterpret_cast<nucleation::Renderer*>(ptr);
}

inline void nucleation::Renderer::operator delete(void* ptr) {
    nucleation::capi::Renderer_destroy(reinterpret_cast<nucleation::capi::Renderer*>(ptr));
}


#endif // NUCLEATION_Renderer_HPP
