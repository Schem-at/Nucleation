#ifndef NUCLEATION_Scripting_HPP
#define NUCLEATION_Scripting_HPP

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


namespace nucleation {
namespace capi {
    extern "C" {

    typedef struct Scripting_run_lua_script_result {union {nucleation::capi::Schematic* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Scripting_run_lua_script_result;
    Scripting_run_lua_script_result Scripting_run_lua_script(nucleation::diplomat::capi::DiplomatStringView path);

    typedef struct Scripting_run_js_script_result {union {nucleation::capi::Schematic* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Scripting_run_js_script_result;
    Scripting_run_js_script_result Scripting_run_js_script(nucleation::diplomat::capi::DiplomatStringView path);

    typedef struct Scripting_run_script_result {union {nucleation::capi::Schematic* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Scripting_run_script_result;
    Scripting_run_script_result Scripting_run_script(nucleation::diplomat::capi::DiplomatStringView path);

    void Scripting_destroy(Scripting* self);

    } // extern "C"
} // namespace capi
} // namespace

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> nucleation::Scripting::run_lua_script(std::string_view path) {
    auto result = nucleation::capi::Scripting_run_lua_script({path.data(), path.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Schematic>>(std::unique_ptr<nucleation::Schematic>(nucleation::Schematic::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> nucleation::Scripting::run_js_script(std::string_view path) {
    auto result = nucleation::capi::Scripting_run_js_script({path.data(), path.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Schematic>>(std::unique_ptr<nucleation::Schematic>(nucleation::Schematic::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> nucleation::Scripting::run_script(std::string_view path) {
    auto result = nucleation::capi::Scripting_run_script({path.data(), path.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Schematic>>(std::unique_ptr<nucleation::Schematic>(nucleation::Schematic::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline const nucleation::capi::Scripting* nucleation::Scripting::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::Scripting*>(this);
}

inline nucleation::capi::Scripting* nucleation::Scripting::AsFFI() {
    return reinterpret_cast<nucleation::capi::Scripting*>(this);
}

inline const nucleation::Scripting* nucleation::Scripting::FromFFI(const nucleation::capi::Scripting* ptr) {
    return reinterpret_cast<const nucleation::Scripting*>(ptr);
}

inline nucleation::Scripting* nucleation::Scripting::FromFFI(nucleation::capi::Scripting* ptr) {
    return reinterpret_cast<nucleation::Scripting*>(ptr);
}

inline void nucleation::Scripting::operator delete(void* ptr) {
    nucleation::capi::Scripting_destroy(reinterpret_cast<nucleation::capi::Scripting*>(ptr));
}


#endif // NUCLEATION_Scripting_HPP
