#ifndef Hdl_HPP
#define Hdl_HPP

#include "Hdl.d.hpp"

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

    typedef struct Hdl_compile_blif_result {union {diplomat::capi::Schematic* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Hdl_compile_blif_result;
    Hdl_compile_blif_result Hdl_compile_blif(diplomat::capi::DiplomatStringView blif, diplomat::capi::DiplomatStringView name, bool bake);

    typedef struct Hdl_compile_blif_report_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Hdl_compile_blif_report_result;
    Hdl_compile_blif_report_result Hdl_compile_blif_report(diplomat::capi::DiplomatStringView blif, diplomat::capi::DiplomatStringView name, diplomat::capi::DiplomatWrite* write);

    typedef struct Hdl_compile_blif_contract_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Hdl_compile_blif_contract_result;
    Hdl_compile_blif_contract_result Hdl_compile_blif_contract(diplomat::capi::DiplomatStringView blif, diplomat::capi::DiplomatStringView name, diplomat::capi::DiplomatWrite* write);

    void Hdl_destroy(Hdl* self);

    } // extern "C"
} // namespace capi
} // namespace

inline diplomat::result<std::unique_ptr<Schematic>, NucleationError> Hdl::compile_blif(std::string_view blif, std::string_view name, bool bake) {
    auto result = diplomat::capi::Hdl_compile_blif({blif.data(), blif.size()},
        {name.data(), name.size()},
        bake);
    return result.is_ok ? diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Ok<std::unique_ptr<Schematic>>(std::unique_ptr<Schematic>(Schematic::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> Hdl::compile_blif_report(std::string_view blif, std::string_view name) {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::Hdl_compile_blif_report({blif.data(), blif.size()},
        {name.data(), name.size()},
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> Hdl::compile_blif_report_write(std::string_view blif, std::string_view name, W& writeable) {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::Hdl_compile_blif_report({blif.data(), blif.size()},
        {name.data(), name.size()},
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> Hdl::compile_blif_contract(std::string_view blif, std::string_view name) {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::Hdl_compile_blif_contract({blif.data(), blif.size()},
        {name.data(), name.size()},
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> Hdl::compile_blif_contract_write(std::string_view blif, std::string_view name, W& writeable) {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::Hdl_compile_blif_contract({blif.data(), blif.size()},
        {name.data(), name.size()},
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline const diplomat::capi::Hdl* Hdl::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::Hdl*>(this);
}

inline diplomat::capi::Hdl* Hdl::AsFFI() {
    return reinterpret_cast<diplomat::capi::Hdl*>(this);
}

inline const Hdl* Hdl::FromFFI(const diplomat::capi::Hdl* ptr) {
    return reinterpret_cast<const Hdl*>(ptr);
}

inline Hdl* Hdl::FromFFI(diplomat::capi::Hdl* ptr) {
    return reinterpret_cast<Hdl*>(ptr);
}

inline void Hdl::operator delete(void* ptr) {
    diplomat::capi::Hdl_destroy(reinterpret_cast<diplomat::capi::Hdl*>(ptr));
}


#endif // Hdl_HPP
