#ifndef NUCLEATION_Value_D_HPP
#define NUCLEATION_Value_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"
namespace nucleation {
namespace capi { struct Value; }
class Value;
class NucleationError;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct Value;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * A typed circuit value (payload-carrying enum; PORTING rule 10).
 */
class Value {
public:

  /**
   * Create an unsigned 32-bit integer value.
   */
  inline static std::unique_ptr<nucleation::Value> from_u32(uint32_t v);

  /**
   * Create a signed 32-bit integer value.
   */
  inline static std::unique_ptr<nucleation::Value> from_i32(int32_t v);

  /**
   * Create a 32-bit IEEE 754 float value.
   */
  inline static std::unique_ptr<nucleation::Value> from_f32(float v);

  /**
   * Create a boolean value.
   */
  inline static std::unique_ptr<nucleation::Value> from_bool(bool v);

  /**
   * Create a string value. Fails if the bytes are not valid UTF-8.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Value>, nucleation::NucleationError> from_string(std::string_view s);

  /**
   * The value as u32. Also accepts u64/non-negative signed ints that
   * fit, and bool (false → 0, true → 1); fails otherwise.
   */
  inline nucleation::diplomat::result<uint32_t, nucleation::NucleationError> as_u32() const;

  /**
   * The value as i32. Also accepts i64 values in i32 range; fails for
   * other types.
   */
  inline nucleation::diplomat::result<int32_t, nucleation::NucleationError> as_i32() const;

  /**
   * The value as f32; fails if this is not an f32 value.
   */
  inline nucleation::diplomat::result<float, nucleation::NucleationError> as_f32() const;

  /**
   * The value as bool; fails if this is not a bool value.
   */
  inline nucleation::diplomat::result<bool, nucleation::NucleationError> as_bool() const;

  /**
   * The string payload; fails if this is not a string value.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> as_string() const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> as_string_write(W& writeable_output) const;

  /**
   * The type name (e.g. "u32", "bool", "string").
   */
  inline std::string type_name() const;
  template<typename W>
  inline void type_name_write(W& writeable_output) const;

    inline const nucleation::capi::Value* AsFFI() const;
    inline nucleation::capi::Value* AsFFI();
    inline static const nucleation::Value* FromFFI(const nucleation::capi::Value* ptr);
    inline static nucleation::Value* FromFFI(nucleation::capi::Value* ptr);
    inline static void operator delete(void* ptr);
private:
    Value() = delete;
    Value(const nucleation::Value&) = delete;
    Value(nucleation::Value&&) noexcept = delete;
    Value operator=(const nucleation::Value&) = delete;
    Value operator=(nucleation::Value&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_Value_D_HPP
