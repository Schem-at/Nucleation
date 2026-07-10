#ifndef NUCLEATION_TypedCircuitExecutor_HPP
#define NUCLEATION_TypedCircuitExecutor_HPP

#include "TypedCircuitExecutor.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "ExecutionMode.hpp"
#include "IoLayout.hpp"
#include "MchprsWorld.hpp"
#include "NucleationError.hpp"
#include "Schematic.hpp"
#include "Value.hpp"
#include "diplomat_runtime.hpp"


namespace nucleation {
namespace capi {
    extern "C" {

    typedef struct TypedCircuitExecutor_from_layout_result {union {nucleation::capi::TypedCircuitExecutor* ok; nucleation::capi::NucleationError err;}; bool is_ok;} TypedCircuitExecutor_from_layout_result;
    TypedCircuitExecutor_from_layout_result TypedCircuitExecutor_from_layout(const nucleation::capi::MchprsWorld* world, const nucleation::capi::IoLayout* layout);

    typedef struct TypedCircuitExecutor_from_layout_with_options_result {union {nucleation::capi::TypedCircuitExecutor* ok; nucleation::capi::NucleationError err;}; bool is_ok;} TypedCircuitExecutor_from_layout_with_options_result;
    TypedCircuitExecutor_from_layout_with_options_result TypedCircuitExecutor_from_layout_with_options(const nucleation::capi::MchprsWorld* world, const nucleation::capi::IoLayout* layout, bool optimize, bool io_only);

    typedef struct TypedCircuitExecutor_from_insign_result {union {nucleation::capi::TypedCircuitExecutor* ok; nucleation::capi::NucleationError err;}; bool is_ok;} TypedCircuitExecutor_from_insign_result;
    TypedCircuitExecutor_from_insign_result TypedCircuitExecutor_from_insign(const nucleation::capi::Schematic* schematic);

    typedef struct TypedCircuitExecutor_from_insign_with_options_result {union {nucleation::capi::TypedCircuitExecutor* ok; nucleation::capi::NucleationError err;}; bool is_ok;} TypedCircuitExecutor_from_insign_with_options_result;
    TypedCircuitExecutor_from_insign_with_options_result TypedCircuitExecutor_from_insign_with_options(const nucleation::capi::Schematic* schematic, bool optimize, bool io_only);

    typedef struct TypedCircuitExecutor_set_state_mode_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} TypedCircuitExecutor_set_state_mode_result;
    TypedCircuitExecutor_set_state_mode_result TypedCircuitExecutor_set_state_mode(nucleation::capi::TypedCircuitExecutor* self, nucleation::diplomat::capi::DiplomatStringView mode);

    typedef struct TypedCircuitExecutor_reset_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} TypedCircuitExecutor_reset_result;
    TypedCircuitExecutor_reset_result TypedCircuitExecutor_reset(nucleation::capi::TypedCircuitExecutor* self);

    void TypedCircuitExecutor_tick(nucleation::capi::TypedCircuitExecutor* self, uint32_t ticks);

    void TypedCircuitExecutor_flush(nucleation::capi::TypedCircuitExecutor* self);

    typedef struct TypedCircuitExecutor_set_input_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} TypedCircuitExecutor_set_input_result;
    TypedCircuitExecutor_set_input_result TypedCircuitExecutor_set_input(nucleation::capi::TypedCircuitExecutor* self, nucleation::diplomat::capi::DiplomatStringView name, const nucleation::capi::Value* value);

    typedef struct TypedCircuitExecutor_read_output_result {union {nucleation::capi::Value* ok; nucleation::capi::NucleationError err;}; bool is_ok;} TypedCircuitExecutor_read_output_result;
    TypedCircuitExecutor_read_output_result TypedCircuitExecutor_read_output(nucleation::capi::TypedCircuitExecutor* self, nucleation::diplomat::capi::DiplomatStringView name);

    typedef struct TypedCircuitExecutor_execute_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} TypedCircuitExecutor_execute_result;
    TypedCircuitExecutor_execute_result TypedCircuitExecutor_execute(nucleation::capi::TypedCircuitExecutor* self, nucleation::diplomat::capi::DiplomatStringView inputs_json, const nucleation::capi::ExecutionMode* mode, nucleation::diplomat::capi::DiplomatWrite* write);

    void TypedCircuitExecutor_input_names_json(const nucleation::capi::TypedCircuitExecutor* self, nucleation::diplomat::capi::DiplomatWrite* write);

    void TypedCircuitExecutor_output_names_json(const nucleation::capi::TypedCircuitExecutor* self, nucleation::diplomat::capi::DiplomatWrite* write);

    void TypedCircuitExecutor_layout_info_json(const nucleation::capi::TypedCircuitExecutor* self, nucleation::diplomat::capi::DiplomatWrite* write);

    nucleation::capi::Schematic* TypedCircuitExecutor_sync_to_schematic(nucleation::capi::TypedCircuitExecutor* self);

    void TypedCircuitExecutor_destroy(TypedCircuitExecutor* self);

    } // extern "C"
} // namespace capi
} // namespace

