#ifndef FieldProgramBuilder_D_HPP
#define FieldProgramBuilder_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"

namespace diplomat::capi { struct FieldProgram; }
class FieldProgram;
class FieldProgramBinaryOp;
class FieldProgramDistanceKind;
class FieldProgramUnaryOp;
class FieldProgramValueType;
class NucleationError;




namespace diplomat {
namespace capi {
    struct FieldProgramBuilder;
} // namespace capi
} // namespace

/**
 * Programmatic builder for a {@link FieldProgram}: append typed stack
 * instructions, then {@link FieldProgramBuilder::build} to validate and
 * obtain a {@link FieldProgram}. Consuming: every method after `build()`
 * (successful or not) returns `AlreadyConsumed`.
 */
class FieldProgramBuilder {
public:

  inline static std::unique_ptr<FieldProgramBuilder> create();

  /**
   * Declare a new typed local slot and return its index.
   */
  inline diplomat::result<uint16_t, NucleationError> add_slot(FieldProgramValueType value_type);

  inline diplomat::result<std::monostate, NucleationError> push_const_scalar(float value);

  inline diplomat::result<std::monostate, NucleationError> push_const_vec3(float x, float y, float z);

  inline diplomat::result<std::monostate, NucleationError> push_const_bool(bool value);

  /**
   * Push the `Vec3` position the program is being evaluated at.
   */
  inline diplomat::result<std::monostate, NucleationError> push_pos();

  inline diplomat::result<std::monostate, NucleationError> load_local(uint16_t slot);

  inline diplomat::result<std::monostate, NucleationError> store_local(uint16_t slot);

  /**
   * Discard the top of the stack.
   */
  inline diplomat::result<std::monostate, NucleationError> pop();

  inline diplomat::result<std::monostate, NucleationError> unary_op(FieldProgramUnaryOp op);

  inline diplomat::result<std::monostate, NucleationError> binary_op(FieldProgramBinaryOp op);

  /**
   * Pop `(x, lo, hi)`, push `x` clamped to `[lo, hi]`.
   */
  inline diplomat::result<std::monostate, NucleationError> clamp();

  /**
   * Pop `(a, b, cond)`, push `a` if `cond` else `b`.
   */
  inline diplomat::result<std::monostate, NucleationError> select();

  /**
   * Pop `(x, y, z)`, push `Vec3([x, y, z])`.
   */
  inline diplomat::result<std::monostate, NucleationError> make_vec3();

  /**
   * Pop a `Bool`; if true, stop the nearest enclosing repeat after
   * this iteration. Only valid inside `beginRepeat`/`endRepeat`.
   */
  inline diplomat::result<std::monostate, NucleationError> break_if();

  /**
   * Open a new statically bounded repeat block; subsequent
   * instructions append to its body until `endRepeat`.
   */
  inline diplomat::result<std::monostate, NucleationError> begin_repeat(uint32_t count);

  /**
   * Close the innermost open repeat block.
   */
  inline diplomat::result<std::monostate, NucleationError> end_repeat();

  /**
   * Declare which scalar slot holds the program's output.
   */
  inline diplomat::result<std::monostate, NucleationError> set_output(uint16_t slot);

  /**
   * Set the program's explicit, author-asserted finite bounds.
   */
  inline diplomat::result<std::monostate, NucleationError> set_bounds(float min_x, float min_y, float min_z, float max_x, float max_y, float max_z);

  inline diplomat::result<std::monostate, NucleationError> set_distance_kind(FieldProgramDistanceKind kind);

  /**
   * Validate and finalize. Consumes the builder even on failure.
   */
  inline diplomat::result<std::unique_ptr<FieldProgram>, NucleationError> build();

    inline const diplomat::capi::FieldProgramBuilder* AsFFI() const;
    inline diplomat::capi::FieldProgramBuilder* AsFFI();
    inline static const FieldProgramBuilder* FromFFI(const diplomat::capi::FieldProgramBuilder* ptr);
    inline static FieldProgramBuilder* FromFFI(diplomat::capi::FieldProgramBuilder* ptr);
    inline static void operator delete(void* ptr);
private:
    FieldProgramBuilder() = delete;
    FieldProgramBuilder(const FieldProgramBuilder&) = delete;
    FieldProgramBuilder(FieldProgramBuilder&&) noexcept = delete;
    FieldProgramBuilder operator=(const FieldProgramBuilder&) = delete;
    FieldProgramBuilder operator=(FieldProgramBuilder&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // FieldProgramBuilder_D_HPP
