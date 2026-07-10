#ifndef NUCLEATION_ExecutionMode_D_HPP
#define NUCLEATION_ExecutionMode_D_HPP

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
namespace capi { struct ExecutionMode; }
class ExecutionMode;
namespace capi { struct OutputCondition; }
class OutputCondition;
class NucleationError;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct ExecutionMode;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * How long to run the circuit for one `execute` call (PORTING rule 10).
 */
class ExecutionMode {
public:

  inline static std::unique_ptr<nucleation::ExecutionMode> fixed_ticks(uint32_t ticks);

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::ExecutionMode>, nucleation::NucleationError> until_condition(std::string_view output_name, const nucleation::OutputCondition& condition, uint32_t max_ticks, uint32_t check_interval);

  inline static std::unique_ptr<nucleation::ExecutionMode> until_change(uint32_t max_ticks, uint32_t check_interval);

  inline static std::unique_ptr<nucleation::ExecutionMode> until_stable(uint32_t stable_ticks, uint32_t max_ticks);

    inline const nucleation::capi::ExecutionMode* AsFFI() const;
    inline nucleation::capi::ExecutionMode* AsFFI();
    inline static const nucleation::ExecutionMode* FromFFI(const nucleation::capi::ExecutionMode* ptr);
    inline static nucleation::ExecutionMode* FromFFI(nucleation::capi::ExecutionMode* ptr);
    inline static void operator delete(void* ptr);
private:
    ExecutionMode() = delete;
    ExecutionMode(const nucleation::ExecutionMode&) = delete;
    ExecutionMode(nucleation::ExecutionMode&&) noexcept = delete;
    ExecutionMode operator=(const nucleation::ExecutionMode&) = delete;
    ExecutionMode operator=(nucleation::ExecutionMode&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_ExecutionMode_D_HPP
