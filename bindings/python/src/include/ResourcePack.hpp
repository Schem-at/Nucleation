#ifndef NUCLEATION_ResourcePack_HPP
#define NUCLEATION_ResourcePack_HPP

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


namespace nucleation {
namespace capi {
    extern "C" {

    typedef struct ResourcePack_from_bytes_result {union {nucleation::capi::ResourcePack* ok; nucleation::capi::NucleationError err;}; bool is_ok;} ResourcePack_from_bytes_result;
    ResourcePack_from_bytes_result ResourcePack_from_bytes(nucleation::diplomat::capi::DiplomatU8View data);

    typedef struct ResourcePack_from_list_result {union {nucleation::capi::ResourcePack* ok; nucleation::capi::NucleationError err;}; bool is_ok;} ResourcePack_from_list_result;
    ResourcePack_from_list_result ResourcePack_from_list(const nucleation::capi::ResourcePackList* list);

    uint32_t ResourcePack_blockstate_count(const nucleation::capi::ResourcePack* self);

    uint32_t ResourcePack_model_count(const nucleation::capi::ResourcePack* self);

    uint32_t ResourcePack_texture_count(const nucleation::capi::ResourcePack* self);

    void ResourcePack_namespaces_json(const nucleation::capi::ResourcePack* self, nucleation::diplomat::capi::DiplomatWrite* write);

    void ResourcePack_list_blockstates_json(const nucleation::capi::ResourcePack* self, nucleation::diplomat::capi::DiplomatWrite* write);

    void ResourcePack_list_models_json(const nucleation::capi::ResourcePack* self, nucleation::diplomat::capi::DiplomatWrite* write);

    void ResourcePack_list_textures_json(const nucleation::capi::ResourcePack* self, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct ResourcePack_get_blockstate_json_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} ResourcePack_get_blockstate_json_result;
    ResourcePack_get_blockstate_json_result ResourcePack_get_blockstate_json(const nucleation::capi::ResourcePack* self, nucleation::diplomat::capi::DiplomatStringView name, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct ResourcePack_get_model_json_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} ResourcePack_get_model_json_result;
    ResourcePack_get_model_json_result ResourcePack_get_model_json(const nucleation::capi::ResourcePack* self, nucleation::diplomat::capi::DiplomatStringView name, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct ResourcePack_get_texture_info_result {union {nucleation::capi::TextureInfo ok; nucleation::capi::NucleationError err;}; bool is_ok;} ResourcePack_get_texture_info_result;
    ResourcePack_get_texture_info_result ResourcePack_get_texture_info(const nucleation::capi::ResourcePack* self, nucleation::diplomat::capi::DiplomatStringView name);

    typedef struct ResourcePack_get_texture_pixels_b64_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} ResourcePack_get_texture_pixels_b64_result;
    ResourcePack_get_texture_pixels_b64_result ResourcePack_get_texture_pixels_b64(const nucleation::capi::ResourcePack* self, nucleation::diplomat::capi::DiplomatStringView name, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct ResourcePack_add_blockstate_json_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} ResourcePack_add_blockstate_json_result;
    ResourcePack_add_blockstate_json_result ResourcePack_add_blockstate_json(nucleation::capi::ResourcePack* self, nucleation::diplomat::capi::DiplomatStringView name, nucleation::diplomat::capi::DiplomatStringView json);

    typedef struct ResourcePack_add_model_json_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} ResourcePack_add_model_json_result;
    ResourcePack_add_model_json_result ResourcePack_add_model_json(nucleation::capi::ResourcePack* self, nucleation::diplomat::capi::DiplomatStringView name, nucleation::diplomat::capi::DiplomatStringView json);

    typedef struct ResourcePack_add_texture_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} ResourcePack_add_texture_result;
    ResourcePack_add_texture_result ResourcePack_add_texture(nucleation::capi::ResourcePack* self, nucleation::diplomat::capi::DiplomatStringView name, uint32_t width, uint32_t height, nucleation::diplomat::capi::DiplomatU8View pixels);

    typedef struct ResourcePack_register_mesh_exporter_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} ResourcePack_register_mesh_exporter_result;
    ResourcePack_register_mesh_exporter_result ResourcePack_register_mesh_exporter(const nucleation::capi::ResourcePack* self);

    void ResourcePack_destroy(ResourcePack* self);

    } // extern "C"
} // namespace capi
} // namespace

