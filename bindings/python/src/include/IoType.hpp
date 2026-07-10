#ifndef NUCLEATION_IoType_HPP
#define NUCLEATION_IoType_HPP

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


namespace nucleation {
namespace capi {
    extern "C" {

    nucleation::capi::IoType* IoType_unsigned_int(uint32_t bits);

    nucleation::capi::IoType* IoType_signed_int(uint32_t bits);

    nucleation::capi::IoType* IoType_float32(void);

    nucleation::capi::IoType* IoType_boolean(void);

    nucleation::capi::IoType* IoType_ascii(uint32_t chars);

    void IoType_destroy(IoType* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::unique_ptr<nucleation::IoType> nucleation::IoType::unsigned_int(uint32_t bits) {
    auto result = nucleation::capi::IoType_unsigned_int(bits);
    return std::unique_ptr<nucleation::IoType>(nucleation::IoType::FromFFI(result));
}

inline std::unique_ptr<nucleation::IoType> nucleation::IoType::signed_int(uint32_t bits) {
    auto result = nucleation::capi::IoType_signed_int(bits);
    return std::unique_ptr<nucleation::IoType>(nucleation::IoType::FromFFI(result));
}

inline std::unique_ptr<nucleation::IoType> nucleation::IoType::float32() {
    auto result = nucleation::capi::IoType_float32();
    return std::unique_ptr<nucleation::IoType>(nucleation::IoType::FromFFI(result));
}

inline std::unique_ptr<nucleation::IoType> nucleation::IoType::boolean() {
    auto result = nucleation::capi::IoType_boolean();
    return std::unique_ptr<nucleation::IoType>(nucleation::IoType::FromFFI(result));
}

inline std::unique_ptr<nucleation::IoType> nucleation::IoType::ascii(uint32_t chars) {
    auto result = nucleation::capi::IoType_ascii(chars);
    return std::unique_ptr<nucleation::IoType>(nucleation::IoType::FromFFI(result));
}

inline const nucleation::capi::IoType* nucleation::IoType::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::IoType*>(this);
}

inline nucleation::capi::IoType* nucleation::IoType::AsFFI() {
    return reinterpret_cast<nucleation::capi::IoType*>(this);
}

inline const nucleation::IoType* nucleation::IoType::FromFFI(const nucleation::capi::IoType* ptr) {
    return reinterpret_cast<const nucleation::IoType*>(ptr);
}

inline nucleation::IoType* nucleation::IoType::FromFFI(nucleation::capi::IoType* ptr) {
    return reinterpret_cast<nucleation::IoType*>(ptr);
}

inline void nucleation::IoType::operator delete(void* ptr) {
    nucleation::capi::IoType_destroy(reinterpret_cast<nucleation::capi::IoType*>(ptr));
}


#endif // NUCLEATION_IoType_HPP
