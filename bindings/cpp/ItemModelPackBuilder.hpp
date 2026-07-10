#ifndef ItemModelPackBuilder_HPP
#define ItemModelPackBuilder_HPP

#include "ItemModelPackBuilder.d.hpp"

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

    diplomat::capi::ItemModelPackBuilder* ItemModelPackBuilder_create(void);

    uint32_t ItemModelPackBuilder_len(const diplomat::capi::ItemModelPackBuilder* self);

    typedef struct ItemModelPackBuilder_build_zip_b64_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} ItemModelPackBuilder_build_zip_b64_result;
    ItemModelPackBuilder_build_zip_b64_result ItemModelPackBuilder_build_zip_b64(const diplomat::capi::ItemModelPackBuilder* self, diplomat::capi::DiplomatWrite* write);

    void ItemModelPackBuilder_destroy(ItemModelPackBuilder* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::unique_ptr<ItemModelPackBuilder> ItemModelPackBuilder::create() {
    auto result = diplomat::capi::ItemModelPackBuilder_create();
    return std::unique_ptr<ItemModelPackBuilder>(ItemModelPackBuilder::FromFFI(result));
}

inline uint32_t ItemModelPackBuilder::len() const {
    auto result = diplomat::capi::ItemModelPackBuilder_len(this->AsFFI());
    return result;
}

inline diplomat::result<std::string, NucleationError> ItemModelPackBuilder::build_zip_b64() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::ItemModelPackBuilder_build_zip_b64(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> ItemModelPackBuilder::build_zip_b64_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::ItemModelPackBuilder_build_zip_b64(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline const diplomat::capi::ItemModelPackBuilder* ItemModelPackBuilder::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::ItemModelPackBuilder*>(this);
}

inline diplomat::capi::ItemModelPackBuilder* ItemModelPackBuilder::AsFFI() {
    return reinterpret_cast<diplomat::capi::ItemModelPackBuilder*>(this);
}

inline const ItemModelPackBuilder* ItemModelPackBuilder::FromFFI(const diplomat::capi::ItemModelPackBuilder* ptr) {
    return reinterpret_cast<const ItemModelPackBuilder*>(ptr);
}

inline ItemModelPackBuilder* ItemModelPackBuilder::FromFFI(diplomat::capi::ItemModelPackBuilder* ptr) {
    return reinterpret_cast<ItemModelPackBuilder*>(ptr);
}

inline void ItemModelPackBuilder::operator delete(void* ptr) {
    diplomat::capi::ItemModelPackBuilder_destroy(reinterpret_cast<diplomat::capi::ItemModelPackBuilder*>(ptr));
}


#endif // ItemModelPackBuilder_HPP
