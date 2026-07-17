#ifndef NUCLEATION_OutputCondition_D_HPP
#define NUCLEATION_OutputCondition_D_HPP

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
namespace capi { struct OutputCondition; }
class OutputCondition;
namespace capi { struct Value; }
class Value;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct OutputCondition;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * A condition on an output value, for `ExecutionMode::until_condition`
 * (PORTING rule 10).
 */
class OutputCondition {
public:

  /**
   * Met when the output equals `value`.
   */
  inline static std::unique_ptr<nucleation::OutputCondition> equals(const nucleation::Value& value);

  /**
   * Met when the output does not equal `value`.
   */
  inline static std::unique_ptr<nucleation::OutputCondition> not_equals(const nucleation::Value& value);

  /**
   * Met when the output is greater than `value`. Numeric only: both
   * sides must be the same numeric type (u32/i32/f32), else never met.
   */
  inline static std::unique_ptr<nucleation::OutputCondition> greater_than(const nucleation::Value& value);

  /**
   * Met when the output is less than `value`. Numeric only: both sides
   * must be the same numeric type (u32/i32/f32), else never met.
   */
  inline static std::unique_ptr<nucleation::OutputCondition> less_than(const nucleation::Value& value);

  /**
   * Met when `output & mask` is non-zero (flag checking). Integer
   * outputs (u32/i32) only; never met for other types.
   */
  inline static std::unique_ptr<nucleation::OutputCondition> bitwise_and(uint32_t mask);

    inline const nucleation::capi::OutputCondition* AsFFI() const;
    inline nucleation::capi::OutputCondition* AsFFI();
    inline static const nucleation::OutputCondition* FromFFI(const nucleation::capi::OutputCondition* ptr);
    inline static nucleation::OutputCondition* FromFFI(nucleation::capi::OutputCondition* ptr);
    inline static void operator delete(void* ptr);
private:
    OutputCondition() = delete;
    OutputCondition(const nucleation::OutputCondition&) = delete;
    OutputCondition(nucleation::OutputCondition&&) noexcept = delete;
    OutputCondition operator=(const nucleation::OutputCondition&) = delete;
    OutputCondition operator=(nucleation::OutputCondition&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_OutputCondition_D_HPP
