#ifndef FieldProgram_HPP
#define FieldProgram_HPP

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


namespace diplomat {
namespace capi {
    extern "C" {

    typedef struct FieldProgram_from_json_string_result {union {diplomat::capi::FieldProgram* ok; diplomat::capi::NucleationError err;}; bool is_ok;} FieldProgram_from_json_string_result;
    FieldProgram_from_json_string_result FieldProgram_from_json_string(diplomat::capi::DiplomatStringView json);

    typedef struct FieldProgram_to_json_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} FieldProgram_to_json_result;
    FieldProgram_to_json_result FieldProgram_to_json(const diplomat::capi::FieldProgram* self, diplomat::capi::DiplomatWrite* write);

    float FieldProgram_eval_at(const diplomat::capi::FieldProgram* self, float x, float y, float z);

    typedef struct FieldProgram_gradient_result {union {diplomat::capi::SdfNormal ok; diplomat::capi::NucleationError err;}; bool is_ok;} FieldProgram_gradient_result;
    FieldProgram_gradient_result FieldProgram_gradient(const diplomat::capi::FieldProgram* self, float x, float y, float z, float epsilon);

    diplomat::capi::SdfBounds FieldProgram_bounds(const diplomat::capi::FieldProgram* self);

    diplomat::capi::FieldProgramDistanceKind FieldProgram_distance_kind(const diplomat::capi::FieldProgram* self);

    void FieldProgram_destroy(FieldProgram* self);

    } // extern "C"
} // namespace capi
} // namespace

inline diplomat::result<std::unique_ptr<FieldProgram>, NucleationError> FieldProgram::from_json_string(std::string_view json) {
    auto result = diplomat::capi::FieldProgram_from_json_string({json.data(), json.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<FieldProgram>, NucleationError>(diplomat::Ok<std::unique_ptr<FieldProgram>>(std::unique_ptr<FieldProgram>(FieldProgram::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<FieldProgram>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> FieldProgram::to_json() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::FieldProgram_to_json(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> FieldProgram::to_json_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::FieldProgram_to_json(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline float FieldProgram::eval_at(float x, float y, float z) const {
    auto result = diplomat::capi::FieldProgram_eval_at(this->AsFFI(),
        x,
        y,
        z);
    return result;
}

inline diplomat::result<SdfNormal, NucleationError> FieldProgram::gradient(float x, float y, float z, float epsilon) const {
    auto result = diplomat::capi::FieldProgram_gradient(this->AsFFI(),
        x,
        y,
        z,
        epsilon);
    return result.is_ok ? diplomat::result<SdfNormal, NucleationError>(diplomat::Ok<SdfNormal>(SdfNormal::FromFFI(result.ok))) : diplomat::result<SdfNormal, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline SdfBounds FieldProgram::bounds() const {
    auto result = diplomat::capi::FieldProgram_bounds(this->AsFFI());
    return SdfBounds::FromFFI(result);
}

inline FieldProgramDistanceKind FieldProgram::distance_kind() const {
    auto result = diplomat::capi::FieldProgram_distance_kind(this->AsFFI());
    return FieldProgramDistanceKind::FromFFI(result);
}

inline const diplomat::capi::FieldProgram* FieldProgram::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::FieldProgram*>(this);
}

inline diplomat::capi::FieldProgram* FieldProgram::AsFFI() {
    return reinterpret_cast<diplomat::capi::FieldProgram*>(this);
}

inline const FieldProgram* FieldProgram::FromFFI(const diplomat::capi::FieldProgram* ptr) {
    return reinterpret_cast<const FieldProgram*>(ptr);
}

inline FieldProgram* FieldProgram::FromFFI(diplomat::capi::FieldProgram* ptr) {
    return reinterpret_cast<FieldProgram*>(ptr);
}

inline void FieldProgram::operator delete(void* ptr) {
    diplomat::capi::FieldProgram_destroy(reinterpret_cast<diplomat::capi::FieldProgram*>(ptr));
}


#endif // FieldProgram_HPP
