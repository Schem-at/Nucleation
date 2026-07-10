#ifndef NUCLEATION_Value_HPP
#define NUCLEATION_Value_HPP

#include "Value.d.hpp"

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

    nucleation::capi::Value* Value_from_u32(uint32_t v);

    nucleation::capi::Value* Value_from_i32(int32_t v);

    nucleation::capi::Value* Value_from_f32(float v);

    nucleation::capi::Value* Value_from_bool(bool v);

    typedef struct Value_from_string_result {union {nucleation::capi::Value* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Value_from_string_result;
    Value_from_string_result Value_from_string(nucleation::diplomat::capi::DiplomatStringView s);

    typedef struct Value_as_u32_result {union {uint32_t ok; nucleation::capi::NucleationError err;}; bool is_ok;} Value_as_u32_result;
    Value_as_u32_result Value_as_u32(const nucleation::capi::Value* self);

    typedef struct Value_as_i32_result {union {int32_t ok; nucleation::capi::NucleationError err;}; bool is_ok;} Value_as_i32_result;
    Value_as_i32_result Value_as_i32(const nucleation::capi::Value* self);

    typedef struct Value_as_f32_result {union {float ok; nucleation::capi::NucleationError err;}; bool is_ok;} Value_as_f32_result;
    Value_as_f32_result Value_as_f32(const nucleation::capi::Value* self);

    typedef struct Value_as_bool_result {union {bool ok; nucleation::capi::NucleationError err;}; bool is_ok;} Value_as_bool_result;
    Value_as_bool_result Value_as_bool(const nucleation::capi::Value* self);

    typedef struct Value_as_string_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Value_as_string_result;
    Value_as_string_result Value_as_string(const nucleation::capi::Value* self, nucleation::diplomat::capi::DiplomatWrite* write);

    void Value_type_name(const nucleation::capi::Value* self, nucleation::diplomat::capi::DiplomatWrite* write);

    void Value_destroy(Value* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::unique_ptr<nucleation::Value> nucleation::Value::from_u32(uint32_t v) {
    auto result = nucleation::capi::Value_from_u32(v);
    return std::unique_ptr<nucleation::Value>(nucleation::Value::FromFFI(result));
}

inline std::unique_ptr<nucleation::Value> nucleation::Value::from_i32(int32_t v) {
    auto result = nucleation::capi::Value_from_i32(v);
    return std::unique_ptr<nucleation::Value>(nucleation::Value::FromFFI(result));
}

inline std::unique_ptr<nucleation::Value> nucleation::Value::from_f32(float v) {
    auto result = nucleation::capi::Value_from_f32(v);
    return std::unique_ptr<nucleation::Value>(nucleation::Value::FromFFI(result));
}

inline std::unique_ptr<nucleation::Value> nucleation::Value::from_bool(bool v) {
    auto result = nucleation::capi::Value_from_bool(v);
    return std::unique_ptr<nucleation::Value>(nucleation::Value::FromFFI(result));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Value>, nucleation::NucleationError> nucleation::Value::from_string(std::string_view s) {
    auto result = nucleation::capi::Value_from_string({s.data(), s.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Value>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Value>>(std::unique_ptr<nucleation::Value>(nucleation::Value::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Value>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<uint32_t, nucleation::NucleationError> nucleation::Value::as_u32() const {
    auto result = nucleation::capi::Value_as_u32(this->AsFFI());
    return result.is_ok ? nucleation::diplomat::result<uint32_t, nucleation::NucleationError>(nucleation::diplomat::Ok<uint32_t>(result.ok)) : nucleation::diplomat::result<uint32_t, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<int32_t, nucleation::NucleationError> nucleation::Value::as_i32() const {
    auto result = nucleation::capi::Value_as_i32(this->AsFFI());
    return result.is_ok ? nucleation::diplomat::result<int32_t, nucleation::NucleationError>(nucleation::diplomat::Ok<int32_t>(result.ok)) : nucleation::diplomat::result<int32_t, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<float, nucleation::NucleationError> nucleation::Value::as_f32() const {
    auto result = nucleation::capi::Value_as_f32(this->AsFFI());
    return result.is_ok ? nucleation::diplomat::result<float, nucleation::NucleationError>(nucleation::diplomat::Ok<float>(result.ok)) : nucleation::diplomat::result<float, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<bool, nucleation::NucleationError> nucleation::Value::as_bool() const {
    auto result = nucleation::capi::Value_as_bool(this->AsFFI());
    return result.is_ok ? nucleation::diplomat::result<bool, nucleation::NucleationError>(nucleation::diplomat::Ok<bool>(result.ok)) : nucleation::diplomat::result<bool, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::Value::as_string() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::Value_as_string(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Value::as_string_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::Value_as_string(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline std::string nucleation::Value::type_name() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::Value_type_name(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void nucleation::Value::type_name_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::Value_type_name(this->AsFFI(),
        &write);
}

inline const nucleation::capi::Value* nucleation::Value::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::Value*>(this);
}

inline nucleation::capi::Value* nucleation::Value::AsFFI() {
    return reinterpret_cast<nucleation::capi::Value*>(this);
}

inline const nucleation::Value* nucleation::Value::FromFFI(const nucleation::capi::Value* ptr) {
    return reinterpret_cast<const nucleation::Value*>(ptr);
}

inline nucleation::Value* nucleation::Value::FromFFI(nucleation::capi::Value* ptr) {
    return reinterpret_cast<nucleation::Value*>(ptr);
}

inline void nucleation::Value::operator delete(void* ptr) {
    nucleation::capi::Value_destroy(reinterpret_cast<nucleation::capi::Value*>(ptr));
}


#endif // NUCLEATION_Value_HPP
