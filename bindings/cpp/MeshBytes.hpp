#ifndef MeshBytes_HPP
#define MeshBytes_HPP

#include "MeshBytes.d.hpp"

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

    uint32_t MeshBytes_len(const diplomat::capi::MeshBytes* self);

    uint32_t MeshBytes_copy_into(const diplomat::capi::MeshBytes* self, uint32_t offset, diplomat::capi::DiplomatU8ViewMut out);

    void MeshBytes_destroy(MeshBytes* self);

    } // extern "C"
} // namespace capi
} // namespace

inline uint32_t MeshBytes::len() const {
    auto result = diplomat::capi::MeshBytes_len(this->AsFFI());
    return result;
}

inline uint32_t MeshBytes::copy_into(uint32_t offset, diplomat::span<uint8_t> out) const {
    auto result = diplomat::capi::MeshBytes_copy_into(this->AsFFI(),
        offset,
        {out.data(), out.size()});
    return result;
}

inline const diplomat::capi::MeshBytes* MeshBytes::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::MeshBytes*>(this);
}

inline diplomat::capi::MeshBytes* MeshBytes::AsFFI() {
    return reinterpret_cast<diplomat::capi::MeshBytes*>(this);
}

inline const MeshBytes* MeshBytes::FromFFI(const diplomat::capi::MeshBytes* ptr) {
    return reinterpret_cast<const MeshBytes*>(ptr);
}

inline MeshBytes* MeshBytes::FromFFI(diplomat::capi::MeshBytes* ptr) {
    return reinterpret_cast<MeshBytes*>(ptr);
}

inline void MeshBytes::operator delete(void* ptr) {
    diplomat::capi::MeshBytes_destroy(reinterpret_cast<diplomat::capi::MeshBytes*>(ptr));
}


#endif // MeshBytes_HPP
