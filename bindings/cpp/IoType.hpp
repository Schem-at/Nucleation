#ifndef IoType_HPP
#define IoType_HPP

#include "IoType.d.hpp"

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

    diplomat::capi::IoType* IoType_unsigned_int(uint32_t bits);

    diplomat::capi::IoType* IoType_signed_int(uint32_t bits);

    diplomat::capi::IoType* IoType_float32(void);

    diplomat::capi::IoType* IoType_boolean(void);

    diplomat::capi::IoType* IoType_ascii(uint32_t chars);

    void IoType_destroy(IoType* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::unique_ptr<IoType> IoType::unsigned_int(uint32_t bits) {
    auto result = diplomat::capi::IoType_unsigned_int(bits);
    return std::unique_ptr<IoType>(IoType::FromFFI(result));
}

inline std::unique_ptr<IoType> IoType::signed_int(uint32_t bits) {
    auto result = diplomat::capi::IoType_signed_int(bits);
    return std::unique_ptr<IoType>(IoType::FromFFI(result));
}

inline std::unique_ptr<IoType> IoType::float32() {
    auto result = diplomat::capi::IoType_float32();
    return std::unique_ptr<IoType>(IoType::FromFFI(result));
}

inline std::unique_ptr<IoType> IoType::boolean() {
    auto result = diplomat::capi::IoType_boolean();
    return std::unique_ptr<IoType>(IoType::FromFFI(result));
}

inline std::unique_ptr<IoType> IoType::ascii(uint32_t chars) {
    auto result = diplomat::capi::IoType_ascii(chars);
    return std::unique_ptr<IoType>(IoType::FromFFI(result));
}

inline const diplomat::capi::IoType* IoType::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::IoType*>(this);
}

inline diplomat::capi::IoType* IoType::AsFFI() {
    return reinterpret_cast<diplomat::capi::IoType*>(this);
}

inline const IoType* IoType::FromFFI(const diplomat::capi::IoType* ptr) {
    return reinterpret_cast<const IoType*>(ptr);
}

inline IoType* IoType::FromFFI(diplomat::capi::IoType* ptr) {
    return reinterpret_cast<IoType*>(ptr);
}

inline void IoType::operator delete(void* ptr) {
    diplomat::capi::IoType_destroy(reinterpret_cast<diplomat::capi::IoType*>(ptr));
}


#endif // IoType_HPP
