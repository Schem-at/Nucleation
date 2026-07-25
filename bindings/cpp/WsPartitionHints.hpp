#ifndef WsPartitionHints_HPP
#define WsPartitionHints_HPP

#include "WsPartitionHints.d.hpp"

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

    diplomat::capi::WsPartitionHints* WsPartitionHints_create(void);

    typedef struct WsPartitionHints_add_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} WsPartitionHints_add_result;
    WsPartitionHints_add_result WsPartitionHints_add(diplomat::capi::WsPartitionHints* self, diplomat::capi::DiplomatStringView id, int32_t x0, int32_t x1, int32_t z0, int32_t z1);

    uint32_t WsPartitionHints_len(const diplomat::capi::WsPartitionHints* self);

    void WsPartitionHints_destroy(WsPartitionHints* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::unique_ptr<WsPartitionHints> WsPartitionHints::create() {
    auto result = diplomat::capi::WsPartitionHints_create();
    return std::unique_ptr<WsPartitionHints>(WsPartitionHints::FromFFI(result));
}

inline diplomat::result<std::monostate, NucleationError> WsPartitionHints::add(std::string_view id, int32_t x0, int32_t x1, int32_t z0, int32_t z1) {
    auto result = diplomat::capi::WsPartitionHints_add(this->AsFFI(),
        {id.data(), id.size()},
        x0,
        x1,
        z0,
        z1);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline uint32_t WsPartitionHints::len() const {
    auto result = diplomat::capi::WsPartitionHints_len(this->AsFFI());
    return result;
}

inline const diplomat::capi::WsPartitionHints* WsPartitionHints::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::WsPartitionHints*>(this);
}

inline diplomat::capi::WsPartitionHints* WsPartitionHints::AsFFI() {
    return reinterpret_cast<diplomat::capi::WsPartitionHints*>(this);
}

inline const WsPartitionHints* WsPartitionHints::FromFFI(const diplomat::capi::WsPartitionHints* ptr) {
    return reinterpret_cast<const WsPartitionHints*>(ptr);
}

inline WsPartitionHints* WsPartitionHints::FromFFI(diplomat::capi::WsPartitionHints* ptr) {
    return reinterpret_cast<WsPartitionHints*>(ptr);
}

inline void WsPartitionHints::operator delete(void* ptr) {
    diplomat::capi::WsPartitionHints_destroy(reinterpret_cast<diplomat::capi::WsPartitionHints*>(ptr));
}


#endif // WsPartitionHints_HPP
