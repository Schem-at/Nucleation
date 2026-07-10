#ifndef IoLayoutBuilder_D_HPP
#define IoLayoutBuilder_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"

namespace diplomat::capi { struct IoLayout; }
class IoLayout;
namespace diplomat::capi { struct IoType; }
class IoType;
namespace diplomat::capi { struct LayoutFunction; }
class LayoutFunction;
class NucleationError;




namespace diplomat {
namespace capi {
    struct IoLayoutBuilder;
} // namespace capi
} // namespace

/**
 * Builder for an {@link IoLayout}. `build` consumes it (PORTING rule 11).
 */
class IoLayoutBuilder {
public:

  inline static std::unique_ptr<IoLayoutBuilder> create();

  /**
   * Add an input. `positions` is flat `[x,y,z, x,y,z, ...]`.
   */
  inline diplomat::result<std::monostate, NucleationError> add_input(std::string_view name, const IoType& io_type, const LayoutFunction& layout, diplomat::span<const int32_t> positions);

  /**
   * Add an output. `positions` is flat `[x,y,z, x,y,z, ...]`.
   */
  inline diplomat::result<std::monostate, NucleationError> add_output(std::string_view name, const IoType& io_type, const LayoutFunction& layout, diplomat::span<const int32_t> positions);

  /**
   * Add an input with automatic layout inference.
   */
  inline diplomat::result<std::monostate, NucleationError> add_input_auto(std::string_view name, const IoType& io_type, diplomat::span<const int32_t> positions);

  /**
   * Add an output with automatic layout inference.
   */
  inline diplomat::result<std::monostate, NucleationError> add_output_auto(std::string_view name, const IoType& io_type, diplomat::span<const int32_t> positions);

  /**
   * Add an input from a region given as flat block positions (sorted by
   * the default YXZ strategy, matching the old region semantics).
   */
  inline diplomat::result<std::monostate, NucleationError> add_input_from_region(std::string_view name, const IoType& io_type, const LayoutFunction& layout, diplomat::span<const int32_t> region_positions);

  /**
   * Add an input from a region (flat positions) with automatic layout
   * inference.
   */
  inline diplomat::result<std::monostate, NucleationError> add_input_from_region_auto(std::string_view name, const IoType& io_type, diplomat::span<const int32_t> region_positions);

  /**
   * Add an output from a region given as flat block positions.
   */
  inline diplomat::result<std::monostate, NucleationError> add_output_from_region(std::string_view name, const IoType& io_type, const LayoutFunction& layout, diplomat::span<const int32_t> region_positions);

  /**
   * Add an output from a region (flat positions) with automatic layout
   * inference.
   */
  inline diplomat::result<std::monostate, NucleationError> add_output_from_region_auto(std::string_view name, const IoType& io_type, diplomat::span<const int32_t> region_positions);

  /**
   * Build the {@link IoLayout}. Consumes the builder (a second call returns
   * `AlreadyConsumed`).
   */
  inline diplomat::result<std::unique_ptr<IoLayout>, NucleationError> build();

    inline const diplomat::capi::IoLayoutBuilder* AsFFI() const;
    inline diplomat::capi::IoLayoutBuilder* AsFFI();
    inline static const IoLayoutBuilder* FromFFI(const diplomat::capi::IoLayoutBuilder* ptr);
    inline static IoLayoutBuilder* FromFFI(diplomat::capi::IoLayoutBuilder* ptr);
    inline static void operator delete(void* ptr);
private:
    IoLayoutBuilder() = delete;
    IoLayoutBuilder(const IoLayoutBuilder&) = delete;
    IoLayoutBuilder(IoLayoutBuilder&&) noexcept = delete;
    IoLayoutBuilder operator=(const IoLayoutBuilder&) = delete;
    IoLayoutBuilder operator=(IoLayoutBuilder&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // IoLayoutBuilder_D_HPP
