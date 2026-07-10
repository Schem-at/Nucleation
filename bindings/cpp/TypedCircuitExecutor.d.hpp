#ifndef TypedCircuitExecutor_D_HPP
#define TypedCircuitExecutor_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"

namespace diplomat::capi { struct ExecutionMode; }
class ExecutionMode;
namespace diplomat::capi { struct IoLayout; }
class IoLayout;
namespace diplomat::capi { struct MchprsWorld; }
class MchprsWorld;
namespace diplomat::capi { struct Schematic; }
class Schematic;
namespace diplomat::capi { struct Value; }
class Value;
class NucleationError;




namespace diplomat {
namespace capi {
    struct TypedCircuitExecutor;
} // namespace capi
} // namespace

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
  inline static diplomat::result<std::unique_ptr<TypedCircuitExecutor>, NucleationError> from_layout(const MchprsWorld& world, const IoLayout& layout);

  /**
   * Create from a world and layout with simulation options.
   */
  inline static diplomat::result<std::unique_ptr<TypedCircuitExecutor>, NucleationError> from_layout_with_options(const MchprsWorld& world, const IoLayout& layout, bool optimize, bool io_only);

  /**
   * Create from Insign annotations in a schematic.
   */
  inline static diplomat::result<std::unique_ptr<TypedCircuitExecutor>, NucleationError> from_insign(const Schematic& schematic);

  /**
   * Create from Insign annotations with options.
   */
  inline static diplomat::result<std::unique_ptr<TypedCircuitExecutor>, NucleationError> from_insign_with_options(const Schematic& schematic, bool optimize, bool io_only);

  /**
   * Set the state mode ("stateless" | "stateful" | "manual").
   */
  inline diplomat::result<std::monostate, NucleationError> set_state_mode(std::string_view mode);

  /**
   * Reset the executor to its initial state.
   */
  inline diplomat::result<std::monostate, NucleationError> reset();

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
  inline diplomat::result<std::monostate, NucleationError> set_input(std::string_view name, const Value& value);

  /**
   * Read a single output value.
   */
  inline diplomat::result<std::unique_ptr<Value>, NucleationError> read_output(std::string_view name);

  /**
   * Execute the circuit. `inputs_json` is a JSON object like
   * `{"input_name": {"type": "u32", "value": 42}}`; writes a JSON
   * object with `outputs`, `ticks_elapsed` and `condition_met`.
   */
  inline diplomat::result<std::string, NucleationError> execute(std::string_view inputs_json, const ExecutionMode& mode);
  template<typename W>
  inline diplomat::result<std::monostate, NucleationError> execute_write(std::string_view inputs_json, const ExecutionMode& mode, W& writeable_output);

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
  inline std::unique_ptr<Schematic> sync_to_schematic();

    inline const diplomat::capi::TypedCircuitExecutor* AsFFI() const;
    inline diplomat::capi::TypedCircuitExecutor* AsFFI();
    inline static const TypedCircuitExecutor* FromFFI(const diplomat::capi::TypedCircuitExecutor* ptr);
    inline static TypedCircuitExecutor* FromFFI(diplomat::capi::TypedCircuitExecutor* ptr);
    inline static void operator delete(void* ptr);
private:
    TypedCircuitExecutor() = delete;
    TypedCircuitExecutor(const TypedCircuitExecutor&) = delete;
    TypedCircuitExecutor(TypedCircuitExecutor&&) noexcept = delete;
    TypedCircuitExecutor operator=(const TypedCircuitExecutor&) = delete;
    TypedCircuitExecutor operator=(TypedCircuitExecutor&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // TypedCircuitExecutor_D_HPP
