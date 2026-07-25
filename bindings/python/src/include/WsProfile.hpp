#ifndef NUCLEATION_WsProfile_HPP
#define NUCLEATION_WsProfile_HPP

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


namespace nucleation {
namespace capi {
    extern "C" {

    typedef struct WsProfile_derive_from_dir_result {union {nucleation::capi::WsProfile* ok; nucleation::capi::NucleationError err;}; bool is_ok;} WsProfile_derive_from_dir_result;
    WsProfile_derive_from_dir_result WsProfile_derive_from_dir(nucleation::diplomat::capi::DiplomatStringView world_dir, int32_t min_y, int32_t max_y, uint32_t sample, float coverage);

    int32_t WsProfile_band_min(const nucleation::capi::WsProfile* self);

    int32_t WsProfile_band_max(const nucleation::capi::WsProfile* self);

    uint32_t WsProfile_palette_len(const nucleation::capi::WsProfile* self);

    typedef struct WsProfile_write_palette_json_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} WsProfile_write_palette_json_result;
    WsProfile_write_palette_json_result WsProfile_write_palette_json(const nucleation::capi::WsProfile* self, nucleation::diplomat::capi::DiplomatWrite* write);

    void WsProfile_destroy(WsProfile* self);

    } // extern "C"
} // namespace capi
} // namespace

inline nucleation::diplomat::result<std::unique_ptr<nucleation::WsProfile>, nucleation::NucleationError> nucleation::WsProfile::derive_from_dir(std::string_view world_dir, int32_t min_y, int32_t max_y, uint32_t sample, float coverage) {
    auto result = nucleation::capi::WsProfile_derive_from_dir({world_dir.data(), world_dir.size()},
        min_y,
        max_y,
        sample,
        coverage);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::WsProfile>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::WsProfile>>(std::unique_ptr<nucleation::WsProfile>(nucleation::WsProfile::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::WsProfile>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline int32_t nucleation::WsProfile::band_min() const {
    auto result = nucleation::capi::WsProfile_band_min(this->AsFFI());
    return result;
}

inline int32_t nucleation::WsProfile::band_max() const {
    auto result = nucleation::capi::WsProfile_band_max(this->AsFFI());
    return result;
}

inline uint32_t nucleation::WsProfile::palette_len() const {
    auto result = nucleation::capi::WsProfile_palette_len(this->AsFFI());
    return result;
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::WsProfile::write_palette_json() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::WsProfile_write_palette_json(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::WsProfile::write_palette_json_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::WsProfile_write_palette_json(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline const nucleation::capi::WsProfile* nucleation::WsProfile::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::WsProfile*>(this);
}

inline nucleation::capi::WsProfile* nucleation::WsProfile::AsFFI() {
    return reinterpret_cast<nucleation::capi::WsProfile*>(this);
}

inline const nucleation::WsProfile* nucleation::WsProfile::FromFFI(const nucleation::capi::WsProfile* ptr) {
    return reinterpret_cast<const nucleation::WsProfile*>(ptr);
}

inline nucleation::WsProfile* nucleation::WsProfile::FromFFI(nucleation::capi::WsProfile* ptr) {
    return reinterpret_cast<nucleation::WsProfile*>(ptr);
}

inline void nucleation::WsProfile::operator delete(void* ptr) {
    nucleation::capi::WsProfile_destroy(reinterpret_cast<nucleation::capi::WsProfile*>(ptr));
}


#endif // NUCLEATION_WsProfile_HPP
