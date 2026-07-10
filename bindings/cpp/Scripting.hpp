#ifndef Scripting_HPP
#define Scripting_HPP

#include "Scripting.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "NucleationError.hpp"
#include "Schematic.hpp"
#include "diplomat_runtime.hpp"


namespace diplomat {
namespace capi {
    extern "C" {

    typedef struct Scripting_run_lua_script_result {union {diplomat::capi::Schematic* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Scripting_run_lua_script_result;
    Scripting_run_lua_script_result Scripting_run_lua_script(diplomat::capi::DiplomatStringView path);

    typedef struct Scripting_run_js_script_result {union {diplomat::capi::Schematic* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Scripting_run_js_script_result;
    Scripting_run_js_script_result Scripting_run_js_script(diplomat::capi::DiplomatStringView path);

    typedef struct Scripting_run_script_result {union {diplomat::capi::Schematic* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Scripting_run_script_result;
    Scripting_run_script_result Scripting_run_script(diplomat::capi::DiplomatStringView path);

    void Scripting_destroy(Scripting* self);

    } // extern "C"
} // namespace capi
} // namespace

inline diplomat::result<std::unique_ptr<Schematic>, NucleationError> Scripting::run_lua_script(std::string_view path) {
    auto result = diplomat::capi::Scripting_run_lua_script({path.data(), path.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Ok<std::unique_ptr<Schematic>>(std::unique_ptr<Schematic>(Schematic::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Schematic>, NucleationError> Scripting::run_js_script(std::string_view path) {
    auto result = diplomat::capi::Scripting_run_js_script({path.data(), path.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Ok<std::unique_ptr<Schematic>>(std::unique_ptr<Schematic>(Schematic::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Schematic>, NucleationError> Scripting::run_script(std::string_view path) {
    auto result = diplomat::capi::Scripting_run_script({path.data(), path.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Ok<std::unique_ptr<Schematic>>(std::unique_ptr<Schematic>(Schematic::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline const diplomat::capi::Scripting* Scripting::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::Scripting*>(this);
}

inline diplomat::capi::Scripting* Scripting::AsFFI() {
    return reinterpret_cast<diplomat::capi::Scripting*>(this);
}

inline const Scripting* Scripting::FromFFI(const diplomat::capi::Scripting* ptr) {
    return reinterpret_cast<const Scripting*>(ptr);
}

inline Scripting* Scripting::FromFFI(diplomat::capi::Scripting* ptr) {
    return reinterpret_cast<Scripting*>(ptr);
}

inline void Scripting::operator delete(void* ptr) {
    diplomat::capi::Scripting_destroy(reinterpret_cast<diplomat::capi::Scripting*>(ptr));
}


#endif // Scripting_HPP
