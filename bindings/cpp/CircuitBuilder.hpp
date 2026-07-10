#ifndef CircuitBuilder_HPP
#define CircuitBuilder_HPP

#include "CircuitBuilder.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "IoType.hpp"
#include "LayoutFunction.hpp"
#include "NucleationError.hpp"
#include "Schematic.hpp"
#include "SortStrategy.hpp"
#include "TypedCircuitExecutor.hpp"
#include "diplomat_runtime.hpp"


namespace diplomat {
namespace capi {
    extern "C" {

    diplomat::capi::CircuitBuilder* CircuitBuilder_create(const diplomat::capi::Schematic* schematic);

    typedef struct CircuitBuilder_from_insign_result {union {diplomat::capi::CircuitBuilder* ok; diplomat::capi::NucleationError err;}; bool is_ok;} CircuitBuilder_from_insign_result;
    CircuitBuilder_from_insign_result CircuitBuilder_from_insign(const diplomat::capi::Schematic* schematic);

    typedef struct CircuitBuilder_with_input_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} CircuitBuilder_with_input_result;
    CircuitBuilder_with_input_result CircuitBuilder_with_input(diplomat::capi::CircuitBuilder* self, diplomat::capi::DiplomatStringView name, const diplomat::capi::IoType* io_type, const diplomat::capi::LayoutFunction* layout, diplomat::capi::DiplomatI32View region_positions);

    typedef struct CircuitBuilder_with_input_sorted_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} CircuitBuilder_with_input_sorted_result;
    CircuitBuilder_with_input_sorted_result CircuitBuilder_with_input_sorted(diplomat::capi::CircuitBuilder* self, diplomat::capi::DiplomatStringView name, const diplomat::capi::IoType* io_type, const diplomat::capi::LayoutFunction* layout, diplomat::capi::DiplomatI32View region_positions, const diplomat::capi::SortStrategy* sort);

