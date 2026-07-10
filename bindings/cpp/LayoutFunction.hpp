#ifndef LayoutFunction_HPP
#define LayoutFunction_HPP

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


namespace diplomat {
namespace capi {
    extern "C" {

    diplomat::capi::LayoutFunction* LayoutFunction_one_to_one(void);

    diplomat::capi::LayoutFunction* LayoutFunction_packed4(void);

    typedef struct LayoutFunction_custom_result {union {diplomat::capi::LayoutFunction* ok; diplomat::capi::NucleationError err;}; bool is_ok;} LayoutFunction_custom_result;
    LayoutFunction_custom_result LayoutFunction_custom(diplomat::capi::DiplomatU32View mapping);

    diplomat::capi::LayoutFunction* LayoutFunction_row_major(uint32_t rows, uint32_t cols, uint32_t bits_per_element);

    diplomat::capi::LayoutFunction* LayoutFunction_column_major(uint32_t rows, uint32_t cols, uint32_t bits_per_element);

    diplomat::capi::LayoutFunction* LayoutFunction_scanline(uint32_t width, uint32_t height, uint32_t bits_per_pixel);

    void LayoutFunction_destroy(LayoutFunction* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::unique_ptr<LayoutFunction> LayoutFunction::one_to_one() {
    auto result = diplomat::capi::LayoutFunction_one_to_one();
    return std::unique_ptr<LayoutFunction>(LayoutFunction::FromFFI(result));
}

inline std::unique_ptr<LayoutFunction> LayoutFunction::packed4() {
    auto result = diplomat::capi::LayoutFunction_packed4();
    return std::unique_ptr<LayoutFunction>(LayoutFunction::FromFFI(result));
}

inline diplomat::result<std::unique_ptr<LayoutFunction>, NucleationError> LayoutFunction::custom(diplomat::span<const uint32_t> mapping) {
    auto result = diplomat::capi::LayoutFunction_custom({mapping.data(), mapping.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<LayoutFunction>, NucleationError>(diplomat::Ok<std::unique_ptr<LayoutFunction>>(std::unique_ptr<LayoutFunction>(LayoutFunction::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<LayoutFunction>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline std::unique_ptr<LayoutFunction> LayoutFunction::row_major(uint32_t rows, uint32_t cols, uint32_t bits_per_element) {
    auto result = diplomat::capi::LayoutFunction_row_major(rows,
        cols,
        bits_per_element);
    return std::unique_ptr<LayoutFunction>(LayoutFunction::FromFFI(result));
}

inline std::unique_ptr<LayoutFunction> LayoutFunction::column_major(uint32_t rows, uint32_t cols, uint32_t bits_per_element) {
    auto result = diplomat::capi::LayoutFunction_column_major(rows,
        cols,
        bits_per_element);
    return std::unique_ptr<LayoutFunction>(LayoutFunction::FromFFI(result));
}

inline std::unique_ptr<LayoutFunction> LayoutFunction::scanline(uint32_t width, uint32_t height, uint32_t bits_per_pixel) {
    auto result = diplomat::capi::LayoutFunction_scanline(width,
        height,
        bits_per_pixel);
    return std::unique_ptr<LayoutFunction>(LayoutFunction::FromFFI(result));
}

inline const diplomat::capi::LayoutFunction* LayoutFunction::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::LayoutFunction*>(this);
}

inline diplomat::capi::LayoutFunction* LayoutFunction::AsFFI() {
    return reinterpret_cast<diplomat::capi::LayoutFunction*>(this);
}

inline const LayoutFunction* LayoutFunction::FromFFI(const diplomat::capi::LayoutFunction* ptr) {
    return reinterpret_cast<const LayoutFunction*>(ptr);
}

inline LayoutFunction* LayoutFunction::FromFFI(diplomat::capi::LayoutFunction* ptr) {
    return reinterpret_cast<LayoutFunction*>(ptr);
}

inline void LayoutFunction::operator delete(void* ptr) {
    diplomat::capi::LayoutFunction_destroy(reinterpret_cast<diplomat::capi::LayoutFunction*>(ptr));
}


#endif // LayoutFunction_HPP
