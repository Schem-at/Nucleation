#ifndef NUCLEATION_Hdl_HPP
#define NUCLEATION_Hdl_HPP

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


namespace nucleation {
namespace capi {
    extern "C" {

    typedef struct Hdl_compile_blif_result {union {nucleation::capi::Schematic* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Hdl_compile_blif_result;
    Hdl_compile_blif_result Hdl_compile_blif(nucleation::diplomat::capi::DiplomatStringView blif, nucleation::diplomat::capi::DiplomatStringView name, bool bake);

    typedef struct Hdl_compile_blif_report_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Hdl_compile_blif_report_result;
    Hdl_compile_blif_report_result Hdl_compile_blif_report(nucleation::diplomat::capi::DiplomatStringView blif, nucleation::diplomat::capi::DiplomatStringView name, nucleation::diplomat::capi::DiplomatWrite* write);

    void Hdl_destroy(Hdl* self);

    } // extern "C"
} // namespace capi
} // namespace

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> nucleation::Hdl::compile_blif(std::string_view blif, std::string_view name, bool bake) {
    auto result = nucleation::capi::Hdl_compile_blif({blif.data(), blif.size()},
        {name.data(), name.size()},
        bake);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Schematic>>(std::unique_ptr<nucleation::Schematic>(nucleation::Schematic::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::Hdl::compile_blif_report(std::string_view blif, std::string_view name) {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::Hdl_compile_blif_report({blif.data(), blif.size()},
        {name.data(), name.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Hdl::compile_blif_report_write(std::string_view blif, std::string_view name, W& writeable) {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::Hdl_compile_blif_report({blif.data(), blif.size()},
        {name.data(), name.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline const nucleation::capi::Hdl* nucleation::Hdl::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::Hdl*>(this);
}

inline nucleation::capi::Hdl* nucleation::Hdl::AsFFI() {
    return reinterpret_cast<nucleation::capi::Hdl*>(this);
}

inline const nucleation::Hdl* nucleation::Hdl::FromFFI(const nucleation::capi::Hdl* ptr) {
    return reinterpret_cast<const nucleation::Hdl*>(ptr);
}

inline nucleation::Hdl* nucleation::Hdl::FromFFI(nucleation::capi::Hdl* ptr) {
    return reinterpret_cast<nucleation::Hdl*>(ptr);
}

inline void nucleation::Hdl::operator delete(void* ptr) {
    nucleation::capi::Hdl_destroy(reinterpret_cast<nucleation::capi::Hdl*>(ptr));
}


#endif // NUCLEATION_Hdl_HPP
