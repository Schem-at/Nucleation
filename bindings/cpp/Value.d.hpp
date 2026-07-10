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

  inline static std::unique_ptr<Value> from_u32(uint32_t v);

  inline static std::unique_ptr<Value> from_i32(int32_t v);

  inline static std::unique_ptr<Value> from_f32(float v);

  inline static std::unique_ptr<Value> from_bool(bool v);

  inline static diplomat::result<std::unique_ptr<Value>, NucleationError> from_string(std::string_view s);

  inline diplomat::result<uint32_t, NucleationError> as_u32() const;

  inline diplomat::result<int32_t, NucleationError> as_i32() const;

  inline diplomat::result<float, NucleationError> as_f32() const;

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
