#ifndef ItemModelConfig_HPP
#define ItemModelConfig_HPP

#include "ItemModelConfig.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "NucleationError.hpp"
#include "diplomat_runtime.hpp"


namespace diplomat {
namespace capi {
    extern "C" {

    typedef struct ItemModelConfig_create_result {union {diplomat::capi::ItemModelConfig* ok; diplomat::capi::NucleationError err;}; bool is_ok;} ItemModelConfig_create_result;
    ItemModelConfig_create_result ItemModelConfig_create(diplomat::capi::DiplomatStringView model_name);

    typedef struct ItemModelConfig_set_namespace_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} ItemModelConfig_set_namespace_result;
    ItemModelConfig_set_namespace_result ItemModelConfig_set_namespace(diplomat::capi::ItemModelConfig* self, diplomat::capi::DiplomatStringView namespace_);

    void ItemModelConfig_set_center(diplomat::capi::ItemModelConfig* self, bool center);

    void ItemModelConfig_set_texture_resolution(diplomat::capi::ItemModelConfig* self, uint32_t resolution);

    typedef struct ItemModelConfig_set_item_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} ItemModelConfig_set_item_result;
    ItemModelConfig_set_item_result ItemModelConfig_set_item(diplomat::capi::ItemModelConfig* self, diplomat::capi::DiplomatStringView item);

    typedef struct ItemModelConfig_set_custom_model_data_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} ItemModelConfig_set_custom_model_data_result;
    ItemModelConfig_set_custom_model_data_result ItemModelConfig_set_custom_model_data(diplomat::capi::ItemModelConfig* self, diplomat::capi::DiplomatStringView cmd);

    void ItemModelConfig_set_scale(diplomat::capi::ItemModelConfig* self, float scale);

    void ItemModelConfig_set_scale_xyz(diplomat::capi::ItemModelConfig* self, float sx, float sy, float sz);

    void ItemModelConfig_set_scale_auto(diplomat::capi::ItemModelConfig* self);

    void ItemModelConfig_destroy(ItemModelConfig* self);

    } // extern "C"
} // namespace capi
} // namespace

inline diplomat::result<std::unique_ptr<ItemModelConfig>, NucleationError> ItemModelConfig::create(std::string_view model_name) {
    auto result = diplomat::capi::ItemModelConfig_create({model_name.data(), model_name.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<ItemModelConfig>, NucleationError>(diplomat::Ok<std::unique_ptr<ItemModelConfig>>(std::unique_ptr<ItemModelConfig>(ItemModelConfig::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<ItemModelConfig>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> ItemModelConfig::set_namespace(std::string_view namespace_) {
    auto result = diplomat::capi::ItemModelConfig_set_namespace(this->AsFFI(),
        {namespace_.data(), namespace_.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline void ItemModelConfig::set_center(bool center) {
    diplomat::capi::ItemModelConfig_set_center(this->AsFFI(),
        center);
}

inline void ItemModelConfig::set_texture_resolution(uint32_t resolution) {
    diplomat::capi::ItemModelConfig_set_texture_resolution(this->AsFFI(),
        resolution);
}

inline diplomat::result<std::monostate, NucleationError> ItemModelConfig::set_item(std::string_view item) {
    auto result = diplomat::capi::ItemModelConfig_set_item(this->AsFFI(),
        {item.data(), item.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> ItemModelConfig::set_custom_model_data(std::string_view cmd) {
    auto result = diplomat::capi::ItemModelConfig_set_custom_model_data(this->AsFFI(),
        {cmd.data(), cmd.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline void ItemModelConfig::set_scale(float scale) {
    diplomat::capi::ItemModelConfig_set_scale(this->AsFFI(),
        scale);
}

inline void ItemModelConfig::set_scale_xyz(float sx, float sy, float sz) {
    diplomat::capi::ItemModelConfig_set_scale_xyz(this->AsFFI(),
        sx,
        sy,
        sz);
}

inline void ItemModelConfig::set_scale_auto() {
    diplomat::capi::ItemModelConfig_set_scale_auto(this->AsFFI());
}

inline const diplomat::capi::ItemModelConfig* ItemModelConfig::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::ItemModelConfig*>(this);
}

inline diplomat::capi::ItemModelConfig* ItemModelConfig::AsFFI() {
    return reinterpret_cast<diplomat::capi::ItemModelConfig*>(this);
}

inline const ItemModelConfig* ItemModelConfig::FromFFI(const diplomat::capi::ItemModelConfig* ptr) {
    return reinterpret_cast<const ItemModelConfig*>(ptr);
}

inline ItemModelConfig* ItemModelConfig::FromFFI(diplomat::capi::ItemModelConfig* ptr) {
    return reinterpret_cast<ItemModelConfig*>(ptr);
}

inline void ItemModelConfig::operator delete(void* ptr) {
    diplomat::capi::ItemModelConfig_destroy(reinterpret_cast<diplomat::capi::ItemModelConfig*>(ptr));
}


#endif // ItemModelConfig_HPP