inline nucleation::diplomat::result<std::unique_ptr<nucleation::ResourcePack>, nucleation::NucleationError> nucleation::ResourcePack::from_bytes(nucleation::diplomat::span<const uint8_t> data) {
    auto result = nucleation::capi::ResourcePack_from_bytes({data.data(), data.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::ResourcePack>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::ResourcePack>>(std::unique_ptr<nucleation::ResourcePack>(nucleation::ResourcePack::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::ResourcePack>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::ResourcePack>, nucleation::NucleationError> nucleation::ResourcePack::from_list(const nucleation::ResourcePackList& list) {
    auto result = nucleation::capi::ResourcePack_from_list(list.AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::ResourcePack>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::ResourcePack>>(std::unique_ptr<nucleation::ResourcePack>(nucleation::ResourcePack::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::ResourcePack>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline uint32_t nucleation::ResourcePack::blockstate_count() const {
    auto result = nucleation::capi::ResourcePack_blockstate_count(this->AsFFI());
    return result;
}

inline uint32_t nucleation::ResourcePack::model_count() const {
    auto result = nucleation::capi::ResourcePack_model_count(this->AsFFI());
    return result;
}

inline uint32_t nucleation::ResourcePack::texture_count() const {
    auto result = nucleation::capi::ResourcePack_texture_count(this->AsFFI());
    return result;
}

inline std::string nucleation::ResourcePack::namespaces_json() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::ResourcePack_namespaces_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void nucleation::ResourcePack::namespaces_json_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::ResourcePack_namespaces_json(this->AsFFI(),
        &write);
}

inline std::string nucleation::ResourcePack::list_blockstates_json() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::ResourcePack_list_blockstates_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void nucleation::ResourcePack::list_blockstates_json_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::ResourcePack_list_blockstates_json(this->AsFFI(),
        &write);
}

inline std::string nucleation::ResourcePack::list_models_json() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::ResourcePack_list_models_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void nucleation::ResourcePack::list_models_json_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::ResourcePack_list_models_json(this->AsFFI(),
        &write);
}

inline std::string nucleation::ResourcePack::list_textures_json() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::ResourcePack_list_textures_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void nucleation::ResourcePack::list_textures_json_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::ResourcePack_list_textures_json(this->AsFFI(),
        &write);
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::ResourcePack::get_blockstate_json(std::string_view name) const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::ResourcePack_get_blockstate_json(this->AsFFI(),
        {name.data(), name.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::ResourcePack::get_blockstate_json_write(std::string_view name, W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::ResourcePack_get_blockstate_json(this->AsFFI(),
        {name.data(), name.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::ResourcePack::get_model_json(std::string_view name) const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::ResourcePack_get_model_json(this->AsFFI(),
        {name.data(), name.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::ResourcePack::get_model_json_write(std::string_view name, W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::ResourcePack_get_model_json(this->AsFFI(),
        {name.data(), name.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<nucleation::TextureInfo, nucleation::NucleationError> nucleation::ResourcePack::get_texture_info(std::string_view name) const {
    auto result = nucleation::capi::ResourcePack_get_texture_info(this->AsFFI(),
        {name.data(), name.size()});
    return result.is_ok ? nucleation::diplomat::result<nucleation::TextureInfo, nucleation::NucleationError>(nucleation::diplomat::Ok<nucleation::TextureInfo>(nucleation::TextureInfo::FromFFI(result.ok))) : nucleation::diplomat::result<nucleation::TextureInfo, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::ResourcePack::get_texture_pixels_b64(std::string_view name) const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::ResourcePack_get_texture_pixels_b64(this->AsFFI(),
        {name.data(), name.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::ResourcePack::get_texture_pixels_b64_write(std::string_view name, W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::ResourcePack_get_texture_pixels_b64(this->AsFFI(),
        {name.data(), name.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::ResourcePack::add_blockstate_json(std::string_view name, std::string_view json) {
    auto result = nucleation::capi::ResourcePack_add_blockstate_json(this->AsFFI(),
        {name.data(), name.size()},
        {json.data(), json.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::ResourcePack::add_model_json(std::string_view name, std::string_view json) {
    auto result = nucleation::capi::ResourcePack_add_model_json(this->AsFFI(),
        {name.data(), name.size()},
        {json.data(), json.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::ResourcePack::add_texture(std::string_view name, uint32_t width, uint32_t height, nucleation::diplomat::span<const uint8_t> pixels) {
    auto result = nucleation::capi::ResourcePack_add_texture(this->AsFFI(),
        {name.data(), name.size()},
        width,
        height,
        {pixels.data(), pixels.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::ResourcePack::register_mesh_exporter() const {
    auto result = nucleation::capi::ResourcePack_register_mesh_exporter(this->AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline const nucleation::capi::ResourcePack* nucleation::ResourcePack::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::ResourcePack*>(this);
}

inline nucleation::capi::ResourcePack* nucleation::ResourcePack::AsFFI() {
    return reinterpret_cast<nucleation::capi::ResourcePack*>(this);
}

inline const nucleation::ResourcePack* nucleation::ResourcePack::FromFFI(const nucleation::capi::ResourcePack* ptr) {
    return reinterpret_cast<const nucleation::ResourcePack*>(ptr);
}

inline nucleation::ResourcePack* nucleation::ResourcePack::FromFFI(nucleation::capi::ResourcePack* ptr) {
    return reinterpret_cast<nucleation::ResourcePack*>(ptr);
}

inline void nucleation::ResourcePack::operator delete(void* ptr) {
    nucleation::capi::ResourcePack_destroy(reinterpret_cast<nucleation::capi::ResourcePack*>(ptr));
}


#endif // NUCLEATION_ResourcePack_HPP
