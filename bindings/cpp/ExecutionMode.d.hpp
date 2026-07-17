#ifndef ExecutionMode_D_HPP
#define ExecutionMode_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"

namespace diplomat::capi { struct OutputCondition; }
class OutputCondition;
class NucleationError;




namespace diplomat {
namespace capi {
    struct ExecutionMode;
} // namespace capi
} // namespace

/**
 * How long to run the circuit for one `execute` call (PORTING rule 10).
 */
class ExecutionMode {
public:

  /**
   * Run for exactly `ticks` ticks.
   */
  inline static std::unique_ptr<ExecutionMode> fixed_ticks(uint32_t ticks);

  /**
   * Run until the output named `output_name` meets `condition`,
   * checking every `check_interval` ticks, giving up after `max_ticks`
   * ticks (the result's `condition_met` reports which happened).
   */
  inline static diplomat::result<std::unique_ptr<ExecutionMode>, NucleationError> until_condition(std::string_view output_name, const OutputCondition& condition, uint32_t max_ticks, uint32_t check_interval);

  /**
   * Run until any output changes from its initial reading, checking
   * every `check_interval` ticks, giving up after `max_ticks` ticks.
   */
  inline static std::unique_ptr<ExecutionMode> until_change(uint32_t max_ticks, uint32_t check_interval);

  /**
   * Run (one tick at a time) until all outputs have been unchanged for
   * `stable_ticks` consecutive ticks, giving up after `max_ticks`
   * ticks (the result's `condition_met` reports stability).
   */
  inline static std::unique_ptr<ExecutionMode> until_stable(uint32_t stable_ticks, uint32_t max_ticks);

    inline const diplomat::capi::ExecutionMode* AsFFI() const;
    inline diplomat::capi::ExecutionMode* AsFFI();
    inline static const ExecutionMode* FromFFI(const diplomat::capi::ExecutionMode* ptr);
    inline static ExecutionMode* FromFFI(diplomat::capi::ExecutionMode* ptr);
    inline static void operator delete(void* ptr);
private:
    ExecutionMode() = delete;
    ExecutionMode(const ExecutionMode&) = delete;
    ExecutionMode(ExecutionMode&&) noexcept = delete;
    ExecutionMode operator=(const ExecutionMode&) = delete;
    ExecutionMode operator=(ExecutionMode&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // ExecutionMode_D_HPP
