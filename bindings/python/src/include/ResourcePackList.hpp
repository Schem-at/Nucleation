#ifndef NUCLEATION_ResourcePackList_HPP
#define NUCLEATION_ResourcePackList_HPP

#include "ResourcePackList.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"


namespace nucleation {
namespace capi {
    extern "C" {

    nucleation::capi::ResourcePackList* ResourcePackList_create(void);

    void ResourcePackList_add(nucleation::capi::ResourcePackList* self, nucleation::diplomat::capi::DiplomatU8View data);

    uint32_t ResourcePackList_len(const nucleation::capi::ResourcePackList* self);

    void ResourcePackList_destroy(ResourcePackList* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::unique_ptr<nucleation::ResourcePackList> nucleation::ResourcePackList::create() {
    auto result = nucleation::capi::ResourcePackList_create();
    return std::unique_ptr<nucleation::ResourcePackList>(nucleation::ResourcePackList::FromFFI(result));
}

inline void nucleation::ResourcePackList::add(nucleation::diplomat::span<const uint8_t> data) {
    nucleation::capi::ResourcePackList_add(this->AsFFI(),
        {data.data(), data.size()});
}

inline uint32_t nucleation::ResourcePackList::len() const {
    auto result = nucleation::capi::ResourcePackList_len(this->AsFFI());
    return result;
}

inline const nucleation::capi::ResourcePackList* nucleation::ResourcePackList::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::ResourcePackList*>(this);
}

inline nucleation::capi::ResourcePackList* nucleation::ResourcePackList::AsFFI() {
    return reinterpret_cast<nucleation::capi::ResourcePackList*>(this);
}

inline const nucleation::ResourcePackList* nucleation::ResourcePackList::FromFFI(const nucleation::capi::ResourcePackList* ptr) {
    return reinterpret_cast<const nucleation::ResourcePackList*>(ptr);
}

inline nucleation::ResourcePackList* nucleation::ResourcePackList::FromFFI(nucleation::capi::ResourcePackList* ptr) {
    return reinterpret_cast<nucleation::ResourcePackList*>(ptr);
}

inline void nucleation::ResourcePackList::operator delete(void* ptr) {
    nucleation::capi::ResourcePackList_destroy(reinterpret_cast<nucleation::capi::ResourcePackList*>(ptr));
}


#endif // NUCLEATION_ResourcePackList_HPP
