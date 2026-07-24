#ifndef FieldProgram_D_HPP
#define FieldProgram_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"

struct SdfBounds;
struct SdfNormal;
class FieldProgramDistanceKind;
class NucleationError;




namespace diplomat {
namespace capi {
    struct FieldProgram;
} // namespace capi
} // namespace

/**
 * A validated, sandboxed custom SDF field program: deterministic typed
 * bytecode over scalar/vec3/bool values with bounded loops, carrying
 * its own explicit finite bounds and distance-kind metadata. Build one
 * with {@link FieldProgramBuilder} or import it from JSON.
 */
class FieldProgram {
public:

  inline static diplomat::result<std::unique_ptr<FieldProgram>, NucleationError> from_json_string(std::string_view json);

  inline diplomat::result<std::string, NucleationError> to_json() const;
  template<typename W>
  inline diplomat::result<std::monostate, NucleationError> to_json_write(W& writeable_output) const;

  inline float eval_at(float x, float y, float z) const;

  /**
   * Unit-length gradient of the scalar output at `(x, y, z)`: the
   * program's own forward-mode analytic gradient where it's
   * differentiable there, falling back to a numerical estimate
   * (central differences via `epsilon`) otherwise.
   */
  inline diplomat::result<SdfNormal, NucleationError> gradient(float x, float y, float z, float epsilon) const;

  inline SdfBounds bounds() const;

  inline FieldProgramDistanceKind distance_kind() const;

    inline const diplomat::capi::FieldProgram* AsFFI() const;
    inline diplomat::capi::FieldProgram* AsFFI();
    inline static const FieldProgram* FromFFI(const diplomat::capi::FieldProgram* ptr);
    inline static FieldProgram* FromFFI(diplomat::capi::FieldProgram* ptr);
    inline static void operator delete(void* ptr);
private:
    FieldProgram() = delete;
    FieldProgram(const FieldProgram&) = delete;
    FieldProgram(FieldProgram&&) noexcept = delete;
    FieldProgram operator=(const FieldProgram&) = delete;
    FieldProgram operator=(FieldProgram&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // FieldProgram_D_HPP
