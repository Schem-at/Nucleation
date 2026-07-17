#ifndef OutputCondition_D_HPP
#define OutputCondition_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"

namespace diplomat::capi { struct Value; }
class Value;




namespace diplomat {
namespace capi {
    struct OutputCondition;
} // namespace capi
} // namespace

/**
 * A condition on an output value, for `ExecutionMode::until_condition`
 * (PORTING rule 10).
 */
class OutputCondition {
public:

  /**
   * Met when the output equals `value`.
   */
  inline static std::unique_ptr<OutputCondition> equals(const Value& value);

  /**
   * Met when the output does not equal `value`.
   */
  inline static std::unique_ptr<OutputCondition> not_equals(const Value& value);

  /**
   * Met when the output is greater than `value`. Numeric only: both
   * sides must be the same numeric type (u32/i32/f32), else never met.
   */
  inline static std::unique_ptr<OutputCondition> greater_than(const Value& value);

  /**
   * Met when the output is less than `value`. Numeric only: both sides
   * must be the same numeric type (u32/i32/f32), else never met.
   */
  inline static std::unique_ptr<OutputCondition> less_than(const Value& value);

  /**
   * Met when `output & mask` is non-zero (flag checking). Integer
   * outputs (u32/i32) only; never met for other types.
   */
  inline static std::unique_ptr<OutputCondition> bitwise_and(uint32_t mask);

    inline const diplomat::capi::OutputCondition* AsFFI() const;
    inline diplomat::capi::OutputCondition* AsFFI();
    inline static const OutputCondition* FromFFI(const diplomat::capi::OutputCondition* ptr);
    inline static OutputCondition* FromFFI(diplomat::capi::OutputCondition* ptr);
    inline static void operator delete(void* ptr);
private:
    OutputCondition() = delete;
    OutputCondition(const OutputCondition&) = delete;
    OutputCondition(OutputCondition&&) noexcept = delete;
    OutputCondition operator=(const OutputCondition&) = delete;
    OutputCondition operator=(OutputCondition&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // OutputCondition_D_HPP
