#ifndef ResourcePack_HPP
#define ResourcePack_HPP

#include "ResourcePack.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "NucleationError.hpp"
#include "ResourcePackList.hpp"
#include "TextureInfo.hpp"
#include "diplomat_runtime.hpp"


namespace diplomat {
namespace capi {
    extern "C" {

    typedef struct ResourcePack_from_bytes_result {union {diplomat::capi::ResourcePack* ok; diplomat::capi::NucleationError err;}; bool is_ok;} ResourcePack_from_bytes_result;
    ResourcePack_from_bytes_result ResourcePack_from_bytes(diplomat::capi::DiplomatU8View data);

    typedef struct ResourcePack_from_list_result {union {diplomat::capi::ResourcePack* ok; diplomat::capi::NucleationError err;}; bool is_ok;} ResourcePack_from_list_result;
    ResourcePack_from_list_result ResourcePack_from_list(const diplomat::capi::ResourcePackList* list);

    uint32_t ResourcePack_blockstate_count(const diplomat::capi::ResourcePack* self);

    uint32_t ResourcePack_model_count(const diplomat::capi::ResourcePack* self);

    uint32_t ResourcePack_texture_count(const diplomat::capi::ResourcePack* self);

    void ResourcePack_namespaces_json(const diplomat::capi::ResourcePack* self, diplomat::capi::DiplomatWrite* write);

    void ResourcePack_list_blockstates_json(const diplomat::capi::ResourcePack* self, diplomat::capi::DiplomatWrite* write);

    void ResourcePack_list_models_json(const diplomat::capi::ResourcePack* self, diplomat::capi::DiplomatWrite* write);

    void ResourcePack_list_textures_json(const diplomat::capi::ResourcePack* self, diplomat::capi::DiplomatWrite* write);

    typedef struct ResourcePack_get_blockstate_json_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} ResourcePack_get_blockstate_json_result;
    ResourcePack_get_blockstate_json_result ResourcePack_get_blockstate_json(const diplomat::capi::ResourcePack* self, diplomat::capi::DiplomatStringView name, diplomat::capi::DiplomatWrite* write);

    typedef struct ResourcePack_get_model_json_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} ResourcePack_get_model_json_result;
    ResourcePack_get_model_json_result ResourcePack_get_model_json(const diplomat::capi::ResourcePack* self, diplomat::capi::DiplomatStringView name, diplomat::capi::DiplomatWrite* write);

    typedef struct ResourcePack_get_texture_info_result {union {diplomat::capi::TextureInfo ok; diplomat::capi::NucleationError err;}; bool is_ok;} ResourcePack_get_texture_info_result;
    ResourcePack_get_texture_info_result ResourcePack_get_texture_info(const diplomat::capi::ResourcePack* self, diplomat::capi::DiplomatStringView name);

    typedef struct ResourcePack_get_texture_pixels_b64_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} ResourcePack_get_texture_pixels_b64_result;
    ResourcePack_get_texture_pixels_b64_result ResourcePack_get_texture_pixels_b64(const diplomat::capi::ResourcePack* self, diplomat::capi::DiplomatStringView name, diplomat::capi::DiplomatWrite* write);

    typedef struct ResourcePack_add_blockstate_json_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} ResourcePack_add_blockstate_json_result;
    ResourcePack_add_blockstate_json_result ResourcePack_add_blockstate_json(diplomat::capi::ResourcePack* self, diplomat::capi::DiplomatStringView name, diplomat::capi::DiplomatStringView json);

    typedef struct ResourcePack_add_model_json_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} ResourcePack_add_model_json_result;
    ResourcePack_add_model_json_result ResourcePack_add_model_json(diplomat::capi::ResourcePack* self, diplomat::capi::DiplomatStringView name, diplomat::capi::DiplomatStringView json);

    typedef struct ResourcePack_add_texture_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} ResourcePack_add_texture_result;
    ResourcePack_add_texture_result ResourcePack_add_texture(diplomat::capi::ResourcePack* self, diplomat::capi::DiplomatStringView name, uint32_t width, uint32_t height, diplomat::capi::DiplomatU8View pixels);

    typedef struct ResourcePack_register_mesh_exporter_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} ResourcePack_register_mesh_exporter_result;
    ResourcePack_register_mesh_exporter_result ResourcePack_register_mesh_exporter(const diplomat::capi::ResourcePack* self);

    void ResourcePack_destroy(ResourcePack* self);

    } // extern "C"
} // namespace capi
} // namespace

