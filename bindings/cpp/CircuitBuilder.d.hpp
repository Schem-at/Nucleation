#ifndef CircuitBuilder_D_HPP
#define CircuitBuilder_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"

namespace diplomat::capi { struct IoType; }
class IoType;
namespace diplomat::capi { struct LayoutFunction; }
class LayoutFunction;
namespace diplomat::capi { struct Schematic; }
class Schematic;
namespace diplomat::capi { struct SortStrategy; }
class SortStrategy;
namespace diplomat::capi { struct TypedCircuitExecutor; }
class TypedCircuitExecutor;
class NucleationError;




namespace diplomat {
namespace capi {
    struct CircuitBuilder;
} // namespace capi
} // namespace

/**
 * High-level circuit builder. `build`/`build_validated` consume it
 * (PORTING rule 11). Regions are given as flat `[x,y,z,...]` positions
 * (see module notes).
 */
class CircuitBuilder {
public:

  /**
   * Create a builder for a schematic (cloned; no IO defined yet).
   */
  inline static std::unique_ptr<CircuitBuilder> create(const Schematic& schematic);

  /**
   * Create a builder pre-populated from Insign annotations.
   */
  inline static diplomat::result<std::unique_ptr<CircuitBuilder>, NucleationError> from_insign(const Schematic& schematic);

  /**
   * Add an input with full control.
   */
  inline diplomat::result<std::monostate, NucleationError> with_input(std::string_view name, const IoType& io_type, const LayoutFunction& layout, diplomat::span<const int32_t> region_positions);

  /**
   * Add an input with full control and a custom sort strategy.
   */
  inline diplomat::result<std::monostate, NucleationError> with_input_sorted(std::string_view name, const IoType& io_type, const LayoutFunction& layout, diplomat::span<const int32_t> region_positions, const SortStrategy& sort);

  /**
   * Add an input with automatic layout inference.
   */
  inline diplomat::result<std::monostate, NucleationError> with_input_auto(std::string_view name, const IoType& io_type, diplomat::span<const int32_t> region_positions);

  /**
   * Add an input with automatic layout inference and a custom sort.
   */
  inline diplomat::result<std::monostate, NucleationError> with_input_auto_sorted(std::string_view name, const IoType& io_type, diplomat::span<const int32_t> region_positions, const SortStrategy& sort);

  /**
   * Add an output with full control.
   */
  inline diplomat::result<std::monostate, NucleationError> with_output(std::string_view name, const IoType& io_type, const LayoutFunction& layout, diplomat::span<const int32_t> region_positions);

  /**
   * Add an output with full control and a custom sort strategy.
   */
  inline diplomat::result<std::monostate, NucleationError> with_output_sorted(std::string_view name, const IoType& io_type, const LayoutFunction& layout, diplomat::span<const int32_t> region_positions, const SortStrategy& sort);

  /**
   * Add an output with automatic layout inference.
   */
  inline diplomat::result<std::monostate, NucleationError> with_output_auto(std::string_view name, const IoType& io_type, diplomat::span<const int32_t> region_positions);

  /**
   * Add an output with automatic layout inference and a custom sort.
   */
  inline diplomat::result<std::monostate, NucleationError> with_output_auto_sorted(std::string_view name, const IoType& io_type, diplomat::span<const int32_t> region_positions, const SortStrategy& sort);

  /**
   * Set simulation options.
   */
  inline diplomat::result<std::monostate, NucleationError> with_options(bool optimize, bool io_only);

  /**
   * Set the state mode ("stateless" | "stateful" | "manual").
   */
  inline diplomat::result<std::monostate, NucleationError> with_state_mode(std::string_view mode);

  /**
   * Validate the configuration without consuming the builder.
   */
  inline diplomat::result<std::monostate, NucleationError> validate() const;

  /**
   * Build the executor. Consumes the builder.
   */
  inline diplomat::result<std::unique_ptr<TypedCircuitExecutor>, NucleationError> build();

  /**
   * Build the executor with validation. Consumes the builder.
   */
  inline diplomat::result<std::unique_ptr<TypedCircuitExecutor>, NucleationError> build_validated();

  /**
   * Number of inputs defined so far (0 if the builder was consumed).
   */
  inline uint32_t input_count() const;

  /**
   * Number of outputs defined so far (0 if the builder was consumed).
   */
  inline uint32_t output_count() const;

  /**
   * Input names as a JSON array string.
   */
  inline diplomat::result<std::string, NucleationError> input_names_json() const;
  template<typename W>
  inline diplomat::result<std::monostate, NucleationError> input_names_json_write(W& writeable_output) const;

  /**
   * Output names as a JSON array string.
   */
  inline diplomat::result<std::string, NucleationError> output_names_json() const;
  template<typename W>
  inline diplomat::result<std::monostate, NucleationError> output_names_json_write(W& writeable_output) const;

    inline const diplomat::capi::CircuitBuilder* AsFFI() const;
    inline diplomat::capi::CircuitBuilder* AsFFI();
    inline static const CircuitBuilder* FromFFI(const diplomat::capi::CircuitBuilder* ptr);
    inline static CircuitBuilder* FromFFI(diplomat::capi::CircuitBuilder* ptr);
    inline static void operator delete(void* ptr);
private:
    CircuitBuilder() = delete;
    CircuitBuilder(const CircuitBuilder&) = delete;
    CircuitBuilder(CircuitBuilder&&) noexcept = delete;
    CircuitBuilder operator=(const CircuitBuilder&) = delete;
    CircuitBuilder operator=(CircuitBuilder&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // CircuitBuilder_D_HPP
