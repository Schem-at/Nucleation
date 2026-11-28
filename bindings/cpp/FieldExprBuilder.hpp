#ifndef FieldExprBuilder_HPP
#define FieldExprBuilder_HPP

#include "FieldExprBuilder.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "Field.hpp"
#include "FieldDistanceKind.hpp"
#include "NucleationError.hpp"
#include "diplomat_runtime.hpp"


namespace diplomat {
namespace capi {
    extern "C" {

    diplomat::capi::FieldExprBuilder* FieldExprBuilder_create(void);

    typedef struct FieldExprBuilder_bind_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} FieldExprBuilder_bind_result;
    FieldExprBuilder_bind_result FieldExprBuilder_bind(diplomat::capi::FieldExprBuilder* self, diplomat::capi::DiplomatStringView name, const diplomat::capi::Field* field);

    typedef struct FieldExprBuilder_set_bounds_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} FieldExprBuilder_set_bounds_result;
    FieldExprBuilder_set_bounds_result FieldExprBuilder_set_bounds(diplomat::capi::FieldExprBuilder* self, float min_x, float min_y, float min_z, float max_x, float max_y, float max_z);

    void FieldExprBuilder_set_distance_kind(diplomat::capi::FieldExprBuilder* self, diplomat::capi::FieldDistanceKind kind);

    typedef struct FieldExprBuilder_compile_result {union {diplomat::capi::Field* ok; diplomat::capi::NucleationError err;}; bool is_ok;} FieldExprBuilder_compile_result;
    FieldExprBuilder_compile_result FieldExprBuilder_compile(const diplomat::capi::FieldExprBuilder* self, diplomat::capi::DiplomatStringView source);

    void FieldExprBuilder_destroy(FieldExprBuilder* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::unique_ptr<FieldExprBuilder> FieldExprBuilder::create() {
    auto result = diplomat::capi::FieldExprBuilder_create();
    return std::unique_ptr<FieldExprBuilder>(FieldExprBuilder::FromFFI(result));
}

inline diplomat::result<std::monostate, NucleationError> FieldExprBuilder::bind(std::string_view name, const Field& field) {
    auto result = diplomat::capi::FieldExprBuilder_bind(this->AsFFI(),
        {name.data(), name.size()},
        field.AsFFI());
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> FieldExprBuilder::set_bounds(float min_x, float min_y, float min_z, float max_x, float max_y, float max_z) {
    auto result = diplomat::capi::FieldExprBuilder_set_bounds(this->AsFFI(),
        min_x,
        min_y,
        min_z,
        max_x,
        max_y,
        max_z);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline void FieldExprBuilder::set_distance_kind(FieldDistanceKind kind) {
    diplomat::capi::FieldExprBuilder_set_distance_kind(this->AsFFI(),
        kind.AsFFI());
}

inline diplomat::result<std::unique_ptr<Field>, NucleationError> FieldExprBuilder::compile(std::string_view source) const {
    auto result = diplomat::capi::FieldExprBuilder_compile(this->AsFFI(),
        {source.data(), source.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Ok<std::unique_ptr<Field>>(std::unique_ptr<Field>(Field::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline const diplomat::capi::FieldExprBuilder* FieldExprBuilder::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::FieldExprBuilder*>(this);
}

inline diplomat::capi::FieldExprBuilder* FieldExprBuilder::AsFFI() {
    return reinterpret_cast<diplomat::capi::FieldExprBuilder*>(this);
}

inline const FieldExprBuilder* FieldExprBuilder::FromFFI(const diplomat::capi::FieldExprBuilder* ptr) {
    return reinterpret_cast<const FieldExprBuilder*>(ptr);
}

inline FieldExprBuilder* FieldExprBuilder::FromFFI(diplomat::capi::FieldExprBuilder* ptr) {
    return reinterpret_cast<FieldExprBuilder*>(ptr);
}

inline void FieldExprBuilder::operator delete(void* ptr) {
    diplomat::capi::FieldExprBuilder_destroy(reinterpret_cast<diplomat::capi::FieldExprBuilder*>(ptr));
}


#endif // FieldExprBuilder_HPP
