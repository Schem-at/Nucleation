#ifndef Value_D_HPP
#define Value_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"

class NucleationError;




namespace diplomat {
namespace capi {
    struct Value;
} // namespace capi
} // namespace

/**
 * A typed circuit value (payload-carrying enum; PORTING rule 10).
 */
class Value {
public:

  /**
   * Create an unsigned 32-bit integer value.
   */
  inline static std::unique_ptr<Value> from_u32(uint32_t v);

  /**
   * Create a signed 32-bit integer value.
   */
  inline static std::unique_ptr<Value> from_i32(int32_t v);

  /**
   * Create a 32-bit IEEE 754 float value.
   */
  inline static std::unique_ptr<Value> from_f32(float v);

  /**
   * Create a boolean value.
   */
  inline static std::unique_ptr<Value> from_bool(bool v);

  /**
   * Create a string value. Fails if the bytes are not valid UTF-8.
   */
  inline static diplomat::result<std::unique_ptr<Value>, NucleationError> from_string(std::string_view s);

  /**
   * The value as u32. Also accepts u64/non-negative signed ints that
   * fit, and bool (false → 0, true → 1); fails otherwise.
   */
  inline diplomat::result<uint32_t, NucleationError> as_u32() const;

  /**
   * The value as i32. Also accepts i64 values in i32 range; fails for
   * other types.
   */
  inline diplomat::result<int32_t, NucleationError> as_i32() const;

  /**
   * The value as f32; fails if this is not an f32 value.
   */
  inline diplomat::result<float, NucleationError> as_f32() const;

  /**
   * The value as bool; fails if this is not a bool value.
   */
  inline diplomat::result<bool, NucleationError> as_bool() const;

  /**
   * The string payload; fails if this is not a string value.
   */
  inline diplomat::result<std::string, NucleationError> as_string() const;
  template<typename W>
  inline diplomat::result<std::monostate, NucleationError> as_string_write(W& writeable_output) const;

  /**
   * The type name (e.g. "u32", "bool", "string").
   */
  inline std::string type_name() const;
  template<typename W>
  inline void type_name_write(W& writeable_output) const;

    inline const diplomat::capi::Value* AsFFI() const;
    inline diplomat::capi::Value* AsFFI();
    inline static const Value* FromFFI(const diplomat::capi::Value* ptr);
    inline static Value* FromFFI(diplomat::capi::Value* ptr);
    inline static void operator delete(void* ptr);
private:
    Value() = delete;
    Value(const Value&) = delete;
    Value(Value&&) noexcept = delete;
    Value operator=(const Value&) = delete;
    Value operator=(Value&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // Value_D_HPP
