#ifndef Value_HPP
#define Value_HPP

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


namespace diplomat {
namespace capi {
    extern "C" {

    diplomat::capi::Value* Value_from_u32(uint32_t v);

    diplomat::capi::Value* Value_from_i32(int32_t v);

    diplomat::capi::Value* Value_from_f32(float v);

    diplomat::capi::Value* Value_from_bool(bool v);

    typedef struct Value_from_string_result {union {diplomat::capi::Value* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Value_from_string_result;
    Value_from_string_result Value_from_string(diplomat::capi::DiplomatStringView s);

    typedef struct Value_as_u32_result {union {uint32_t ok; diplomat::capi::NucleationError err;}; bool is_ok;} Value_as_u32_result;
    Value_as_u32_result Value_as_u32(const diplomat::capi::Value* self);

    typedef struct Value_as_i32_result {union {int32_t ok; diplomat::capi::NucleationError err;}; bool is_ok;} Value_as_i32_result;
    Value_as_i32_result Value_as_i32(const diplomat::capi::Value* self);

    typedef struct Value_as_f32_result {union {float ok; diplomat::capi::NucleationError err;}; bool is_ok;} Value_as_f32_result;
    Value_as_f32_result Value_as_f32(const diplomat::capi::Value* self);

    typedef struct Value_as_bool_result {union {bool ok; diplomat::capi::NucleationError err;}; bool is_ok;} Value_as_bool_result;
    Value_as_bool_result Value_as_bool(const diplomat::capi::Value* self);

    typedef struct Value_as_string_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Value_as_string_result;
    Value_as_string_result Value_as_string(const diplomat::capi::Value* self, diplomat::capi::DiplomatWrite* write);

    void Value_type_name(const diplomat::capi::Value* self, diplomat::capi::DiplomatWrite* write);

    void Value_destroy(Value* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::unique_ptr<Value> Value::from_u32(uint32_t v) {
    auto result = diplomat::capi::Value_from_u32(v);
    return std::unique_ptr<Value>(Value::FromFFI(result));
}

inline std::unique_ptr<Value> Value::from_i32(int32_t v) {
    auto result = diplomat::capi::Value_from_i32(v);
    return std::unique_ptr<Value>(Value::FromFFI(result));
}

inline std::unique_ptr<Value> Value::from_f32(float v) {
    auto result = diplomat::capi::Value_from_f32(v);
    return std::unique_ptr<Value>(Value::FromFFI(result));
}

inline std::unique_ptr<Value> Value::from_bool(bool v) {
    auto result = diplomat::capi::Value_from_bool(v);
    return std::unique_ptr<Value>(Value::FromFFI(result));
}

inline diplomat::result<std::unique_ptr<Value>, NucleationError> Value::from_string(std::string_view s) {
    auto result = diplomat::capi::Value_from_string({s.data(), s.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<Value>, NucleationError>(diplomat::Ok<std::unique_ptr<Value>>(std::unique_ptr<Value>(Value::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Value>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<uint32_t, NucleationError> Value::as_u32() const {
    auto result = diplomat::capi::Value_as_u32(this->AsFFI());
    return result.is_ok ? diplomat::result<uint32_t, NucleationError>(diplomat::Ok<uint32_t>(result.ok)) : diplomat::result<uint32_t, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<int32_t, NucleationError> Value::as_i32() const {
    auto result = diplomat::capi::Value_as_i32(this->AsFFI());
    return result.is_ok ? diplomat::result<int32_t, NucleationError>(diplomat::Ok<int32_t>(result.ok)) : diplomat::result<int32_t, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<float, NucleationError> Value::as_f32() const {
    auto result = diplomat::capi::Value_as_f32(this->AsFFI());
    return result.is_ok ? diplomat::result<float, NucleationError>(diplomat::Ok<float>(result.ok)) : diplomat::result<float, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<bool, NucleationError> Value::as_bool() const {
    auto result = diplomat::capi::Value_as_bool(this->AsFFI());
    return result.is_ok ? diplomat::result<bool, NucleationError>(diplomat::Ok<bool>(result.ok)) : diplomat::result<bool, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> Value::as_string() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::Value_as_string(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> Value::as_string_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::Value_as_string(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline std::string Value::type_name() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::Value_type_name(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void Value::type_name_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::Value_type_name(this->AsFFI(),
        &write);
}

inline const diplomat::capi::Value* Value::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::Value*>(this);
}

inline diplomat::capi::Value* Value::AsFFI() {
    return reinterpret_cast<diplomat::capi::Value*>(this);
}

inline const Value* Value::FromFFI(const diplomat::capi::Value* ptr) {
    return reinterpret_cast<const Value*>(ptr);
}

inline Value* Value::FromFFI(diplomat::capi::Value* ptr) {
    return reinterpret_cast<Value*>(ptr);
}

inline void Value::operator delete(void* ptr) {
    diplomat::capi::Value_destroy(reinterpret_cast<diplomat::capi::Value*>(ptr));
}


#endif // Value_HPP
