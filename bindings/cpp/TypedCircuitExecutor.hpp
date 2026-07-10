#ifndef TypedCircuitExecutor_HPP
#define TypedCircuitExecutor_HPP

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


namespace diplomat {
namespace capi {
    extern "C" {

    typedef struct TypedCircuitExecutor_from_layout_result {union {diplomat::capi::TypedCircuitExecutor* ok; diplomat::capi::NucleationError err;}; bool is_ok;} TypedCircuitExecutor_from_layout_result;
    TypedCircuitExecutor_from_layout_result TypedCircuitExecutor_from_layout(const diplomat::capi::MchprsWorld* world, const diplomat::capi::IoLayout* layout);

    typedef struct TypedCircuitExecutor_from_layout_with_options_result {union {diplomat::capi::TypedCircuitExecutor* ok; diplomat::capi::NucleationError err;}; bool is_ok;} TypedCircuitExecutor_from_layout_with_options_result;
    TypedCircuitExecutor_from_layout_with_options_result TypedCircuitExecutor_from_layout_with_options(const diplomat::capi::MchprsWorld* world, const diplomat::capi::IoLayout* layout, bool optimize, bool io_only);

    typedef struct TypedCircuitExecutor_from_insign_result {union {diplomat::capi::TypedCircuitExecutor* ok; diplomat::capi::NucleationError err;}; bool is_ok;} TypedCircuitExecutor_from_insign_result;
    TypedCircuitExecutor_from_insign_result TypedCircuitExecutor_from_insign(const diplomat::capi::Schematic* schematic);

    typedef struct TypedCircuitExecutor_from_insign_with_options_result {union {diplomat::capi::TypedCircuitExecutor* ok; diplomat::capi::NucleationError err;}; bool is_ok;} TypedCircuitExecutor_from_insign_with_options_result;
    TypedCircuitExecutor_from_insign_with_options_result TypedCircuitExecutor_from_insign_with_options(const diplomat::capi::Schematic* schematic, bool optimize, bool io_only);

    typedef struct TypedCircuitExecutor_set_state_mode_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} TypedCircuitExecutor_set_state_mode_result;
    TypedCircuitExecutor_set_state_mode_result TypedCircuitExecutor_set_state_mode(diplomat::capi::TypedCircuitExecutor* self, diplomat::capi::DiplomatStringView mode);

    typedef struct TypedCircuitExecutor_reset_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} TypedCircuitExecutor_reset_result;
    TypedCircuitExecutor_reset_result TypedCircuitExecutor_reset(diplomat::capi::TypedCircuitExecutor* self);

    void TypedCircuitExecutor_tick(diplomat::capi::TypedCircuitExecutor* self, uint32_t ticks);

    void TypedCircuitExecutor_flush(diplomat::capi::TypedCircuitExecutor* self);

    typedef struct TypedCircuitExecutor_set_input_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} TypedCircuitExecutor_set_input_result;
    TypedCircuitExecutor_set_input_result TypedCircuitExecutor_set_input(diplomat::capi::TypedCircuitExecutor* self, diplomat::capi::DiplomatStringView name, const diplomat::capi::Value* value);

    typedef struct TypedCircuitExecutor_read_output_result {union {diplomat::capi::Value* ok; diplomat::capi::NucleationError err;}; bool is_ok;} TypedCircuitExecutor_read_output_result;
    TypedCircuitExecutor_read_output_result TypedCircuitExecutor_read_output(diplomat::capi::TypedCircuitExecutor* self, diplomat::capi::DiplomatStringView name);

    typedef struct TypedCircuitExecutor_execute_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} TypedCircuitExecutor_execute_result;
    TypedCircuitExecutor_execute_result TypedCircuitExecutor_execute(diplomat::capi::TypedCircuitExecutor* self, diplomat::capi::DiplomatStringView inputs_json, const diplomat::capi::ExecutionMode* mode, diplomat::capi::DiplomatWrite* write);

    void TypedCircuitExecutor_input_names_json(const diplomat::capi::TypedCircuitExecutor* self, diplomat::capi::DiplomatWrite* write);

    void TypedCircuitExecutor_output_names_json(const diplomat::capi::TypedCircuitExecutor* self, diplomat::capi::DiplomatWrite* write);

    void TypedCircuitExecutor_layout_info_json(const diplomat::capi::TypedCircuitExecutor* self, diplomat::capi::DiplomatWrite* write);

    diplomat::capi::Schematic* TypedCircuitExecutor_sync_to_schematic(diplomat::capi::TypedCircuitExecutor* self);

    void TypedCircuitExecutor_destroy(TypedCircuitExecutor* self);

    } // extern "C"
} // namespace capi
} // namespace

