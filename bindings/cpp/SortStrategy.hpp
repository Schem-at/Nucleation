#ifndef SortStrategy_HPP
#define SortStrategy_HPP

#include "SortStrategy.d.hpp"

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

    diplomat::capi::SortStrategy* SortStrategy_yxz(void);

    diplomat::capi::SortStrategy* SortStrategy_xyz(void);

    diplomat::capi::SortStrategy* SortStrategy_zyx(void);

    diplomat::capi::SortStrategy* SortStrategy_y_desc_xz(void);

    diplomat::capi::SortStrategy* SortStrategy_x_desc_yz(void);

    diplomat::capi::SortStrategy* SortStrategy_z_desc_yx(void);

    diplomat::capi::SortStrategy* SortStrategy_descending(void);

    diplomat::capi::SortStrategy* SortStrategy_distance_from(int32_t x, int32_t y, int32_t z);

    diplomat::capi::SortStrategy* SortStrategy_distance_from_desc(int32_t x, int32_t y, int32_t z);

    diplomat::capi::SortStrategy* SortStrategy_preserve(void);

    diplomat::capi::SortStrategy* SortStrategy_reverse(void);

    typedef struct SortStrategy_from_string_result {union {diplomat::capi::SortStrategy* ok; diplomat::capi::NucleationError err;}; bool is_ok;} SortStrategy_from_string_result;
    SortStrategy_from_string_result SortStrategy_from_string(diplomat::capi::DiplomatStringView s);

    void SortStrategy_name(const diplomat::capi::SortStrategy* self, diplomat::capi::DiplomatWrite* write);

    void SortStrategy_destroy(SortStrategy* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::unique_ptr<SortStrategy> SortStrategy::yxz() {
    auto result = diplomat::capi::SortStrategy_yxz();
    return std::unique_ptr<SortStrategy>(SortStrategy::FromFFI(result));
}

inline std::unique_ptr<SortStrategy> SortStrategy::xyz() {
    auto result = diplomat::capi::SortStrategy_xyz();
    return std::unique_ptr<SortStrategy>(SortStrategy::FromFFI(result));
}

inline std::unique_ptr<SortStrategy> SortStrategy::zyx() {
    auto result = diplomat::capi::SortStrategy_zyx();
    return std::unique_ptr<SortStrategy>(SortStrategy::FromFFI(result));
}

inline std::unique_ptr<SortStrategy> SortStrategy::y_desc_xz() {
    auto result = diplomat::capi::SortStrategy_y_desc_xz();
    return std::unique_ptr<SortStrategy>(SortStrategy::FromFFI(result));
}

inline std::unique_ptr<SortStrategy> SortStrategy::x_desc_yz() {
    auto result = diplomat::capi::SortStrategy_x_desc_yz();
    return std::unique_ptr<SortStrategy>(SortStrategy::FromFFI(result));
}

inline std::unique_ptr<SortStrategy> SortStrategy::z_desc_yx() {
    auto result = diplomat::capi::SortStrategy_z_desc_yx();
    return std::unique_ptr<SortStrategy>(SortStrategy::FromFFI(result));
}

inline std::unique_ptr<SortStrategy> SortStrategy::descending() {
    auto result = diplomat::capi::SortStrategy_descending();
    return std::unique_ptr<SortStrategy>(SortStrategy::FromFFI(result));
}

inline std::unique_ptr<SortStrategy> SortStrategy::distance_from(int32_t x, int32_t y, int32_t z) {
    auto result = diplomat::capi::SortStrategy_distance_from(x,
        y,
        z);
    return std::unique_ptr<SortStrategy>(SortStrategy::FromFFI(result));
}

inline std::unique_ptr<SortStrategy> SortStrategy::distance_from_desc(int32_t x, int32_t y, int32_t z) {
    auto result = diplomat::capi::SortStrategy_distance_from_desc(x,
        y,
        z);
    return std::unique_ptr<SortStrategy>(SortStrategy::FromFFI(result));
}

inline std::unique_ptr<SortStrategy> SortStrategy::preserve() {
    auto result = diplomat::capi::SortStrategy_preserve();
    return std::unique_ptr<SortStrategy>(SortStrategy::FromFFI(result));
}

inline std::unique_ptr<SortStrategy> SortStrategy::reverse() {
    auto result = diplomat::capi::SortStrategy_reverse();
    return std::unique_ptr<SortStrategy>(SortStrategy::FromFFI(result));
}

inline diplomat::result<std::unique_ptr<SortStrategy>, NucleationError> SortStrategy::from_string(std::string_view s) {
    auto result = diplomat::capi::SortStrategy_from_string({s.data(), s.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<SortStrategy>, NucleationError>(diplomat::Ok<std::unique_ptr<SortStrategy>>(std::unique_ptr<SortStrategy>(SortStrategy::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<SortStrategy>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline std::string SortStrategy::name() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::SortStrategy_name(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void SortStrategy::name_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::SortStrategy_name(this->AsFFI(),
        &write);
}

inline const diplomat::capi::SortStrategy* SortStrategy::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::SortStrategy*>(this);
}

inline diplomat::capi::SortStrategy* SortStrategy::AsFFI() {
    return reinterpret_cast<diplomat::capi::SortStrategy*>(this);
}

inline const SortStrategy* SortStrategy::FromFFI(const diplomat::capi::SortStrategy* ptr) {
    return reinterpret_cast<const SortStrategy*>(ptr);
}

inline SortStrategy* SortStrategy::FromFFI(diplomat::capi::SortStrategy* ptr) {
    return reinterpret_cast<SortStrategy*>(ptr);
}

inline void SortStrategy::operator delete(void* ptr) {
    diplomat::capi::SortStrategy_destroy(reinterpret_cast<diplomat::capi::SortStrategy*>(ptr));
}


#endif // SortStrategy_HPP
