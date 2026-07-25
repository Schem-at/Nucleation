#ifndef NUCLEATION_WsPartitionHints_HPP
#define NUCLEATION_WsPartitionHints_HPP

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


namespace nucleation {
namespace capi {
    extern "C" {

    nucleation::capi::WsPartitionHints* WsPartitionHints_create(void);

    typedef struct WsPartitionHints_add_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} WsPartitionHints_add_result;
    WsPartitionHints_add_result WsPartitionHints_add(nucleation::capi::WsPartitionHints* self, nucleation::diplomat::capi::DiplomatStringView id, int32_t x0, int32_t x1, int32_t z0, int32_t z1);

    uint32_t WsPartitionHints_len(const nucleation::capi::WsPartitionHints* self);

    void WsPartitionHints_destroy(WsPartitionHints* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::unique_ptr<nucleation::WsPartitionHints> nucleation::WsPartitionHints::create() {
    auto result = nucleation::capi::WsPartitionHints_create();
    return std::unique_ptr<nucleation::WsPartitionHints>(nucleation::WsPartitionHints::FromFFI(result));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::WsPartitionHints::add(std::string_view id, int32_t x0, int32_t x1, int32_t z0, int32_t z1) {
    auto result = nucleation::capi::WsPartitionHints_add(this->AsFFI(),
        {id.data(), id.size()},
        x0,
        x1,
        z0,
        z1);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline uint32_t nucleation::WsPartitionHints::len() const {
    auto result = nucleation::capi::WsPartitionHints_len(this->AsFFI());
    return result;
}

inline const nucleation::capi::WsPartitionHints* nucleation::WsPartitionHints::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::WsPartitionHints*>(this);
}

inline nucleation::capi::WsPartitionHints* nucleation::WsPartitionHints::AsFFI() {
    return reinterpret_cast<nucleation::capi::WsPartitionHints*>(this);
}

inline const nucleation::WsPartitionHints* nucleation::WsPartitionHints::FromFFI(const nucleation::capi::WsPartitionHints* ptr) {
    return reinterpret_cast<const nucleation::WsPartitionHints*>(ptr);
}

inline nucleation::WsPartitionHints* nucleation::WsPartitionHints::FromFFI(nucleation::capi::WsPartitionHints* ptr) {
    return reinterpret_cast<nucleation::WsPartitionHints*>(ptr);
}

inline void nucleation::WsPartitionHints::operator delete(void* ptr) {
    nucleation::capi::WsPartitionHints_destroy(reinterpret_cast<nucleation::capi::WsPartitionHints*>(ptr));
}


#endif // NUCLEATION_WsPartitionHints_HPP
