#ifndef NUCLEATION_Field3_D_HPP
#define NUCLEATION_Field3_D_HPP

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
namespace capi { struct Field3; }
class Field3;
struct FieldRange;
class NucleationError;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct Field3;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * Immutable scalar field evaluated over world-space `(x, y, z)`.
 *
 * A `Field3` has scalar semantics only. It may be shared by geometry and
 * material consumers without being reinterpreted as a signed-distance field.
 */
class Field3 {
public:

  /**
   * Deterministic value-noise FBM normalized to `[-1, 1]`.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Field3>, nucleation::NucleationError> value_noise_fbm(float frequency, int32_t seed, uint32_t octaves);

  inline float eval_at(float x, float y, float z) const;

  /**
   * The field's analytically proven output range.
   *
   * Returns `NotFound` when no range can be proven — callers mapping a
   * field onto a gradient must handle that rather than silently
   * propagating a sentinel into their `lo`/`hi` bounds.
   */
  inline nucleation::diplomat::result<nucleation::FieldRange, nucleation::NucleationError> output_range() const;

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Field3>, nucleation::NucleationError> from_json_string(std::string_view json);

  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> to_json() const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> to_json_write(W& writeable_output) const;

    inline const nucleation::capi::Field3* AsFFI() const;
    inline nucleation::capi::Field3* AsFFI();
    inline static const nucleation::Field3* FromFFI(const nucleation::capi::Field3* ptr);
    inline static nucleation::Field3* FromFFI(nucleation::capi::Field3* ptr);
    inline static void operator delete(void* ptr);
private:
    Field3() = delete;
    Field3(const nucleation::Field3&) = delete;
    Field3(nucleation::Field3&&) noexcept = delete;
    Field3 operator=(const nucleation::Field3&) = delete;
    Field3 operator=(nucleation::Field3&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_Field3_D_HPP
