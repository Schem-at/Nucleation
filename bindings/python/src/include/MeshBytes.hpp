#ifndef NUCLEATION_MeshBytes_HPP
#define NUCLEATION_MeshBytes_HPP

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


namespace nucleation {
namespace capi {
    extern "C" {

    uint32_t MeshBytes_len(const nucleation::capi::MeshBytes* self);

    uint32_t MeshBytes_copy_into(const nucleation::capi::MeshBytes* self, uint32_t offset, nucleation::diplomat::capi::DiplomatU8ViewMut out);

    void MeshBytes_destroy(MeshBytes* self);

    } // extern "C"
} // namespace capi
} // namespace

inline uint32_t nucleation::MeshBytes::len() const {
    auto result = nucleation::capi::MeshBytes_len(this->AsFFI());
    return result;
}

inline uint32_t nucleation::MeshBytes::copy_into(uint32_t offset, nucleation::diplomat::span<uint8_t> out) const {
    auto result = nucleation::capi::MeshBytes_copy_into(this->AsFFI(),
        offset,
        {out.data(), out.size()});
    return result;
}

inline const nucleation::capi::MeshBytes* nucleation::MeshBytes::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::MeshBytes*>(this);
}

inline nucleation::capi::MeshBytes* nucleation::MeshBytes::AsFFI() {
    return reinterpret_cast<nucleation::capi::MeshBytes*>(this);
}

inline const nucleation::MeshBytes* nucleation::MeshBytes::FromFFI(const nucleation::capi::MeshBytes* ptr) {
    return reinterpret_cast<const nucleation::MeshBytes*>(ptr);
}

inline nucleation::MeshBytes* nucleation::MeshBytes::FromFFI(nucleation::capi::MeshBytes* ptr) {
    return reinterpret_cast<nucleation::MeshBytes*>(ptr);
}

inline void nucleation::MeshBytes::operator delete(void* ptr) {
    nucleation::capi::MeshBytes_destroy(reinterpret_cast<nucleation::capi::MeshBytes*>(ptr));
}


#endif // NUCLEATION_MeshBytes_HPP