inline diplomat::result<std::unique_ptr<TypedCircuitExecutor>, NucleationError> TypedCircuitExecutor::from_layout(const MchprsWorld& world, const IoLayout& layout) {
    auto result = diplomat::capi::TypedCircuitExecutor_from_layout(world.AsFFI(),
        layout.AsFFI());
    return result.is_ok ? diplomat::result<std::unique_ptr<TypedCircuitExecutor>, NucleationError>(diplomat::Ok<std::unique_ptr<TypedCircuitExecutor>>(std::unique_ptr<TypedCircuitExecutor>(TypedCircuitExecutor::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<TypedCircuitExecutor>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<TypedCircuitExecutor>, NucleationError> TypedCircuitExecutor::from_layout_with_options(const MchprsWorld& world, const IoLayout& layout, bool optimize, bool io_only) {
    auto result = diplomat::capi::TypedCircuitExecutor_from_layout_with_options(world.AsFFI(),
        layout.AsFFI(),
        optimize,
        io_only);
    return result.is_ok ? diplomat::result<std::unique_ptr<TypedCircuitExecutor>, NucleationError>(diplomat::Ok<std::unique_ptr<TypedCircuitExecutor>>(std::unique_ptr<TypedCircuitExecutor>(TypedCircuitExecutor::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<TypedCircuitExecutor>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<TypedCircuitExecutor>, NucleationError> TypedCircuitExecutor::from_insign(const Schematic& schematic) {
    auto result = diplomat::capi::TypedCircuitExecutor_from_insign(schematic.AsFFI());
    return result.is_ok ? diplomat::result<std::unique_ptr<TypedCircuitExecutor>, NucleationError>(diplomat::Ok<std::unique_ptr<TypedCircuitExecutor>>(std::unique_ptr<TypedCircuitExecutor>(TypedCircuitExecutor::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<TypedCircuitExecutor>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<TypedCircuitExecutor>, NucleationError> TypedCircuitExecutor::from_insign_with_options(const Schematic& schematic, bool optimize, bool io_only) {
    auto result = diplomat::capi::TypedCircuitExecutor_from_insign_with_options(schematic.AsFFI(),
        optimize,
        io_only);
    return result.is_ok ? diplomat::result<std::unique_ptr<TypedCircuitExecutor>, NucleationError>(diplomat::Ok<std::unique_ptr<TypedCircuitExecutor>>(std::unique_ptr<TypedCircuitExecutor>(TypedCircuitExecutor::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<TypedCircuitExecutor>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> TypedCircuitExecutor::set_state_mode(std::string_view mode) {
    auto result = diplomat::capi::TypedCircuitExecutor_set_state_mode(this->AsFFI(),
        {mode.data(), mode.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> TypedCircuitExecutor::reset() {
    auto result = diplomat::capi::TypedCircuitExecutor_reset(this->AsFFI());
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline void TypedCircuitExecutor::tick(uint32_t ticks) {
    diplomat::capi::TypedCircuitExecutor_tick(this->AsFFI(),
        ticks);
}

inline void TypedCircuitExecutor::flush() {
    diplomat::capi::TypedCircuitExecutor_flush(this->AsFFI());
}

inline diplomat::result<std::monostate, NucleationError> TypedCircuitExecutor::set_input(std::string_view name, const Value& value) {
    auto result = diplomat::capi::TypedCircuitExecutor_set_input(this->AsFFI(),
        {name.data(), name.size()},
        value.AsFFI());
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Value>, NucleationError> TypedCircuitExecutor::read_output(std::string_view name) {
    auto result = diplomat::capi::TypedCircuitExecutor_read_output(this->AsFFI(),
        {name.data(), name.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<Value>, NucleationError>(diplomat::Ok<std::unique_ptr<Value>>(std::unique_ptr<Value>(Value::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Value>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> TypedCircuitExecutor::execute(std::string_view inputs_json, const ExecutionMode& mode) {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::TypedCircuitExecutor_execute(this->AsFFI(),
        {inputs_json.data(), inputs_json.size()},
        mode.AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> TypedCircuitExecutor::execute_write(std::string_view inputs_json, const ExecutionMode& mode, W& writeable) {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::TypedCircuitExecutor_execute(this->AsFFI(),
        {inputs_json.data(), inputs_json.size()},
        mode.AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline std::string TypedCircuitExecutor::input_names_json() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::TypedCircuitExecutor_input_names_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void TypedCircuitExecutor::input_names_json_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::TypedCircuitExecutor_input_names_json(this->AsFFI(),
        &write);
}

inline std::string TypedCircuitExecutor::output_names_json() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::TypedCircuitExecutor_output_names_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void TypedCircuitExecutor::output_names_json_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::TypedCircuitExecutor_output_names_json(this->AsFFI(),
        &write);
}

inline std::string TypedCircuitExecutor::layout_info_json() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::TypedCircuitExecutor_layout_info_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void TypedCircuitExecutor::layout_info_json_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::TypedCircuitExecutor_layout_info_json(this->AsFFI(),
        &write);
}

inline std::unique_ptr<Schematic> TypedCircuitExecutor::sync_to_schematic() {
    auto result = diplomat::capi::TypedCircuitExecutor_sync_to_schematic(this->AsFFI());
    return std::unique_ptr<Schematic>(Schematic::FromFFI(result));
}

inline const diplomat::capi::TypedCircuitExecutor* TypedCircuitExecutor::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::TypedCircuitExecutor*>(this);
}

inline diplomat::capi::TypedCircuitExecutor* TypedCircuitExecutor::AsFFI() {
    return reinterpret_cast<diplomat::capi::TypedCircuitExecutor*>(this);
}

inline const TypedCircuitExecutor* TypedCircuitExecutor::FromFFI(const diplomat::capi::TypedCircuitExecutor* ptr) {
    return reinterpret_cast<const TypedCircuitExecutor*>(ptr);
}

inline TypedCircuitExecutor* TypedCircuitExecutor::FromFFI(diplomat::capi::TypedCircuitExecutor* ptr) {
    return reinterpret_cast<TypedCircuitExecutor*>(ptr);
}

inline void TypedCircuitExecutor::operator delete(void* ptr) {
    diplomat::capi::TypedCircuitExecutor_destroy(reinterpret_cast<diplomat::capi::TypedCircuitExecutor*>(ptr));
}


#endif // TypedCircuitExecutor_HPP
