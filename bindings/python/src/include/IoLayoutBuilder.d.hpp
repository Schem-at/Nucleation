#ifndef NUCLEATION_IoLayoutBuilder_D_HPP
#define NUCLEATION_IoLayoutBuilder_D_HPP

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
namespace capi { struct IoLayout; }
class IoLayout;
namespace capi { struct IoLayoutBuilder; }
class IoLayoutBuilder;
namespace capi { struct IoType; }
class IoType;
namespace capi { struct LayoutFunction; }
class LayoutFunction;
class NucleationError;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct IoLayoutBuilder;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * Builder for an {@link IoLayout}. `build` consumes it (PORTING rule 11).
 */
class IoLayoutBuilder {
public:

  /**
   * Create an empty layout builder.
   */
  inline static std::unique_ptr<nucleation::IoLayoutBuilder> create();

  /**
   * Add an input. `positions` is flat `[x,y,z, x,y,z, ...]`.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> add_input(std::string_view name, const nucleation::IoType& io_type, const nucleation::LayoutFunction& layout, nucleation::diplomat::span<const int32_t> positions);

  /**
   * Add an output. `positions` is flat `[x,y,z, x,y,z, ...]`.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> add_output(std::string_view name, const nucleation::IoType& io_type, const nucleation::LayoutFunction& layout, nucleation::diplomat::span<const int32_t> positions);

  /**
   * Add an input with automatic layout inference.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> add_input_auto(std::string_view name, const nucleation::IoType& io_type, nucleation::diplomat::span<const int32_t> positions);

  /**
   * Add an output with automatic layout inference.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> add_output_auto(std::string_view name, const nucleation::IoType& io_type, nucleation::diplomat::span<const int32_t> positions);

  /**
   * Add an input from a region given as flat block positions (sorted by
   * the default YXZ strategy, matching the old region semantics).
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> add_input_from_region(std::string_view name, const nucleation::IoType& io_type, const nucleation::LayoutFunction& layout, nucleation::diplomat::span<const int32_t> region_positions);

  /**
   * Add an input from a region (flat positions) with automatic layout
   * inference.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> add_input_from_region_auto(std::string_view name, const nucleation::IoType& io_type, nucleation::diplomat::span<const int32_t> region_positions);

  /**
   * Add an output from a region given as flat block positions.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> add_output_from_region(std::string_view name, const nucleation::IoType& io_type, const nucleation::LayoutFunction& layout, nucleation::diplomat::span<const int32_t> region_positions);

  /**
   * Add an output from a region (flat positions) with automatic layout
   * inference.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> add_output_from_region_auto(std::string_view name, const nucleation::IoType& io_type, nucleation::diplomat::span<const int32_t> region_positions);

  /**
   * Build the {@link IoLayout}. Consumes the builder (a second call returns
   * `AlreadyConsumed`).
   */
  inline nucleation::diplomat::result<std::unique_ptr<nucleation::IoLayout>, nucleation::NucleationError> build();

    inline const nucleation::capi::IoLayoutBuilder* AsFFI() const;
    inline nucleation::capi::IoLayoutBuilder* AsFFI();
    inline static const nucleation::IoLayoutBuilder* FromFFI(const nucleation::capi::IoLayoutBuilder* ptr);
    inline static nucleation::IoLayoutBuilder* FromFFI(nucleation::capi::IoLayoutBuilder* ptr);
    inline static void operator delete(void* ptr);
private:
    IoLayoutBuilder() = delete;
    IoLayoutBuilder(const nucleation::IoLayoutBuilder&) = delete;
    IoLayoutBuilder(nucleation::IoLayoutBuilder&&) noexcept = delete;
    IoLayoutBuilder operator=(const nucleation::IoLayoutBuilder&) = delete;
    IoLayoutBuilder operator=(nucleation::IoLayoutBuilder&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_IoLayoutBuilder_D_HPP