inline nucleation::diplomat::result<std::unique_ptr<nucleation::TypedCircuitExecutor>, nucleation::NucleationError> nucleation::TypedCircuitExecutor::from_layout(const nucleation::MchprsWorld& world, const nucleation::IoLayout& layout) {
    auto result = nucleation::capi::TypedCircuitExecutor_from_layout(world.AsFFI(),
        layout.AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::TypedCircuitExecutor>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::TypedCircuitExecutor>>(std::unique_ptr<nucleation::TypedCircuitExecutor>(nucleation::TypedCircuitExecutor::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::TypedCircuitExecutor>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::TypedCircuitExecutor>, nucleation::NucleationError> nucleation::TypedCircuitExecutor::from_layout_with_options(const nucleation::MchprsWorld& world, const nucleation::IoLayout& layout, bool optimize, bool io_only) {
    auto result = nucleation::capi::TypedCircuitExecutor_from_layout_with_options(world.AsFFI(),
        layout.AsFFI(),
        optimize,
        io_only);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::TypedCircuitExecutor>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::TypedCircuitExecutor>>(std::unique_ptr<nucleation::TypedCircuitExecutor>(nucleation::TypedCircuitExecutor::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::TypedCircuitExecutor>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::TypedCircuitExecutor>, nucleation::NucleationError> nucleation::TypedCircuitExecutor::from_insign(const nucleation::Schematic& schematic) {
    auto result = nucleation::capi::TypedCircuitExecutor_from_insign(schematic.AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::TypedCircuitExecutor>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::TypedCircuitExecutor>>(std::unique_ptr<nucleation::TypedCircuitExecutor>(nucleation::TypedCircuitExecutor::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::TypedCircuitExecutor>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::TypedCircuitExecutor>, nucleation::NucleationError> nucleation::TypedCircuitExecutor::from_insign_with_options(const nucleation::Schematic& schematic, bool optimize, bool io_only) {
    auto result = nucleation::capi::TypedCircuitExecutor_from_insign_with_options(schematic.AsFFI(),
        optimize,
        io_only);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::TypedCircuitExecutor>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::TypedCircuitExecutor>>(std::unique_ptr<nucleation::TypedCircuitExecutor>(nucleation::TypedCircuitExecutor::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::TypedCircuitExecutor>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::TypedCircuitExecutor::set_state_mode(std::string_view mode) {
    auto result = nucleation::capi::TypedCircuitExecutor_set_state_mode(this->AsFFI(),
        {mode.data(), mode.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::TypedCircuitExecutor::reset() {
    auto result = nucleation::capi::TypedCircuitExecutor_reset(this->AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline void nucleation::TypedCircuitExecutor::tick(uint32_t ticks) {
    nucleation::capi::TypedCircuitExecutor_tick(this->AsFFI(),
        ticks);
}

inline void nucleation::TypedCircuitExecutor::flush() {
    nucleation::capi::TypedCircuitExecutor_flush(this->AsFFI());
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::TypedCircuitExecutor::set_input(std::string_view name, const nucleation::Value& value) {
    auto result = nucleation::capi::TypedCircuitExecutor_set_input(this->AsFFI(),
        {name.data(), name.size()},
        value.AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Value>, nucleation::NucleationError> nucleation::TypedCircuitExecutor::read_output(std::string_view name) {
    auto result = nucleation::capi::TypedCircuitExecutor_read_output(this->AsFFI(),
        {name.data(), name.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Value>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Value>>(std::unique_ptr<nucleation::Value>(nucleation::Value::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Value>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::TypedCircuitExecutor::execute(std::string_view inputs_json, const nucleation::ExecutionMode& mode) {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::TypedCircuitExecutor_execute(this->AsFFI(),
        {inputs_json.data(), inputs_json.size()},
        mode.AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::TypedCircuitExecutor::execute_write(std::string_view inputs_json, const nucleation::ExecutionMode& mode, W& writeable) {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::TypedCircuitExecutor_execute(this->AsFFI(),
        {inputs_json.data(), inputs_json.size()},
        mode.AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline std::string nucleation::TypedCircuitExecutor::input_names_json() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::TypedCircuitExecutor_input_names_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void nucleation::TypedCircuitExecutor::input_names_json_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::TypedCircuitExecutor_input_names_json(this->AsFFI(),
        &write);
}

inline std::string nucleation::TypedCircuitExecutor::output_names_json() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::TypedCircuitExecutor_output_names_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void nucleation::TypedCircuitExecutor::output_names_json_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::TypedCircuitExecutor_output_names_json(this->AsFFI(),
        &write);
}

inline std::string nucleation::TypedCircuitExecutor::layout_info_json() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::TypedCircuitExecutor_layout_info_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void nucleation::TypedCircuitExecutor::layout_info_json_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::TypedCircuitExecutor_layout_info_json(this->AsFFI(),
        &write);
}

inline std::unique_ptr<nucleation::Schematic> nucleation::TypedCircuitExecutor::sync_to_schematic() {
    auto result = nucleation::capi::TypedCircuitExecutor_sync_to_schematic(this->AsFFI());
    return std::unique_ptr<nucleation::Schematic>(nucleation::Schematic::FromFFI(result));
}

inline const nucleation::capi::TypedCircuitExecutor* nucleation::TypedCircuitExecutor::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::TypedCircuitExecutor*>(this);
}

inline nucleation::capi::TypedCircuitExecutor* nucleation::TypedCircuitExecutor::AsFFI() {
    return reinterpret_cast<nucleation::capi::TypedCircuitExecutor*>(this);
}

inline const nucleation::TypedCircuitExecutor* nucleation::TypedCircuitExecutor::FromFFI(const nucleation::capi::TypedCircuitExecutor* ptr) {
    return reinterpret_cast<const nucleation::TypedCircuitExecutor*>(ptr);
}

inline nucleation::TypedCircuitExecutor* nucleation::TypedCircuitExecutor::FromFFI(nucleation::capi::TypedCircuitExecutor* ptr) {
    return reinterpret_cast<nucleation::TypedCircuitExecutor*>(ptr);
}

inline void nucleation::TypedCircuitExecutor::operator delete(void* ptr) {
    nucleation::capi::TypedCircuitExecutor_destroy(reinterpret_cast<nucleation::capi::TypedCircuitExecutor*>(ptr));
}


#endif // NUCLEATION_TypedCircuitExecutor_HPP
