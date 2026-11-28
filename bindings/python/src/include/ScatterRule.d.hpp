#ifndef NUCLEATION_ScatterRule_D_HPP
#define NUCLEATION_ScatterRule_D_HPP

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
namespace capi { struct MaterialCondition; }
class MaterialCondition;
namespace capi { struct ScatterRule; }
class ScatterRule;
class NucleationError;
class ScatterFace;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct ScatterRule;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * Surface scatter rule attached to a {@link VoxelPipeline}.
 */
class ScatterRule {
public:

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::ScatterRule>, nucleation::NucleationError> create(nucleation::ScatterFace face, float density, std::string_view blocks_json, nucleation::diplomat::span<const float> weights, int32_t seed);

  inline void set_condition(const nucleation::MaterialCondition& condition);

  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> set_on_block(std::string_view prefix);

  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> set_height_range(uint32_t min, uint32_t max);

    inline const nucleation::capi::ScatterRule* AsFFI() const;
    inline nucleation::capi::ScatterRule* AsFFI();
    inline static const nucleation::ScatterRule* FromFFI(const nucleation::capi::ScatterRule* ptr);
    inline static nucleation::ScatterRule* FromFFI(nucleation::capi::ScatterRule* ptr);
    inline static void operator delete(void* ptr);
private:
    ScatterRule() = delete;
    ScatterRule(const nucleation::ScatterRule&) = delete;
    ScatterRule(nucleation::ScatterRule&&) noexcept = delete;
    ScatterRule operator=(const nucleation::ScatterRule&) = delete;
    ScatterRule operator=(nucleation::ScatterRule&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_ScatterRule_D_HPP
