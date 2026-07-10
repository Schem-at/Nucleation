#ifndef NUCLEATION_CircuitBuilder_D_HPP
#define NUCLEATION_CircuitBuilder_D_HPP

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
namespace capi { struct CircuitBuilder; }
class CircuitBuilder;
namespace capi { struct IoType; }
class IoType;
namespace capi { struct LayoutFunction; }
class LayoutFunction;
namespace capi { struct Schematic; }
class Schematic;
namespace capi { struct SortStrategy; }
class SortStrategy;
namespace capi { struct TypedCircuitExecutor; }
class TypedCircuitExecutor;
class NucleationError;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct CircuitBuilder;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * High-level circuit builder. `build`/`build_validated` consume it
 * (PORTING rule 11). Regions are given as flat `[x,y,z,...]` positions
 * (see module notes).
 */
class CircuitBuilder {
public:

  inline static std::unique_ptr<nucleation::CircuitBuilder> create(const nucleation::Schematic& schematic);

  /**
   * Create a builder pre-populated from Insign annotations.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::CircuitBuilder>, nucleation::NucleationError> from_insign(const nucleation::Schematic& schematic);

  /**
   * Add an input with full control.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> with_input(std::string_view name, const nucleation::IoType& io_type, const nucleation::LayoutFunction& layout, nucleation::diplomat::span<const int32_t> region_positions);

  /**
   * Add an input with full control and a custom sort strategy.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> with_input_sorted(std::string_view name, const nucleation::IoType& io_type, const nucleation::LayoutFunction& layout, nucleation::diplomat::span<const int32_t> region_positions, const nucleation::SortStrategy& sort);

  /**
   * Add an input with automatic layout inference.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> with_input_auto(std::string_view name, const nucleation::IoType& io_type, nucleation::diplomat::span<const int32_t> region_positions);

  /**
   * Add an input with automatic layout inference and a custom sort.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> with_input_auto_sorted(std::string_view name, const nucleation::IoType& io_type, nucleation::diplomat::span<const int32_t> region_positions, const nucleation::SortStrategy& sort);

  /**
   * Add an output with full control.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> with_output(std::string_view name, const nucleation::IoType& io_type, const nucleation::LayoutFunction& layout, nucleation::diplomat::span<const int32_t> region_positions);

  /**
   * Add an output with full control and a custom sort strategy.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> with_output_sorted(std::string_view name, const nucleation::IoType& io_type, const nucleation::LayoutFunction& layout, nucleation::diplomat::span<const int32_t> region_positions, const nucleation::SortStrategy& sort);

  /**
   * Add an output with automatic layout inference.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> with_output_auto(std::string_view name, const nucleation::IoType& io_type, nucleation::diplomat::span<const int32_t> region_positions);

  /**
   * Add an output with automatic layout inference and a custom sort.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> with_output_auto_sorted(std::string_view name, const nucleation::IoType& io_type, nucleation::diplomat::span<const int32_t> region_positions, const nucleation::SortStrategy& sort);

  /**
   * Set simulation options.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> with_options(bool optimize, bool io_only);

  /**
   * Set the state mode ("stateless" | "stateful" | "manual").
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> with_state_mode(std::string_view mode);

  /**
   * Validate the configuration without consuming the builder.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> validate() const;

  /**
   * Build the executor. Consumes the builder.
   */
  inline nucleation::diplomat::result<std::unique_ptr<nucleation::TypedCircuitExecutor>, nucleation::NucleationError> build();

  /**
   * Build the executor with validation. Consumes the builder.
   */
  inline nucleation::diplomat::result<std::unique_ptr<nucleation::TypedCircuitExecutor>, nucleation::NucleationError> build_validated();

  inline uint32_t input_count() const;

  inline uint32_t output_count() const;

  /**
   * Input names as a JSON array string.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> input_names_json() const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> input_names_json_write(W& writeable_output) const;

  /**
   * Output names as a JSON array string.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> output_names_json() const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> output_names_json_write(W& writeable_output) const;

    inline const nucleation::capi::CircuitBuilder* AsFFI() const;
    inline nucleation::capi::CircuitBuilder* AsFFI();
    inline static const nucleation::CircuitBuilder* FromFFI(const nucleation::capi::CircuitBuilder* ptr);
    inline static nucleation::CircuitBuilder* FromFFI(nucleation::capi::CircuitBuilder* ptr);
    inline static void operator delete(void* ptr);
private:
    CircuitBuilder() = delete;
    CircuitBuilder(const nucleation::CircuitBuilder&) = delete;
    CircuitBuilder(nucleation::CircuitBuilder&&) noexcept = delete;
    CircuitBuilder operator=(const nucleation::CircuitBuilder&) = delete;
    CircuitBuilder operator=(nucleation::CircuitBuilder&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_CircuitBuilder_D_HPP
