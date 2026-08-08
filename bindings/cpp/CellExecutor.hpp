#ifndef CellExecutor_HPP
#define CellExecutor_HPP

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


namespace diplomat {
namespace capi {
    extern "C" {

    typedef struct CellExecutor_for_schematic_result {union {diplomat::capi::CellExecutor* ok; diplomat::capi::NucleationError err;}; bool is_ok;} CellExecutor_for_schematic_result;
    CellExecutor_for_schematic_result CellExecutor_for_schematic(const diplomat::capi::Schematic* schematic);

    typedef struct CellExecutor_set_input_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} CellExecutor_set_input_result;
    CellExecutor_set_input_result CellExecutor_set_input(diplomat::capi::CellExecutor* self, diplomat::capi::DiplomatStringView name, const diplomat::capi::Value* value);

    bool CellExecutor_settle(diplomat::capi::CellExecutor* self, uint32_t budget);

    typedef struct CellExecutor_read_output_result {union {diplomat::capi::Value* ok; diplomat::capi::NucleationError err;}; bool is_ok;} CellExecutor_read_output_result;
    CellExecutor_read_output_result CellExecutor_read_output(diplomat::capi::CellExecutor* self, diplomat::capi::DiplomatStringView name);

    typedef struct CellExecutor_reset_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} CellExecutor_reset_result;
    CellExecutor_reset_result CellExecutor_reset(diplomat::capi::CellExecutor* self);

    void CellExecutor_destroy(CellExecutor* self);

    } // extern "C"
} // namespace capi
} // namespace

inline diplomat::result<std::unique_ptr<CellExecutor>, NucleationError> CellExecutor::for_schematic(const Schematic& schematic) {
    auto result = diplomat::capi::CellExecutor_for_schematic(schematic.AsFFI());
    return result.is_ok ? diplomat::result<std::unique_ptr<CellExecutor>, NucleationError>(diplomat::Ok<std::unique_ptr<CellExecutor>>(std::unique_ptr<CellExecutor>(CellExecutor::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<CellExecutor>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> CellExecutor::set_input(std::string_view name, const Value& value) {
    auto result = diplomat::capi::CellExecutor_set_input(this->AsFFI(),
        {name.data(), name.size()},
        value.AsFFI());
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline bool CellExecutor::settle(uint32_t budget) {
    auto result = diplomat::capi::CellExecutor_settle(this->AsFFI(),
        budget);
    return result;
}

inline diplomat::result<std::unique_ptr<Value>, NucleationError> CellExecutor::read_output(std::string_view name) {
    auto result = diplomat::capi::CellExecutor_read_output(this->AsFFI(),
        {name.data(), name.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<Value>, NucleationError>(diplomat::Ok<std::unique_ptr<Value>>(std::unique_ptr<Value>(Value::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Value>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> CellExecutor::reset() {
    auto result = diplomat::capi::CellExecutor_reset(this->AsFFI());
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline const diplomat::capi::CellExecutor* CellExecutor::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::CellExecutor*>(this);
}

inline diplomat::capi::CellExecutor* CellExecutor::AsFFI() {
    return reinterpret_cast<diplomat::capi::CellExecutor*>(this);
}

inline const CellExecutor* CellExecutor::FromFFI(const diplomat::capi::CellExecutor* ptr) {
    return reinterpret_cast<const CellExecutor*>(ptr);
}

inline CellExecutor* CellExecutor::FromFFI(diplomat::capi::CellExecutor* ptr) {
    return reinterpret_cast<CellExecutor*>(ptr);
}

inline void CellExecutor::operator delete(void* ptr) {
    diplomat::capi::CellExecutor_destroy(reinterpret_cast<diplomat::capi::CellExecutor*>(ptr));
}


#endif // CellExecutor_HPP
