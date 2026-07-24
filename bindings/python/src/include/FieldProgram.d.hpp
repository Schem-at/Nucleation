#ifndef NUCLEATION_FieldProgram_D_HPP
#define NUCLEATION_FieldProgram_D_HPP

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
namespace capi { struct FieldProgram; }
class FieldProgram;
struct SdfBounds;
struct SdfNormal;
class FieldProgramDistanceKind;
class NucleationError;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct FieldProgram;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * A validated, sandboxed custom SDF field program: deterministic typed
 * bytecode over scalar/vec3/bool values with bounded loops, carrying
 * its own explicit finite bounds and distance-kind metadata. Build one
 * with {@link FieldProgramBuilder} or import it from JSON.
 */
class FieldProgram {
public:

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::FieldProgram>, nucleation::NucleationError> from_json_string(std::string_view json);

  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> to_json() const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> to_json_write(W& writeable_output) const;

  inline float eval_at(float x, float y, float z) const;

  /**
   * Unit-length gradient of the scalar output at `(x, y, z)`: the
   * program's own forward-mode analytic gradient where it's
   * differentiable there, falling back to a numerical estimate
   * (central differences via `epsilon`) otherwise.
   */
  inline nucleation::diplomat::result<nucleation::SdfNormal, nucleation::NucleationError> gradient(float x, float y, float z, float epsilon) const;

  inline nucleation::SdfBounds bounds() const;

  inline nucleation::FieldProgramDistanceKind distance_kind() const;

    inline const nucleation::capi::FieldProgram* AsFFI() const;
    inline nucleation::capi::FieldProgram* AsFFI();
    inline static const nucleation::FieldProgram* FromFFI(const nucleation::capi::FieldProgram* ptr);
    inline static nucleation::FieldProgram* FromFFI(nucleation::capi::FieldProgram* ptr);
    inline static void operator delete(void* ptr);
private:
    FieldProgram() = delete;
    FieldProgram(const nucleation::FieldProgram&) = delete;
    FieldProgram(nucleation::FieldProgram&&) noexcept = delete;
    FieldProgram operator=(const nucleation::FieldProgram&) = delete;
    FieldProgram operator=(nucleation::FieldProgram&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_FieldProgram_D_HPP
