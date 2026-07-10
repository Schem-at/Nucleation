#ifndef ResourcePackList_HPP
#define ResourcePackList_HPP

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


namespace diplomat {
namespace capi {
    extern "C" {

    diplomat::capi::ResourcePackList* ResourcePackList_create(void);

    void ResourcePackList_add(diplomat::capi::ResourcePackList* self, diplomat::capi::DiplomatU8View data);

    uint32_t ResourcePackList_len(const diplomat::capi::ResourcePackList* self);

    void ResourcePackList_destroy(ResourcePackList* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::unique_ptr<ResourcePackList> ResourcePackList::create() {
    auto result = diplomat::capi::ResourcePackList_create();
    return std::unique_ptr<ResourcePackList>(ResourcePackList::FromFFI(result));
}

inline void ResourcePackList::add(diplomat::span<const uint8_t> data) {
    diplomat::capi::ResourcePackList_add(this->AsFFI(),
        {data.data(), data.size()});
}

inline uint32_t ResourcePackList::len() const {
    auto result = diplomat::capi::ResourcePackList_len(this->AsFFI());
    return result;
}

inline const diplomat::capi::ResourcePackList* ResourcePackList::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::ResourcePackList*>(this);
}

inline diplomat::capi::ResourcePackList* ResourcePackList::AsFFI() {
    return reinterpret_cast<diplomat::capi::ResourcePackList*>(this);
}

inline const ResourcePackList* ResourcePackList::FromFFI(const diplomat::capi::ResourcePackList* ptr) {
    return reinterpret_cast<const ResourcePackList*>(ptr);
}

inline ResourcePackList* ResourcePackList::FromFFI(diplomat::capi::ResourcePackList* ptr) {
    return reinterpret_cast<ResourcePackList*>(ptr);
}

inline void ResourcePackList::operator delete(void* ptr) {
    diplomat::capi::ResourcePackList_destroy(reinterpret_cast<diplomat::capi::ResourcePackList*>(ptr));
}


#endif // ResourcePackList_HPP
