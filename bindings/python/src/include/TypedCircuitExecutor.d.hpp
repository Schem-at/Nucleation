#ifndef NUCLEATION_TypedCircuitExecutor_D_HPP
#define NUCLEATION_TypedCircuitExecutor_D_HPP

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
namespace capi { struct ExecutionMode; }
class ExecutionMode;
namespace capi { struct IoLayout; }
class IoLayout;
namespace capi { struct MchprsWorld; }
class MchprsWorld;
namespace capi { struct Schematic; }
class Schematic;
namespace capi { struct TypedCircuitExecutor; }
class TypedCircuitExecutor;
namespace capi { struct Value; }
class Value;
class NucleationError;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct TypedCircuitExecutor;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * A typed circuit executor. Wraps
 * {@link crate::simulation::typed_executor::TypedCircuitExecutor}.
 */
class TypedCircuitExecutor {
public:

  /**
   * Create from a world and layout. Builds a fresh world from the
   * world's schematic (matches the old ABI, which cloned internally).
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::TypedCircuitExecutor>, nucleation::NucleationError> from_layout(const nucleation::MchprsWorld& world, const nucleation::IoLayout& layout);

  /**
   * Create from a world and layout with simulation options.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::TypedCircuitExecutor>, nucleation::NucleationError> from_layout_with_options(const nucleation::MchprsWorld& world, const nucleation::IoLayout& layout, bool optimize, bool io_only);

  /**
   * Create from Insign annotations in a schematic.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::TypedCircuitExecutor>, nucleation::NucleationError> from_insign(const nucleation::Schematic& schematic);

  /**
   * Create from Insign annotations with options.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::TypedCircuitExecutor>, nucleation::NucleationError> from_insign_with_options(const nucleation::Schematic& schematic, bool optimize, bool io_only);

  /**
   * Set the state mode ("stateless" | "stateful" | "manual").
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> set_state_mode(std::string_view mode);

  /**
   * Reset the executor to its initial state.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> reset();

  /**
   * Advance the simulation by `ticks` ticks.
   */
  inline void tick(uint32_t ticks);

  /**
   * Flush pending changes.
   */
  inline void flush();

  /**
   * Set a single input value.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> set_input(std::string_view name, const nucleation::Value& value);

  /**
   * Read a single output value.
   */
  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Value>, nucleation::NucleationError> read_output(std::string_view name);

  /**
   * Execute the circuit. `inputs_json` is a JSON object like
   * `{"input_name": {"type": "u32", "value": 42}}`; writes a JSON
   * object with `outputs`, `ticks_elapsed` and `condition_met`.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> execute(std::string_view inputs_json, const nucleation::ExecutionMode& mode);
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> execute_write(std::string_view inputs_json, const nucleation::ExecutionMode& mode, W& writeable_output);

  /**
   * Input names as a JSON array string.
   */
  inline std::string input_names_json() const;
  template<typename W>
  inline void input_names_json_write(W& writeable_output) const;

  /**
   * Output names as a JSON array string.
   */
  inline std::string output_names_json() const;
  template<typename W>
  inline void output_names_json_write(W& writeable_output) const;

  /**
   * Layout info as a JSON object string
   * (old ABI: `typed_executor_get_layout_info`).
   */
  inline std::string layout_info_json() const;
  template<typename W>
  inline void layout_info_json_write(W& writeable_output) const;

  /**
   * Sync the simulation state and return the schematic (cloned).
   */
  inline std::unique_ptr<nucleation::Schematic> sync_to_schematic();

    inline const nucleation::capi::TypedCircuitExecutor* AsFFI() const;
    inline nucleation::capi::TypedCircuitExecutor* AsFFI();
    inline static const nucleation::TypedCircuitExecutor* FromFFI(const nucleation::capi::TypedCircuitExecutor* ptr);
    inline static nucleation::TypedCircuitExecutor* FromFFI(nucleation::capi::TypedCircuitExecutor* ptr);
    inline static void operator delete(void* ptr);
private:
    TypedCircuitExecutor() = delete;
    TypedCircuitExecutor(const nucleation::TypedCircuitExecutor&) = delete;
    TypedCircuitExecutor(nucleation::TypedCircuitExecutor&&) noexcept = delete;
    TypedCircuitExecutor operator=(const nucleation::TypedCircuitExecutor&) = delete;
    TypedCircuitExecutor operator=(nucleation::TypedCircuitExecutor&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_TypedCircuitExecutor_D_HPP
