#ifndef Field3_HPP
#define Field3_HPP

#include "Field3.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "FieldRange.hpp"
#include "NucleationError.hpp"
#include "diplomat_runtime.hpp"


namespace diplomat {
namespace capi {
    extern "C" {

    typedef struct Field3_value_noise_fbm_result {union {diplomat::capi::Field3* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field3_value_noise_fbm_result;
    Field3_value_noise_fbm_result Field3_value_noise_fbm(float frequency, int32_t seed, uint32_t octaves);

    float Field3_eval_at(const diplomat::capi::Field3* self, float x, float y, float z);

    typedef struct Field3_output_range_result {union {diplomat::capi::FieldRange ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field3_output_range_result;
    Field3_output_range_result Field3_output_range(const diplomat::capi::Field3* self);

    typedef struct Field3_from_json_string_result {union {diplomat::capi::Field3* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Field3_from_json_string_result;
    Field3_from_json_string_result Field3_from_json_string(diplomat::capi::DiplomatStringView json);

    typedef struct Field3_to_json_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Field3_to_json_result;
    Field3_to_json_result Field3_to_json(const diplomat::capi::Field3* self, diplomat::capi::DiplomatWrite* write);

    void Field3_destroy(Field3* self);

    } // extern "C"
} // namespace capi
} // namespace

inline diplomat::result<std::unique_ptr<Field3>, NucleationError> Field3::value_noise_fbm(float frequency, int32_t seed, uint32_t octaves) {
    auto result = diplomat::capi::Field3_value_noise_fbm(frequency,
        seed,
        octaves);
    return result.is_ok ? diplomat::result<std::unique_ptr<Field3>, NucleationError>(diplomat::Ok<std::unique_ptr<Field3>>(std::unique_ptr<Field3>(Field3::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field3>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline float Field3::eval_at(float x, float y, float z) const {
    auto result = diplomat::capi::Field3_eval_at(this->AsFFI(),
        x,
        y,
        z);
    return result;
}

inline diplomat::result<FieldRange, NucleationError> Field3::output_range() const {
    auto result = diplomat::capi::Field3_output_range(this->AsFFI());
    return result.is_ok ? diplomat::result<FieldRange, NucleationError>(diplomat::Ok<FieldRange>(FieldRange::FromFFI(result.ok))) : diplomat::result<FieldRange, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Field3>, NucleationError> Field3::from_json_string(std::string_view json) {
    auto result = diplomat::capi::Field3_from_json_string({json.data(), json.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<Field3>, NucleationError>(diplomat::Ok<std::unique_ptr<Field3>>(std::unique_ptr<Field3>(Field3::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Field3>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> Field3::to_json() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::Field3_to_json(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> Field3::to_json_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::Field3_to_json(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline const diplomat::capi::Field3* Field3::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::Field3*>(this);
}

inline diplomat::capi::Field3* Field3::AsFFI() {
    return reinterpret_cast<diplomat::capi::Field3*>(this);
}

inline const Field3* Field3::FromFFI(const diplomat::capi::Field3* ptr) {
    return reinterpret_cast<const Field3*>(ptr);
}

inline Field3* Field3::FromFFI(diplomat::capi::Field3* ptr) {
    return reinterpret_cast<Field3*>(ptr);
}

inline void Field3::operator delete(void* ptr) {
    diplomat::capi::Field3_destroy(reinterpret_cast<diplomat::capi::Field3*>(ptr));
}


#endif // Field3_HPP
