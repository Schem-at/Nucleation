#ifndef NUCLEATION_FieldProgramBuilder_D_HPP
#define NUCLEATION_FieldProgramBuilder_D_HPP

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
namespace capi { struct FieldProgramBuilder; }
class FieldProgramBuilder;
class FieldProgramBinaryOp;
class FieldProgramDistanceKind;
class FieldProgramUnaryOp;
class FieldProgramValueType;
class NucleationError;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct FieldProgramBuilder;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * Programmatic builder for a {@link FieldProgram}: append typed stack
 * instructions, then {@link FieldProgramBuilder::build} to validate and
 * obtain a {@link FieldProgram}. Consuming: every method after `build()`
 * (successful or not) returns `AlreadyConsumed`.
 */
class FieldProgramBuilder {
public:

  inline static std::unique_ptr<nucleation::FieldProgramBuilder> create();

  /**
   * Declare a new typed local slot and return its index.
   */
  inline nucleation::diplomat::result<uint16_t, nucleation::NucleationError> add_slot(nucleation::FieldProgramValueType value_type);

  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> push_const_scalar(float value);

  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> push_const_vec3(float x, float y, float z);

  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> push_const_bool(bool value);

  /**
   * Push the `Vec3` position the program is being evaluated at.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> push_pos();

  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> load_local(uint16_t slot);

  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> store_local(uint16_t slot);

  /**
   * Discard the top of the stack.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> pop();

  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> unary_op(nucleation::FieldProgramUnaryOp op);

  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> binary_op(nucleation::FieldProgramBinaryOp op);

  /**
   * Pop `(x, lo, hi)`, push `x` clamped to `[lo, hi]`.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> clamp();

  /**
   * Pop `(a, b, cond)`, push `a` if `cond` else `b`.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> select();

  /**
   * Pop `(x, y, z)`, push `Vec3([x, y, z])`.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> make_vec3();

  /**
   * Pop a `Bool`; if true, stop the nearest enclosing repeat after
   * this iteration. Only valid inside `beginRepeat`/`endRepeat`.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> break_if();

  /**
   * Open a new statically bounded repeat block; subsequent
   * instructions append to its body until `endRepeat`.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> begin_repeat(uint32_t count);

  /**
   * Close the innermost open repeat block.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> end_repeat();

  /**
   * Declare which scalar slot holds the program's output.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> set_output(uint16_t slot);

  /**
   * Set the program's explicit, author-asserted finite bounds.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> set_bounds(float min_x, float min_y, float min_z, float max_x, float max_y, float max_z);

  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> set_distance_kind(nucleation::FieldProgramDistanceKind kind);

  /**
   * Validate and finalize. Consumes the builder even on failure.
   */
  inline nucleation::diplomat::result<std::unique_ptr<nucleation::FieldProgram>, nucleation::NucleationError> build();

    inline const nucleation::capi::FieldProgramBuilder* AsFFI() const;
    inline nucleation::capi::FieldProgramBuilder* AsFFI();
    inline static const nucleation::FieldProgramBuilder* FromFFI(const nucleation::capi::FieldProgramBuilder* ptr);
    inline static nucleation::FieldProgramBuilder* FromFFI(nucleation::capi::FieldProgramBuilder* ptr);
    inline static void operator delete(void* ptr);
private:
    FieldProgramBuilder() = delete;
    FieldProgramBuilder(const nucleation::FieldProgramBuilder&) = delete;
    FieldProgramBuilder(nucleation::FieldProgramBuilder&&) noexcept = delete;
    FieldProgramBuilder operator=(const nucleation::FieldProgramBuilder&) = delete;
    FieldProgramBuilder operator=(nucleation::FieldProgramBuilder&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_FieldProgramBuilder_D_HPP
