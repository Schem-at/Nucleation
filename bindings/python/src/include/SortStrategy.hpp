#ifndef NUCLEATION_SortStrategy_HPP
#define NUCLEATION_SortStrategy_HPP

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


namespace nucleation {
namespace capi {
    extern "C" {

    nucleation::capi::SortStrategy* SortStrategy_yxz(void);

    nucleation::capi::SortStrategy* SortStrategy_xyz(void);

    nucleation::capi::SortStrategy* SortStrategy_zyx(void);

    nucleation::capi::SortStrategy* SortStrategy_y_desc_xz(void);

    nucleation::capi::SortStrategy* SortStrategy_x_desc_yz(void);

    nucleation::capi::SortStrategy* SortStrategy_z_desc_yx(void);

    nucleation::capi::SortStrategy* SortStrategy_descending(void);

    nucleation::capi::SortStrategy* SortStrategy_distance_from(int32_t x, int32_t y, int32_t z);

    nucleation::capi::SortStrategy* SortStrategy_distance_from_desc(int32_t x, int32_t y, int32_t z);

    nucleation::capi::SortStrategy* SortStrategy_preserve(void);

    nucleation::capi::SortStrategy* SortStrategy_reverse(void);

    typedef struct SortStrategy_from_string_result {union {nucleation::capi::SortStrategy* ok; nucleation::capi::NucleationError err;}; bool is_ok;} SortStrategy_from_string_result;
    SortStrategy_from_string_result SortStrategy_from_string(nucleation::diplomat::capi::DiplomatStringView s);

    void SortStrategy_name(const nucleation::capi::SortStrategy* self, nucleation::diplomat::capi::DiplomatWrite* write);

    void SortStrategy_destroy(SortStrategy* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::unique_ptr<nucleation::SortStrategy> nucleation::SortStrategy::yxz() {
    auto result = nucleation::capi::SortStrategy_yxz();
    return std::unique_ptr<nucleation::SortStrategy>(nucleation::SortStrategy::FromFFI(result));
}

inline std::unique_ptr<nucleation::SortStrategy> nucleation::SortStrategy::xyz() {
    auto result = nucleation::capi::SortStrategy_xyz();
    return std::unique_ptr<nucleation::SortStrategy>(nucleation::SortStrategy::FromFFI(result));
}

inline std::unique_ptr<nucleation::SortStrategy> nucleation::SortStrategy::zyx() {
    auto result = nucleation::capi::SortStrategy_zyx();
    return std::unique_ptr<nucleation::SortStrategy>(nucleation::SortStrategy::FromFFI(result));
}

inline std::unique_ptr<nucleation::SortStrategy> nucleation::SortStrategy::y_desc_xz() {
    auto result = nucleation::capi::SortStrategy_y_desc_xz();
    return std::unique_ptr<nucleation::SortStrategy>(nucleation::SortStrategy::FromFFI(result));
}

inline std::unique_ptr<nucleation::SortStrategy> nucleation::SortStrategy::x_desc_yz() {
    auto result = nucleation::capi::SortStrategy_x_desc_yz();
    return std::unique_ptr<nucleation::SortStrategy>(nucleation::SortStrategy::FromFFI(result));
}

inline std::unique_ptr<nucleation::SortStrategy> nucleation::SortStrategy::z_desc_yx() {
    auto result = nucleation::capi::SortStrategy_z_desc_yx();
    return std::unique_ptr<nucleation::SortStrategy>(nucleation::SortStrategy::FromFFI(result));
}

inline std::unique_ptr<nucleation::SortStrategy> nucleation::SortStrategy::descending() {
    auto result = nucleation::capi::SortStrategy_descending();
    return std::unique_ptr<nucleation::SortStrategy>(nucleation::SortStrategy::FromFFI(result));
}

inline std::unique_ptr<nucleation::SortStrategy> nucleation::SortStrategy::distance_from(int32_t x, int32_t y, int32_t z) {
    auto result = nucleation::capi::SortStrategy_distance_from(x,
        y,
        z);
    return std::unique_ptr<nucleation::SortStrategy>(nucleation::SortStrategy::FromFFI(result));
}

inline std::unique_ptr<nucleation::SortStrategy> nucleation::SortStrategy::distance_from_desc(int32_t x, int32_t y, int32_t z) {
    auto result = nucleation::capi::SortStrategy_distance_from_desc(x,
        y,
        z);
    return std::unique_ptr<nucleation::SortStrategy>(nucleation::SortStrategy::FromFFI(result));
}

inline std::unique_ptr<nucleation::SortStrategy> nucleation::SortStrategy::preserve() {
    auto result = nucleation::capi::SortStrategy_preserve();
    return std::unique_ptr<nucleation::SortStrategy>(nucleation::SortStrategy::FromFFI(result));
}

inline std::unique_ptr<nucleation::SortStrategy> nucleation::SortStrategy::reverse() {
    auto result = nucleation::capi::SortStrategy_reverse();
    return std::unique_ptr<nucleation::SortStrategy>(nucleation::SortStrategy::FromFFI(result));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::SortStrategy>, nucleation::NucleationError> nucleation::SortStrategy::from_string(std::string_view s) {
    auto result = nucleation::capi::SortStrategy_from_string({s.data(), s.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::SortStrategy>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::SortStrategy>>(std::unique_ptr<nucleation::SortStrategy>(nucleation::SortStrategy::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::SortStrategy>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline std::string nucleation::SortStrategy::name() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::SortStrategy_name(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void nucleation::SortStrategy::name_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::SortStrategy_name(this->AsFFI(),
        &write);
}

inline const nucleation::capi::SortStrategy* nucleation::SortStrategy::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::SortStrategy*>(this);
}

inline nucleation::capi::SortStrategy* nucleation::SortStrategy::AsFFI() {
    return reinterpret_cast<nucleation::capi::SortStrategy*>(this);
}

inline const nucleation::SortStrategy* nucleation::SortStrategy::FromFFI(const nucleation::capi::SortStrategy* ptr) {
    return reinterpret_cast<const nucleation::SortStrategy*>(ptr);
}

inline nucleation::SortStrategy* nucleation::SortStrategy::FromFFI(nucleation::capi::SortStrategy* ptr) {
    return reinterpret_cast<nucleation::SortStrategy*>(ptr);
}

inline void nucleation::SortStrategy::operator delete(void* ptr) {
    nucleation::capi::SortStrategy_destroy(reinterpret_cast<nucleation::capi::SortStrategy*>(ptr));
}


#endif // NUCLEATION_SortStrategy_HPP
