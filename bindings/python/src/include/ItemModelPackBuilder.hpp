#ifndef NUCLEATION_ItemModelPackBuilder_HPP
#define NUCLEATION_ItemModelPackBuilder_HPP

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


namespace nucleation {
namespace capi {
    extern "C" {

    nucleation::capi::ItemModelPackBuilder* ItemModelPackBuilder_create(void);

    uint32_t ItemModelPackBuilder_len(const nucleation::capi::ItemModelPackBuilder* self);

    typedef struct ItemModelPackBuilder_build_zip_b64_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} ItemModelPackBuilder_build_zip_b64_result;
    ItemModelPackBuilder_build_zip_b64_result ItemModelPackBuilder_build_zip_b64(const nucleation::capi::ItemModelPackBuilder* self, nucleation::diplomat::capi::DiplomatWrite* write);

    void ItemModelPackBuilder_destroy(ItemModelPackBuilder* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::unique_ptr<nucleation::ItemModelPackBuilder> nucleation::ItemModelPackBuilder::create() {
    auto result = nucleation::capi::ItemModelPackBuilder_create();
    return std::unique_ptr<nucleation::ItemModelPackBuilder>(nucleation::ItemModelPackBuilder::FromFFI(result));
}

inline uint32_t nucleation::ItemModelPackBuilder::len() const {
    auto result = nucleation::capi::ItemModelPackBuilder_len(this->AsFFI());
    return result;
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::ItemModelPackBuilder::build_zip_b64() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::ItemModelPackBuilder_build_zip_b64(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::ItemModelPackBuilder::build_zip_b64_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::ItemModelPackBuilder_build_zip_b64(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline const nucleation::capi::ItemModelPackBuilder* nucleation::ItemModelPackBuilder::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::ItemModelPackBuilder*>(this);
}

inline nucleation::capi::ItemModelPackBuilder* nucleation::ItemModelPackBuilder::AsFFI() {
    return reinterpret_cast<nucleation::capi::ItemModelPackBuilder*>(this);
}

inline const nucleation::ItemModelPackBuilder* nucleation::ItemModelPackBuilder::FromFFI(const nucleation::capi::ItemModelPackBuilder* ptr) {
    return reinterpret_cast<const nucleation::ItemModelPackBuilder*>(ptr);
}

inline nucleation::ItemModelPackBuilder* nucleation::ItemModelPackBuilder::FromFFI(nucleation::capi::ItemModelPackBuilder* ptr) {
    return reinterpret_cast<nucleation::ItemModelPackBuilder*>(ptr);
}

inline void nucleation::ItemModelPackBuilder::operator delete(void* ptr) {
    nucleation::capi::ItemModelPackBuilder_destroy(reinterpret_cast<nucleation::capi::ItemModelPackBuilder*>(ptr));
}


#endif // NUCLEATION_ItemModelPackBuilder_HPP
