#ifndef NUCLEATION_FieldExprBuilder_HPP
#define NUCLEATION_FieldExprBuilder_HPP

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


namespace nucleation {
namespace capi {
    extern "C" {

    nucleation::capi::FieldExprBuilder* FieldExprBuilder_create(void);

    typedef struct FieldExprBuilder_bind_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} FieldExprBuilder_bind_result;
    FieldExprBuilder_bind_result FieldExprBuilder_bind(nucleation::capi::FieldExprBuilder* self, nucleation::diplomat::capi::DiplomatStringView name, const nucleation::capi::Field* field);

    typedef struct FieldExprBuilder_set_bounds_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} FieldExprBuilder_set_bounds_result;
    FieldExprBuilder_set_bounds_result FieldExprBuilder_set_bounds(nucleation::capi::FieldExprBuilder* self, float min_x, float min_y, float min_z, float max_x, float max_y, float max_z);

    void FieldExprBuilder_set_distance_kind(nucleation::capi::FieldExprBuilder* self, nucleation::capi::FieldDistanceKind kind);

    typedef struct FieldExprBuilder_compile_result {union {nucleation::capi::Field* ok; nucleation::capi::NucleationError err;}; bool is_ok;} FieldExprBuilder_compile_result;
    FieldExprBuilder_compile_result FieldExprBuilder_compile(const nucleation::capi::FieldExprBuilder* self, nucleation::diplomat::capi::DiplomatStringView source);

    void FieldExprBuilder_destroy(FieldExprBuilder* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::unique_ptr<nucleation::FieldExprBuilder> nucleation::FieldExprBuilder::create() {
    auto result = nucleation::capi::FieldExprBuilder_create();
    return std::unique_ptr<nucleation::FieldExprBuilder>(nucleation::FieldExprBuilder::FromFFI(result));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::FieldExprBuilder::bind(std::string_view name, const nucleation::Field& field) {
    auto result = nucleation::capi::FieldExprBuilder_bind(this->AsFFI(),
        {name.data(), name.size()},
        field.AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::FieldExprBuilder::set_bounds(float min_x, float min_y, float min_z, float max_x, float max_y, float max_z) {
    auto result = nucleation::capi::FieldExprBuilder_set_bounds(this->AsFFI(),
        min_x,
        min_y,
        min_z,
        max_x,
        max_y,
        max_z);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline void nucleation::FieldExprBuilder::set_distance_kind(nucleation::FieldDistanceKind kind) {
    nucleation::capi::FieldExprBuilder_set_distance_kind(this->AsFFI(),
        kind.AsFFI());
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError> nucleation::FieldExprBuilder::compile(std::string_view source) const {
    auto result = nucleation::capi::FieldExprBuilder_compile(this->AsFFI(),
        {source.data(), source.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field>>(std::unique_ptr<nucleation::Field>(nucleation::Field::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline const nucleation::capi::FieldExprBuilder* nucleation::FieldExprBuilder::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::FieldExprBuilder*>(this);
}

inline nucleation::capi::FieldExprBuilder* nucleation::FieldExprBuilder::AsFFI() {
    return reinterpret_cast<nucleation::capi::FieldExprBuilder*>(this);
}

inline const nucleation::FieldExprBuilder* nucleation::FieldExprBuilder::FromFFI(const nucleation::capi::FieldExprBuilder* ptr) {
    return reinterpret_cast<const nucleation::FieldExprBuilder*>(ptr);
}

inline nucleation::FieldExprBuilder* nucleation::FieldExprBuilder::FromFFI(nucleation::capi::FieldExprBuilder* ptr) {
    return reinterpret_cast<nucleation::FieldExprBuilder*>(ptr);
}

inline void nucleation::FieldExprBuilder::operator delete(void* ptr) {
    nucleation::capi::FieldExprBuilder_destroy(reinterpret_cast<nucleation::capi::FieldExprBuilder*>(ptr));
}


#endif // NUCLEATION_FieldExprBuilder_HPP