inline diplomat::result<std::unique_ptr<ResourcePack>, NucleationError> ResourcePack::from_bytes(diplomat::span<const uint8_t> data) {
    auto result = diplomat::capi::ResourcePack_from_bytes({data.data(), data.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<ResourcePack>, NucleationError>(diplomat::Ok<std::unique_ptr<ResourcePack>>(std::unique_ptr<ResourcePack>(ResourcePack::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<ResourcePack>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<ResourcePack>, NucleationError> ResourcePack::from_list(const ResourcePackList& list) {
    auto result = diplomat::capi::ResourcePack_from_list(list.AsFFI());
    return result.is_ok ? diplomat::result<std::unique_ptr<ResourcePack>, NucleationError>(diplomat::Ok<std::unique_ptr<ResourcePack>>(std::unique_ptr<ResourcePack>(ResourcePack::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<ResourcePack>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline uint32_t ResourcePack::blockstate_count() const {
    auto result = diplomat::capi::ResourcePack_blockstate_count(this->AsFFI());
    return result;
}

inline uint32_t ResourcePack::model_count() const {
    auto result = diplomat::capi::ResourcePack_model_count(this->AsFFI());
    return result;
}

inline uint32_t ResourcePack::texture_count() const {
    auto result = diplomat::capi::ResourcePack_texture_count(this->AsFFI());
    return result;
}

inline std::string ResourcePack::namespaces_json() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::ResourcePack_namespaces_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void ResourcePack::namespaces_json_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::ResourcePack_namespaces_json(this->AsFFI(),
        &write);
}

inline std::string ResourcePack::list_blockstates_json() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::ResourcePack_list_blockstates_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void ResourcePack::list_blockstates_json_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::ResourcePack_list_blockstates_json(this->AsFFI(),
        &write);
}

inline std::string ResourcePack::list_models_json() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::ResourcePack_list_models_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void ResourcePack::list_models_json_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::ResourcePack_list_models_json(this->AsFFI(),
        &write);
}

inline std::string ResourcePack::list_textures_json() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::ResourcePack_list_textures_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void ResourcePack::list_textures_json_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::ResourcePack_list_textures_json(this->AsFFI(),
        &write);
}

inline diplomat::result<std::string, NucleationError> ResourcePack::get_blockstate_json(std::string_view name) const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::ResourcePack_get_blockstate_json(this->AsFFI(),
        {name.data(), name.size()},
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> ResourcePack::get_blockstate_json_write(std::string_view name, W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::ResourcePack_get_blockstate_json(this->AsFFI(),
        {name.data(), name.size()},
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> ResourcePack::get_model_json(std::string_view name) const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::ResourcePack_get_model_json(this->AsFFI(),
        {name.data(), name.size()},
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> ResourcePack::get_model_json_write(std::string_view name, W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::ResourcePack_get_model_json(this->AsFFI(),
        {name.data(), name.size()},
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<TextureInfo, NucleationError> ResourcePack::get_texture_info(std::string_view name) const {
    auto result = diplomat::capi::ResourcePack_get_texture_info(this->AsFFI(),
        {name.data(), name.size()});
    return result.is_ok ? diplomat::result<TextureInfo, NucleationError>(diplomat::Ok<TextureInfo>(TextureInfo::FromFFI(result.ok))) : diplomat::result<TextureInfo, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> ResourcePack::get_texture_pixels_b64(std::string_view name) const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::ResourcePack_get_texture_pixels_b64(this->AsFFI(),
        {name.data(), name.size()},
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> ResourcePack::get_texture_pixels_b64_write(std::string_view name, W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::ResourcePack_get_texture_pixels_b64(this->AsFFI(),
        {name.data(), name.size()},
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> ResourcePack::add_blockstate_json(std::string_view name, std::string_view json) {
    auto result = diplomat::capi::ResourcePack_add_blockstate_json(this->AsFFI(),
        {name.data(), name.size()},
        {json.data(), json.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> ResourcePack::add_model_json(std::string_view name, std::string_view json) {
    auto result = diplomat::capi::ResourcePack_add_model_json(this->AsFFI(),
        {name.data(), name.size()},
        {json.data(), json.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> ResourcePack::add_texture(std::string_view name, uint32_t width, uint32_t height, diplomat::span<const uint8_t> pixels) {
    auto result = diplomat::capi::ResourcePack_add_texture(this->AsFFI(),
        {name.data(), name.size()},
        width,
        height,
        {pixels.data(), pixels.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> ResourcePack::register_mesh_exporter() const {
    auto result = diplomat::capi::ResourcePack_register_mesh_exporter(this->AsFFI());
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline const diplomat::capi::ResourcePack* ResourcePack::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::ResourcePack*>(this);
}

inline diplomat::capi::ResourcePack* ResourcePack::AsFFI() {
    return reinterpret_cast<diplomat::capi::ResourcePack*>(this);
}

inline const ResourcePack* ResourcePack::FromFFI(const diplomat::capi::ResourcePack* ptr) {
    return reinterpret_cast<const ResourcePack*>(ptr);
}

inline ResourcePack* ResourcePack::FromFFI(diplomat::capi::ResourcePack* ptr) {
    return reinterpret_cast<ResourcePack*>(ptr);
}

inline void ResourcePack::operator delete(void* ptr) {
    diplomat::capi::ResourcePack_destroy(reinterpret_cast<diplomat::capi::ResourcePack*>(ptr));
}


#endif // ResourcePack_HPP
