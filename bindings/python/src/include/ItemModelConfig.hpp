#ifndef NUCLEATION_ItemModelConfig_HPP
#define NUCLEATION_ItemModelConfig_HPP

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


namespace nucleation {
namespace capi {
    extern "C" {

    typedef struct ItemModelConfig_create_result {union {nucleation::capi::ItemModelConfig* ok; nucleation::capi::NucleationError err;}; bool is_ok;} ItemModelConfig_create_result;
    ItemModelConfig_create_result ItemModelConfig_create(nucleation::diplomat::capi::DiplomatStringView model_name);

    typedef struct ItemModelConfig_set_namespace_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} ItemModelConfig_set_namespace_result;
    ItemModelConfig_set_namespace_result ItemModelConfig_set_namespace(nucleation::capi::ItemModelConfig* self, nucleation::diplomat::capi::DiplomatStringView namespace_);

    void ItemModelConfig_set_center(nucleation::capi::ItemModelConfig* self, bool center);

    void ItemModelConfig_set_texture_resolution(nucleation::capi::ItemModelConfig* self, uint32_t resolution);

    typedef struct ItemModelConfig_set_item_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} ItemModelConfig_set_item_result;
    ItemModelConfig_set_item_result ItemModelConfig_set_item(nucleation::capi::ItemModelConfig* self, nucleation::diplomat::capi::DiplomatStringView item);

    typedef struct ItemModelConfig_set_custom_model_data_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} ItemModelConfig_set_custom_model_data_result;
    ItemModelConfig_set_custom_model_data_result ItemModelConfig_set_custom_model_data(nucleation::capi::ItemModelConfig* self, nucleation::diplomat::capi::DiplomatStringView cmd);

    void ItemModelConfig_set_scale(nucleation::capi::ItemModelConfig* self, float scale);

    void ItemModelConfig_set_scale_xyz(nucleation::capi::ItemModelConfig* self, float sx, float sy, float sz);

    void ItemModelConfig_set_scale_auto(nucleation::capi::ItemModelConfig* self);

    void ItemModelConfig_destroy(ItemModelConfig* self);

    } // extern "C"
} // namespace capi
} // namespace

inline nucleation::diplomat::result<std::unique_ptr<nucleation::ItemModelConfig>, nucleation::NucleationError> nucleation::ItemModelConfig::create(std::string_view model_name) {
    auto result = nucleation::capi::ItemModelConfig_create({model_name.data(), model_name.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::ItemModelConfig>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::ItemModelConfig>>(std::unique_ptr<nucleation::ItemModelConfig>(nucleation::ItemModelConfig::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::ItemModelConfig>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::ItemModelConfig::set_namespace(std::string_view namespace_) {
    auto result = nucleation::capi::ItemModelConfig_set_namespace(this->AsFFI(),
        {namespace_.data(), namespace_.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline void nucleation::ItemModelConfig::set_center(bool center) {
    nucleation::capi::ItemModelConfig_set_center(this->AsFFI(),
        center);
}

inline void nucleation::ItemModelConfig::set_texture_resolution(uint32_t resolution) {
    nucleation::capi::ItemModelConfig_set_texture_resolution(this->AsFFI(),
        resolution);
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::ItemModelConfig::set_item(std::string_view item) {
    auto result = nucleation::capi::ItemModelConfig_set_item(this->AsFFI(),
        {item.data(), item.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::ItemModelConfig::set_custom_model_data(std::string_view cmd) {
    auto result = nucleation::capi::ItemModelConfig_set_custom_model_data(this->AsFFI(),
        {cmd.data(), cmd.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline void nucleation::ItemModelConfig::set_scale(float scale) {
    nucleation::capi::ItemModelConfig_set_scale(this->AsFFI(),
        scale);
}

inline void nucleation::ItemModelConfig::set_scale_xyz(float sx, float sy, float sz) {
    nucleation::capi::ItemModelConfig_set_scale_xyz(this->AsFFI(),
        sx,
        sy,
        sz);
}

inline void nucleation::ItemModelConfig::set_scale_auto() {
    nucleation::capi::ItemModelConfig_set_scale_auto(this->AsFFI());
}

inline const nucleation::capi::ItemModelConfig* nucleation::ItemModelConfig::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::ItemModelConfig*>(this);
}

inline nucleation::capi::ItemModelConfig* nucleation::ItemModelConfig::AsFFI() {
    return reinterpret_cast<nucleation::capi::ItemModelConfig*>(this);
}

inline const nucleation::ItemModelConfig* nucleation::ItemModelConfig::FromFFI(const nucleation::capi::ItemModelConfig* ptr) {
    return reinterpret_cast<const nucleation::ItemModelConfig*>(ptr);
}

inline nucleation::ItemModelConfig* nucleation::ItemModelConfig::FromFFI(nucleation::capi::ItemModelConfig* ptr) {
    return reinterpret_cast<nucleation::ItemModelConfig*>(ptr);
}

inline void nucleation::ItemModelConfig::operator delete(void* ptr) {
    nucleation::capi::ItemModelConfig_destroy(reinterpret_cast<nucleation::capi::ItemModelConfig*>(ptr));
}


#endif // NUCLEATION_ItemModelConfig_HPP
