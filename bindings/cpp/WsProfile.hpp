#ifndef WsProfile_HPP
#define WsProfile_HPP

#include "WsProfile.d.hpp"

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

    typedef struct WsProfile_derive_from_dir_result {union {diplomat::capi::WsProfile* ok; diplomat::capi::NucleationError err;}; bool is_ok;} WsProfile_derive_from_dir_result;
    WsProfile_derive_from_dir_result WsProfile_derive_from_dir(diplomat::capi::DiplomatStringView world_dir, int32_t min_y, int32_t max_y, uint32_t sample, float coverage);

    int32_t WsProfile_band_min(const diplomat::capi::WsProfile* self);

    int32_t WsProfile_band_max(const diplomat::capi::WsProfile* self);

    uint32_t WsProfile_palette_len(const diplomat::capi::WsProfile* self);

    typedef struct WsProfile_write_palette_json_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} WsProfile_write_palette_json_result;
    WsProfile_write_palette_json_result WsProfile_write_palette_json(const diplomat::capi::WsProfile* self, diplomat::capi::DiplomatWrite* write);

    void WsProfile_destroy(WsProfile* self);

    } // extern "C"
} // namespace capi
} // namespace

inline diplomat::result<std::unique_ptr<WsProfile>, NucleationError> WsProfile::derive_from_dir(std::string_view world_dir, int32_t min_y, int32_t max_y, uint32_t sample, float coverage) {
    auto result = diplomat::capi::WsProfile_derive_from_dir({world_dir.data(), world_dir.size()},
        min_y,
        max_y,
        sample,
        coverage);
    return result.is_ok ? diplomat::result<std::unique_ptr<WsProfile>, NucleationError>(diplomat::Ok<std::unique_ptr<WsProfile>>(std::unique_ptr<WsProfile>(WsProfile::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<WsProfile>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline int32_t WsProfile::band_min() const {
    auto result = diplomat::capi::WsProfile_band_min(this->AsFFI());
    return result;
}

inline int32_t WsProfile::band_max() const {
    auto result = diplomat::capi::WsProfile_band_max(this->AsFFI());
    return result;
}

inline uint32_t WsProfile::palette_len() const {
    auto result = diplomat::capi::WsProfile_palette_len(this->AsFFI());
    return result;
}

inline diplomat::result<std::string, NucleationError> WsProfile::write_palette_json() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::WsProfile_write_palette_json(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> WsProfile::write_palette_json_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::WsProfile_write_palette_json(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline const diplomat::capi::WsProfile* WsProfile::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::WsProfile*>(this);
}

inline diplomat::capi::WsProfile* WsProfile::AsFFI() {
    return reinterpret_cast<diplomat::capi::WsProfile*>(this);
}

inline const WsProfile* WsProfile::FromFFI(const diplomat::capi::WsProfile* ptr) {
    return reinterpret_cast<const WsProfile*>(ptr);
}

inline WsProfile* WsProfile::FromFFI(diplomat::capi::WsProfile* ptr) {
    return reinterpret_cast<WsProfile*>(ptr);
}

inline void WsProfile::operator delete(void* ptr) {
    diplomat::capi::WsProfile_destroy(reinterpret_cast<diplomat::capi::WsProfile*>(ptr));
}


#endif // WsProfile_HPP
