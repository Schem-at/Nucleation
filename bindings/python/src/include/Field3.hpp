#ifndef NUCLEATION_Field3_HPP
#define NUCLEATION_Field3_HPP

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


namespace nucleation {
namespace capi {
    extern "C" {

    typedef struct Field3_value_noise_fbm_result {union {nucleation::capi::Field3* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field3_value_noise_fbm_result;
    Field3_value_noise_fbm_result Field3_value_noise_fbm(float frequency, int32_t seed, uint32_t octaves);

    float Field3_eval_at(const nucleation::capi::Field3* self, float x, float y, float z);

    typedef struct Field3_output_range_result {union {nucleation::capi::FieldRange ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field3_output_range_result;
    Field3_output_range_result Field3_output_range(const nucleation::capi::Field3* self);

    typedef struct Field3_from_json_string_result {union {nucleation::capi::Field3* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Field3_from_json_string_result;
    Field3_from_json_string_result Field3_from_json_string(nucleation::diplomat::capi::DiplomatStringView json);

    typedef struct Field3_to_json_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Field3_to_json_result;
    Field3_to_json_result Field3_to_json(const nucleation::capi::Field3* self, nucleation::diplomat::capi::DiplomatWrite* write);

    void Field3_destroy(Field3* self);

    } // extern "C"
} // namespace capi
} // namespace

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field3>, nucleation::NucleationError> nucleation::Field3::value_noise_fbm(float frequency, int32_t seed, uint32_t octaves) {
    auto result = nucleation::capi::Field3_value_noise_fbm(frequency,
        seed,
        octaves);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field3>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field3>>(std::unique_ptr<nucleation::Field3>(nucleation::Field3::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field3>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline float nucleation::Field3::eval_at(float x, float y, float z) const {
    auto result = nucleation::capi::Field3_eval_at(this->AsFFI(),
        x,
        y,
        z);
    return result;
}

inline nucleation::diplomat::result<nucleation::FieldRange, nucleation::NucleationError> nucleation::Field3::output_range() const {
    auto result = nucleation::capi::Field3_output_range(this->AsFFI());
    return result.is_ok ? nucleation::diplomat::result<nucleation::FieldRange, nucleation::NucleationError>(nucleation::diplomat::Ok<nucleation::FieldRange>(nucleation::FieldRange::FromFFI(result.ok))) : nucleation::diplomat::result<nucleation::FieldRange, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Field3>, nucleation::NucleationError> nucleation::Field3::from_json_string(std::string_view json) {
    auto result = nucleation::capi::Field3_from_json_string({json.data(), json.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Field3>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Field3>>(std::unique_ptr<nucleation::Field3>(nucleation::Field3::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Field3>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::Field3::to_json() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::Field3_to_json(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Field3::to_json_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::Field3_to_json(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline const nucleation::capi::Field3* nucleation::Field3::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::Field3*>(this);
}

inline nucleation::capi::Field3* nucleation::Field3::AsFFI() {
    return reinterpret_cast<nucleation::capi::Field3*>(this);
}

inline const nucleation::Field3* nucleation::Field3::FromFFI(const nucleation::capi::Field3* ptr) {
    return reinterpret_cast<const nucleation::Field3*>(ptr);
}

inline nucleation::Field3* nucleation::Field3::FromFFI(nucleation::capi::Field3* ptr) {
    return reinterpret_cast<nucleation::Field3*>(ptr);
}

inline void nucleation::Field3::operator delete(void* ptr) {
    nucleation::capi::Field3_destroy(reinterpret_cast<nucleation::capi::Field3*>(ptr));
}


#endif // NUCLEATION_Field3_HPP
