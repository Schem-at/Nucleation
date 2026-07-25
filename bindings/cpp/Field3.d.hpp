#ifndef Field3_D_HPP
#define Field3_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"

struct FieldRange;
class NucleationError;




namespace diplomat {
namespace capi {
    struct Field3;
} // namespace capi
} // namespace

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
  inline static diplomat::result<std::unique_ptr<Field3>, NucleationError> value_noise_fbm(float frequency, int32_t seed, uint32_t octaves);

  inline float eval_at(float x, float y, float z) const;

  /**
   * The field's analytically proven output range.
   *
   * Returns `NotFound` when no range can be proven — callers mapping a
   * field onto a gradient must handle that rather than silently
   * propagating a sentinel into their `lo`/`hi` bounds.
   */
  inline diplomat::result<FieldRange, NucleationError> output_range() const;

  inline static diplomat::result<std::unique_ptr<Field3>, NucleationError> from_json_string(std::string_view json);

  inline diplomat::result<std::string, NucleationError> to_json() const;
  template<typename W>
  inline diplomat::result<std::monostate, NucleationError> to_json_write(W& writeable_output) const;

    inline const diplomat::capi::Field3* AsFFI() const;
    inline diplomat::capi::Field3* AsFFI();
    inline static const Field3* FromFFI(const diplomat::capi::Field3* ptr);
    inline static Field3* FromFFI(diplomat::capi::Field3* ptr);
    inline static void operator delete(void* ptr);
private:
    Field3() = delete;
    Field3(const Field3&) = delete;
    Field3(Field3&&) noexcept = delete;
    Field3 operator=(const Field3&) = delete;
    Field3 operator=(Field3&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // Field3_D_HPP
