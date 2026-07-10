#ifndef NUCLEATION_Scripting_D_HPP
#define NUCLEATION_Scripting_D_HPP

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
namespace capi { struct Schematic; }
class Schematic;
class NucleationError;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct Scripting;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * Namespace for the script-runner free functions of the old ABI
 * (`run_lua_script`, `run_js_script`, `run_script`).
 */
class Scripting {
public:

  /**
   * Run a Lua script file. Returns the schematic the script assigns to
   * `result`; `NotFound` if it produced none, `Parse` if it failed, and
   * `InvalidArgument` when built without the `scripting-lua` feature.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> run_lua_script(std::string_view path);

  /**
   * Run a JS script file. Returns the schematic the script assigns to
   * `result`; `NotFound` if it produced none, `Parse` if it failed, and
   * `InvalidArgument` when built without the `scripting-js` feature.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> run_js_script(std::string_view path);

  /**
   * Run a script file, auto-detecting the engine by extension (`.lua` or
   * `.js`). Returns the schematic the script assigns to `result`; `NotFound`
   * if it produced none, `Parse` if it failed (including unsupported
   * extensions).
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> run_script(std::string_view path);

    inline const nucleation::capi::Scripting* AsFFI() const;
    inline nucleation::capi::Scripting* AsFFI();
    inline static const nucleation::Scripting* FromFFI(const nucleation::capi::Scripting* ptr);
    inline static nucleation::Scripting* FromFFI(nucleation::capi::Scripting* ptr);
    inline static void operator delete(void* ptr);
private:
    Scripting() = delete;
    Scripting(const nucleation::Scripting&) = delete;
    Scripting(nucleation::Scripting&&) noexcept = delete;
    Scripting operator=(const nucleation::Scripting&) = delete;
    Scripting operator=(nucleation::Scripting&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_Scripting_D_HPP
