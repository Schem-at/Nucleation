#ifndef NUCLEATION_CellExecutor_HPP
#define NUCLEATION_CellExecutor_HPP

#include "CellExecutor.d.hpp"

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
#include "Value.hpp"
#include "diplomat_runtime.hpp"


namespace nucleation {
namespace capi {
    extern "C" {

    typedef struct CellExecutor_for_schematic_result {union {nucleation::capi::CellExecutor* ok; nucleation::capi::NucleationError err;}; bool is_ok;} CellExecutor_for_schematic_result;
    CellExecutor_for_schematic_result CellExecutor_for_schematic(const nucleation::capi::Schematic* schematic);

    typedef struct CellExecutor_set_input_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} CellExecutor_set_input_result;
    CellExecutor_set_input_result CellExecutor_set_input(nucleation::capi::CellExecutor* self, nucleation::diplomat::capi::DiplomatStringView name, const nucleation::capi::Value* value);

    bool CellExecutor_settle(nucleation::capi::CellExecutor* self, uint32_t budget);

    typedef struct CellExecutor_read_output_result {union {nucleation::capi::Value* ok; nucleation::capi::NucleationError err;}; bool is_ok;} CellExecutor_read_output_result;
    CellExecutor_read_output_result CellExecutor_read_output(nucleation::capi::CellExecutor* self, nucleation::diplomat::capi::DiplomatStringView name);

    typedef struct CellExecutor_reset_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} CellExecutor_reset_result;
    CellExecutor_reset_result CellExecutor_reset(nucleation::capi::CellExecutor* self);

    void CellExecutor_destroy(CellExecutor* self);

    } // extern "C"
} // namespace capi
} // namespace

inline nucleation::diplomat::result<std::unique_ptr<nucleation::CellExecutor>, nucleation::NucleationError> nucleation::CellExecutor::for_schematic(const nucleation::Schematic& schematic) {
    auto result = nucleation::capi::CellExecutor_for_schematic(schematic.AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::CellExecutor>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::CellExecutor>>(std::unique_ptr<nucleation::CellExecutor>(nucleation::CellExecutor::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::CellExecutor>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::CellExecutor::set_input(std::string_view name, const nucleation::Value& value) {
    auto result = nucleation::capi::CellExecutor_set_input(this->AsFFI(),
        {name.data(), name.size()},
        value.AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline bool nucleation::CellExecutor::settle(uint32_t budget) {
    auto result = nucleation::capi::CellExecutor_settle(this->AsFFI(),
        budget);
    return result;
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Value>, nucleation::NucleationError> nucleation::CellExecutor::read_output(std::string_view name) {
    auto result = nucleation::capi::CellExecutor_read_output(this->AsFFI(),
        {name.data(), name.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Value>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Value>>(std::unique_ptr<nucleation::Value>(nucleation::Value::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Value>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::CellExecutor::reset() {
    auto result = nucleation::capi::CellExecutor_reset(this->AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline const nucleation::capi::CellExecutor* nucleation::CellExecutor::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::CellExecutor*>(this);
}

inline nucleation::capi::CellExecutor* nucleation::CellExecutor::AsFFI() {
    return reinterpret_cast<nucleation::capi::CellExecutor*>(this);
}

inline const nucleation::CellExecutor* nucleation::CellExecutor::FromFFI(const nucleation::capi::CellExecutor* ptr) {
    return reinterpret_cast<const nucleation::CellExecutor*>(ptr);
}

inline nucleation::CellExecutor* nucleation::CellExecutor::FromFFI(nucleation::capi::CellExecutor* ptr) {
    return reinterpret_cast<nucleation::CellExecutor*>(ptr);
}

inline void nucleation::CellExecutor::operator delete(void* ptr) {
    nucleation::capi::CellExecutor_destroy(reinterpret_cast<nucleation::capi::CellExecutor*>(ptr));
}


#endif // NUCLEATION_CellExecutor_HPP