    typedef struct CircuitBuilder_with_input_auto_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} CircuitBuilder_with_input_auto_result;
    CircuitBuilder_with_input_auto_result CircuitBuilder_with_input_auto(diplomat::capi::CircuitBuilder* self, diplomat::capi::DiplomatStringView name, const diplomat::capi::IoType* io_type, diplomat::capi::DiplomatI32View region_positions);

    typedef struct CircuitBuilder_with_input_auto_sorted_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} CircuitBuilder_with_input_auto_sorted_result;
    CircuitBuilder_with_input_auto_sorted_result CircuitBuilder_with_input_auto_sorted(diplomat::capi::CircuitBuilder* self, diplomat::capi::DiplomatStringView name, const diplomat::capi::IoType* io_type, diplomat::capi::DiplomatI32View region_positions, const diplomat::capi::SortStrategy* sort);

    typedef struct CircuitBuilder_with_output_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} CircuitBuilder_with_output_result;
    CircuitBuilder_with_output_result CircuitBuilder_with_output(diplomat::capi::CircuitBuilder* self, diplomat::capi::DiplomatStringView name, const diplomat::capi::IoType* io_type, const diplomat::capi::LayoutFunction* layout, diplomat::capi::DiplomatI32View region_positions);

    typedef struct CircuitBuilder_with_output_sorted_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} CircuitBuilder_with_output_sorted_result;
    CircuitBuilder_with_output_sorted_result CircuitBuilder_with_output_sorted(diplomat::capi::CircuitBuilder* self, diplomat::capi::DiplomatStringView name, const diplomat::capi::IoType* io_type, const diplomat::capi::LayoutFunction* layout, diplomat::capi::DiplomatI32View region_positions, const diplomat::capi::SortStrategy* sort);

    typedef struct CircuitBuilder_with_output_auto_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} CircuitBuilder_with_output_auto_result;
    CircuitBuilder_with_output_auto_result CircuitBuilder_with_output_auto(diplomat::capi::CircuitBuilder* self, diplomat::capi::DiplomatStringView name, const diplomat::capi::IoType* io_type, diplomat::capi::DiplomatI32View region_positions);

    typedef struct CircuitBuilder_with_output_auto_sorted_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} CircuitBuilder_with_output_auto_sorted_result;
    CircuitBuilder_with_output_auto_sorted_result CircuitBuilder_with_output_auto_sorted(diplomat::capi::CircuitBuilder* self, diplomat::capi::DiplomatStringView name, const diplomat::capi::IoType* io_type, diplomat::capi::DiplomatI32View region_positions, const diplomat::capi::SortStrategy* sort);

    typedef struct CircuitBuilder_with_options_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} CircuitBuilder_with_options_result;
    CircuitBuilder_with_options_result CircuitBuilder_with_options(diplomat::capi::CircuitBuilder* self, bool optimize, bool io_only);

    typedef struct CircuitBuilder_with_state_mode_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} CircuitBuilder_with_state_mode_result;
    CircuitBuilder_with_state_mode_result CircuitBuilder_with_state_mode(diplomat::capi::CircuitBuilder* self, diplomat::capi::DiplomatStringView mode);

    typedef struct CircuitBuilder_validate_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} CircuitBuilder_validate_result;
    CircuitBuilder_validate_result CircuitBuilder_validate(const diplomat::capi::CircuitBuilder* self);

    typedef struct CircuitBuilder_build_result {union {diplomat::capi::TypedCircuitExecutor* ok; diplomat::capi::NucleationError err;}; bool is_ok;} CircuitBuilder_build_result;
    CircuitBuilder_build_result CircuitBuilder_build(diplomat::capi::CircuitBuilder* self);

    typedef struct CircuitBuilder_build_validated_result {union {diplomat::capi::TypedCircuitExecutor* ok; diplomat::capi::NucleationError err;}; bool is_ok;} CircuitBuilder_build_validated_result;
    CircuitBuilder_build_validated_result CircuitBuilder_build_validated(diplomat::capi::CircuitBuilder* self);

    uint32_t CircuitBuilder_input_count(const diplomat::capi::CircuitBuilder* self);

    uint32_t CircuitBuilder_output_count(const diplomat::capi::CircuitBuilder* self);

    typedef struct CircuitBuilder_input_names_json_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} CircuitBuilder_input_names_json_result;
    CircuitBuilder_input_names_json_result CircuitBuilder_input_names_json(const diplomat::capi::CircuitBuilder* self, diplomat::capi::DiplomatWrite* write);

    typedef struct CircuitBuilder_output_names_json_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} CircuitBuilder_output_names_json_result;
    CircuitBuilder_output_names_json_result CircuitBuilder_output_names_json(const diplomat::capi::CircuitBuilder* self, diplomat::capi::DiplomatWrite* write);

    void CircuitBuilder_destroy(CircuitBuilder* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::unique_ptr<CircuitBuilder> CircuitBuilder::create(const Schematic& schematic) {
    auto result = diplomat::capi::CircuitBuilder_create(schematic.AsFFI());
    return std::unique_ptr<CircuitBuilder>(CircuitBuilder::FromFFI(result));
}

inline diplomat::result<std::unique_ptr<CircuitBuilder>, NucleationError> CircuitBuilder::from_insign(const Schematic& schematic) {
    auto result = diplomat::capi::CircuitBuilder_from_insign(schematic.AsFFI());
    return result.is_ok ? diplomat::result<std::unique_ptr<CircuitBuilder>, NucleationError>(diplomat::Ok<std::unique_ptr<CircuitBuilder>>(std::unique_ptr<CircuitBuilder>(CircuitBuilder::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<CircuitBuilder>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> CircuitBuilder::with_input(std::string_view name, const IoType& io_type, const LayoutFunction& layout, diplomat::span<const int32_t> region_positions) {
    auto result = diplomat::capi::CircuitBuilder_with_input(this->AsFFI(),
        {name.data(), name.size()},
        io_type.AsFFI(),
        layout.AsFFI(),
        {region_positions.data(), region_positions.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> CircuitBuilder::with_input_sorted(std::string_view name, const IoType& io_type, const LayoutFunction& layout, diplomat::span<const int32_t> region_positions, const SortStrategy& sort) {
    auto result = diplomat::capi::CircuitBuilder_with_input_sorted(this->AsFFI(),
        {name.data(), name.size()},
        io_type.AsFFI(),
        layout.AsFFI(),
        {region_positions.data(), region_positions.size()},
        sort.AsFFI());
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> CircuitBuilder::with_input_auto(std::string_view name, const IoType& io_type, diplomat::span<const int32_t> region_positions) {
    auto result = diplomat::capi::CircuitBuilder_with_input_auto(this->AsFFI(),
        {name.data(), name.size()},
        io_type.AsFFI(),
        {region_positions.data(), region_positions.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> CircuitBuilder::with_input_auto_sorted(std::string_view name, const IoType& io_type, diplomat::span<const int32_t> region_positions, const SortStrategy& sort) {
    auto result = diplomat::capi::CircuitBuilder_with_input_auto_sorted(this->AsFFI(),
        {name.data(), name.size()},
        io_type.AsFFI(),
        {region_positions.data(), region_positions.size()},
        sort.AsFFI());
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> CircuitBuilder::with_output(std::string_view name, const IoType& io_type, const LayoutFunction& layout, diplomat::span<const int32_t> region_positions) {
    auto result = diplomat::capi::CircuitBuilder_with_output(this->AsFFI(),
        {name.data(), name.size()},
        io_type.AsFFI(),
        layout.AsFFI(),
        {region_positions.data(), region_positions.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> CircuitBuilder::with_output_sorted(std::string_view name, const IoType& io_type, const LayoutFunction& layout, diplomat::span<const int32_t> region_positions, const SortStrategy& sort) {
    auto result = diplomat::capi::CircuitBuilder_with_output_sorted(this->AsFFI(),
        {name.data(), name.size()},
        io_type.AsFFI(),
        layout.AsFFI(),
        {region_positions.data(), region_positions.size()},
        sort.AsFFI());
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> CircuitBuilder::with_output_auto(std::string_view name, const IoType& io_type, diplomat::span<const int32_t> region_positions) {
    auto result = diplomat::capi::CircuitBuilder_with_output_auto(this->AsFFI(),
        {name.data(), name.size()},
        io_type.AsFFI(),
        {region_positions.data(), region_positions.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> CircuitBuilder::with_output_auto_sorted(std::string_view name, const IoType& io_type, diplomat::span<const int32_t> region_positions, const SortStrategy& sort) {
    auto result = diplomat::capi::CircuitBuilder_with_output_auto_sorted(this->AsFFI(),
        {name.data(), name.size()},
        io_type.AsFFI(),
        {region_positions.data(), region_positions.size()},
        sort.AsFFI());
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> CircuitBuilder::with_options(bool optimize, bool io_only) {
    auto result = diplomat::capi::CircuitBuilder_with_options(this->AsFFI(),
        optimize,
        io_only);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> CircuitBuilder::with_state_mode(std::string_view mode) {
    auto result = diplomat::capi::CircuitBuilder_with_state_mode(this->AsFFI(),
        {mode.data(), mode.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> CircuitBuilder::validate() const {
    auto result = diplomat::capi::CircuitBuilder_validate(this->AsFFI());
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<TypedCircuitExecutor>, NucleationError> CircuitBuilder::build() {
    auto result = diplomat::capi::CircuitBuilder_build(this->AsFFI());
    return result.is_ok ? diplomat::result<std::unique_ptr<TypedCircuitExecutor>, NucleationError>(diplomat::Ok<std::unique_ptr<TypedCircuitExecutor>>(std::unique_ptr<TypedCircuitExecutor>(TypedCircuitExecutor::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<TypedCircuitExecutor>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<TypedCircuitExecutor>, NucleationError> CircuitBuilder::build_validated() {
    auto result = diplomat::capi::CircuitBuilder_build_validated(this->AsFFI());
    return result.is_ok ? diplomat::result<std::unique_ptr<TypedCircuitExecutor>, NucleationError>(diplomat::Ok<std::unique_ptr<TypedCircuitExecutor>>(std::unique_ptr<TypedCircuitExecutor>(TypedCircuitExecutor::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<TypedCircuitExecutor>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline uint32_t CircuitBuilder::input_count() const {
    auto result = diplomat::capi::CircuitBuilder_input_count(this->AsFFI());
    return result;
}

inline uint32_t CircuitBuilder::output_count() const {
    auto result = diplomat::capi::CircuitBuilder_output_count(this->AsFFI());
    return result;
}

inline diplomat::result<std::string, NucleationError> CircuitBuilder::input_names_json() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::CircuitBuilder_input_names_json(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> CircuitBuilder::input_names_json_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::CircuitBuilder_input_names_json(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> CircuitBuilder::output_names_json() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::CircuitBuilder_output_names_json(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> CircuitBuilder::output_names_json_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::CircuitBuilder_output_names_json(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline const diplomat::capi::CircuitBuilder* CircuitBuilder::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::CircuitBuilder*>(this);
}

inline diplomat::capi::CircuitBuilder* CircuitBuilder::AsFFI() {
    return reinterpret_cast<diplomat::capi::CircuitBuilder*>(this);
}

inline const CircuitBuilder* CircuitBuilder::FromFFI(const diplomat::capi::CircuitBuilder* ptr) {
    return reinterpret_cast<const CircuitBuilder*>(ptr);
}

inline CircuitBuilder* CircuitBuilder::FromFFI(diplomat::capi::CircuitBuilder* ptr) {
    return reinterpret_cast<CircuitBuilder*>(ptr);
}

inline void CircuitBuilder::operator delete(void* ptr) {
    diplomat::capi::CircuitBuilder_destroy(reinterpret_cast<diplomat::capi::CircuitBuilder*>(ptr));
}


#endif // CircuitBuilder_HPP
