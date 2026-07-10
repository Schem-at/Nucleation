#ifndef NUCLEATION_LayoutFunction_HPP
#define NUCLEATION_LayoutFunction_HPP

#include "LayoutFunction.d.hpp"

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

    nucleation::capi::LayoutFunction* LayoutFunction_one_to_one(void);

    nucleation::capi::LayoutFunction* LayoutFunction_packed4(void);

    typedef struct LayoutFunction_custom_result {union {nucleation::capi::LayoutFunction* ok; nucleation::capi::NucleationError err;}; bool is_ok;} LayoutFunction_custom_result;
    LayoutFunction_custom_result LayoutFunction_custom(nucleation::diplomat::capi::DiplomatU32View mapping);

    nucleation::capi::LayoutFunction* LayoutFunction_row_major(uint32_t rows, uint32_t cols, uint32_t bits_per_element);

    nucleation::capi::LayoutFunction* LayoutFunction_column_major(uint32_t rows, uint32_t cols, uint32_t bits_per_element);

    nucleation::capi::LayoutFunction* LayoutFunction_scanline(uint32_t width, uint32_t height, uint32_t bits_per_pixel);

    void LayoutFunction_destroy(LayoutFunction* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::unique_ptr<nucleation::LayoutFunction> nucleation::LayoutFunction::one_to_one() {
    auto result = nucleation::capi::LayoutFunction_one_to_one();
    return std::unique_ptr<nucleation::LayoutFunction>(nucleation::LayoutFunction::FromFFI(result));
}

inline std::unique_ptr<nucleation::LayoutFunction> nucleation::LayoutFunction::packed4() {
    auto result = nucleation::capi::LayoutFunction_packed4();
    return std::unique_ptr<nucleation::LayoutFunction>(nucleation::LayoutFunction::FromFFI(result));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::LayoutFunction>, nucleation::NucleationError> nucleation::LayoutFunction::custom(nucleation::diplomat::span<const uint32_t> mapping) {
    auto result = nucleation::capi::LayoutFunction_custom({mapping.data(), mapping.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::LayoutFunction>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::LayoutFunction>>(std::unique_ptr<nucleation::LayoutFunction>(nucleation::LayoutFunction::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::LayoutFunction>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline std::unique_ptr<nucleation::LayoutFunction> nucleation::LayoutFunction::row_major(uint32_t rows, uint32_t cols, uint32_t bits_per_element) {
    auto result = nucleation::capi::LayoutFunction_row_major(rows,
        cols,
        bits_per_element);
    return std::unique_ptr<nucleation::LayoutFunction>(nucleation::LayoutFunction::FromFFI(result));
}

inline std::unique_ptr<nucleation::LayoutFunction> nucleation::LayoutFunction::column_major(uint32_t rows, uint32_t cols, uint32_t bits_per_element) {
    auto result = nucleation::capi::LayoutFunction_column_major(rows,
        cols,
        bits_per_element);
    return std::unique_ptr<nucleation::LayoutFunction>(nucleation::LayoutFunction::FromFFI(result));
}

inline std::unique_ptr<nucleation::LayoutFunction> nucleation::LayoutFunction::scanline(uint32_t width, uint32_t height, uint32_t bits_per_pixel) {
    auto result = nucleation::capi::LayoutFunction_scanline(width,
        height,
        bits_per_pixel);
    return std::unique_ptr<nucleation::LayoutFunction>(nucleation::LayoutFunction::FromFFI(result));
}

inline const nucleation::capi::LayoutFunction* nucleation::LayoutFunction::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::LayoutFunction*>(this);
}

inline nucleation::capi::LayoutFunction* nucleation::LayoutFunction::AsFFI() {
    return reinterpret_cast<nucleation::capi::LayoutFunction*>(this);
}

inline const nucleation::LayoutFunction* nucleation::LayoutFunction::FromFFI(const nucleation::capi::LayoutFunction* ptr) {
    return reinterpret_cast<const nucleation::LayoutFunction*>(ptr);
}

inline nucleation::LayoutFunction* nucleation::LayoutFunction::FromFFI(nucleation::capi::LayoutFunction* ptr) {
    return reinterpret_cast<nucleation::LayoutFunction*>(ptr);
}

inline void nucleation::LayoutFunction::operator delete(void* ptr) {
    nucleation::capi::LayoutFunction_destroy(reinterpret_cast<nucleation::capi::LayoutFunction*>(ptr));
}


#endif // NUCLEATION_LayoutFunction_HPP
