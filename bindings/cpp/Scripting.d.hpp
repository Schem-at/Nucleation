#ifndef Scripting_D_HPP
#define Scripting_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"

namespace diplomat::capi { struct Schematic; }
class Schematic;
class NucleationError;




namespace diplomat {
namespace capi {
    struct Scripting;
} // namespace capi
} // namespace

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
  inline static diplomat::result<std::unique_ptr<Schematic>, NucleationError> run_lua_script(std::string_view path);

  /**
   * Run a JS script file. Returns the schematic the script assigns to
   * `result`; `NotFound` if it produced none, `Parse` if it failed, and
   * `InvalidArgument` when built without the `scripting-js` feature.
   */
  inline static diplomat::result<std::unique_ptr<Schematic>, NucleationError> run_js_script(std::string_view path);

  /**
   * Run a script file, auto-detecting the engine by extension (`.lua` or
   * `.js`). Returns the schematic the script assigns to `result`; `NotFound`
   * if it produced none, `Parse` if it failed (including unsupported
   * extensions).
   */
  inline static diplomat::result<std::unique_ptr<Schematic>, NucleationError> run_script(std::string_view path);

    inline const diplomat::capi::Scripting* AsFFI() const;
    inline diplomat::capi::Scripting* AsFFI();
    inline static const Scripting* FromFFI(const diplomat::capi::Scripting* ptr);
    inline static Scripting* FromFFI(diplomat::capi::Scripting* ptr);
    inline static void operator delete(void* ptr);
private:
    Scripting() = delete;
    Scripting(const Scripting&) = delete;
    Scripting(Scripting&&) noexcept = delete;
    Scripting operator=(const Scripting&) = delete;
    Scripting operator=(Scripting&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // Scripting_D_HPP
