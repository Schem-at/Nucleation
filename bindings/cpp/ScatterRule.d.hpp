#ifndef ScatterRule_D_HPP
#define ScatterRule_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"

namespace diplomat::capi { struct MaterialCondition; }
class MaterialCondition;
class NucleationError;
class ScatterFace;




namespace diplomat {
namespace capi {
    struct ScatterRule;
} // namespace capi
} // namespace

/**
 * Surface scatter rule attached to a {@link VoxelPipeline}.
 */
class ScatterRule {
public:

  inline static diplomat::result<std::unique_ptr<ScatterRule>, NucleationError> create(ScatterFace face, float density, std::string_view blocks_json, diplomat::span<const float> weights, int32_t seed);

  inline void set_condition(const MaterialCondition& condition);

  inline diplomat::result<std::monostate, NucleationError> set_on_block(std::string_view prefix);

  inline diplomat::result<std::monostate, NucleationError> set_height_range(uint32_t min, uint32_t max);

    inline const diplomat::capi::ScatterRule* AsFFI() const;
    inline diplomat::capi::ScatterRule* AsFFI();
    inline static const ScatterRule* FromFFI(const diplomat::capi::ScatterRule* ptr);
    inline static ScatterRule* FromFFI(diplomat::capi::ScatterRule* ptr);
    inline static void operator delete(void* ptr);
private:
    ScatterRule() = delete;
    ScatterRule(const ScatterRule&) = delete;
    ScatterRule(ScatterRule&&) noexcept = delete;
    ScatterRule operator=(const ScatterRule&) = delete;
    ScatterRule operator=(ScatterRule&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // ScatterRule_D_HPP
