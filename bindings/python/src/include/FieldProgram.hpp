#ifndef NUCLEATION_FieldProgram_HPP
#define NUCLEATION_FieldProgram_HPP

#include "FieldProgram.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "FieldProgramDistanceKind.hpp"
#include "NucleationError.hpp"
#include "SdfBounds.hpp"
#include "SdfNormal.hpp"
#include "diplomat_runtime.hpp"


namespace nucleation {
namespace capi {
    extern "C" {

    typedef struct FieldProgram_from_json_string_result {union {nucleation::capi::FieldProgram* ok; nucleation::capi::NucleationError err;}; bool is_ok;} FieldProgram_from_json_string_result;
    FieldProgram_from_json_string_result FieldProgram_from_json_string(nucleation::diplomat::capi::DiplomatStringView json);

    typedef struct FieldProgram_to_json_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} FieldProgram_to_json_result;
    FieldProgram_to_json_result FieldProgram_to_json(const nucleation::capi::FieldProgram* self, nucleation::diplomat::capi::DiplomatWrite* write);

    float FieldProgram_eval_at(const nucleation::capi::FieldProgram* self, float x, float y, float z);

    typedef struct FieldProgram_gradient_result {union {nucleation::capi::SdfNormal ok; nucleation::capi::NucleationError err;}; bool is_ok;} FieldProgram_gradient_result;
    FieldProgram_gradient_result FieldProgram_gradient(const nucleation::capi::FieldProgram* self, float x, float y, float z, float epsilon);

    nucleation::capi::SdfBounds FieldProgram_bounds(const nucleation::capi::FieldProgram* self);

    nucleation::capi::FieldProgramDistanceKind FieldProgram_distance_kind(const nucleation::capi::FieldProgram* self);

    void FieldProgram_destroy(FieldProgram* self);

    } // extern "C"
} // namespace capi
} // namespace

inline nucleation::diplomat::result<std::unique_ptr<nucleation::FieldProgram>, nucleation::NucleationError> nucleation::FieldProgram::from_json_string(std::string_view json) {
    auto result = nucleation::capi::FieldProgram_from_json_string({json.data(), json.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::FieldProgram>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::FieldProgram>>(std::unique_ptr<nucleation::FieldProgram>(nucleation::FieldProgram::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::FieldProgram>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::FieldProgram::to_json() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::FieldProgram_to_json(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::FieldProgram::to_json_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::FieldProgram_to_json(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline float nucleation::FieldProgram::eval_at(float x, float y, float z) const {
    auto result = nucleation::capi::FieldProgram_eval_at(this->AsFFI(),
        x,
        y,
        z);
    return result;
}

inline nucleation::diplomat::result<nucleation::SdfNormal, nucleation::NucleationError> nucleation::FieldProgram::gradient(float x, float y, float z, float epsilon) const {
    auto result = nucleation::capi::FieldProgram_gradient(this->AsFFI(),
        x,
        y,
        z,
        epsilon);
    return result.is_ok ? nucleation::diplomat::result<nucleation::SdfNormal, nucleation::NucleationError>(nucleation::diplomat::Ok<nucleation::SdfNormal>(nucleation::SdfNormal::FromFFI(result.ok))) : nucleation::diplomat::result<nucleation::SdfNormal, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::SdfBounds nucleation::FieldProgram::bounds() const {
    auto result = nucleation::capi::FieldProgram_bounds(this->AsFFI());
    return nucleation::SdfBounds::FromFFI(result);
}

inline nucleation::FieldProgramDistanceKind nucleation::FieldProgram::distance_kind() const {
    auto result = nucleation::capi::FieldProgram_distance_kind(this->AsFFI());
    return nucleation::FieldProgramDistanceKind::FromFFI(result);
}

inline const nucleation::capi::FieldProgram* nucleation::FieldProgram::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::FieldProgram*>(this);
}

inline nucleation::capi::FieldProgram* nucleation::FieldProgram::AsFFI() {
    return reinterpret_cast<nucleation::capi::FieldProgram*>(this);
}

inline const nucleation::FieldProgram* nucleation::FieldProgram::FromFFI(const nucleation::capi::FieldProgram* ptr) {
    return reinterpret_cast<const nucleation::FieldProgram*>(ptr);
}

inline nucleation::FieldProgram* nucleation::FieldProgram::FromFFI(nucleation::capi::FieldProgram* ptr) {
    return reinterpret_cast<nucleation::FieldProgram*>(ptr);
}

inline void nucleation::FieldProgram::operator delete(void* ptr) {
    nucleation::capi::FieldProgram_destroy(reinterpret_cast<nucleation::capi::FieldProgram*>(ptr));
}


#endif // NUCLEATION_FieldProgram_HPP
