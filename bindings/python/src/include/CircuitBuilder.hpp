#ifndef NUCLEATION_CircuitBuilder_HPP
#define NUCLEATION_CircuitBuilder_HPP

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


namespace nucleation {
namespace capi {
    extern "C" {

    nucleation::capi::CircuitBuilder* CircuitBuilder_create(const nucleation::capi::Schematic* schematic);

    typedef struct CircuitBuilder_from_insign_result {union {nucleation::capi::CircuitBuilder* ok; nucleation::capi::NucleationError err;}; bool is_ok;} CircuitBuilder_from_insign_result;
    CircuitBuilder_from_insign_result CircuitBuilder_from_insign(const nucleation::capi::Schematic* schematic);

    typedef struct CircuitBuilder_with_input_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} CircuitBuilder_with_input_result;
    CircuitBuilder_with_input_result CircuitBuilder_with_input(nucleation::capi::CircuitBuilder* self, nucleation::diplomat::capi::DiplomatStringView name, const nucleation::capi::IoType* io_type, const nucleation::capi::LayoutFunction* layout, nucleation::diplomat::capi::DiplomatI32View region_positions);

    typedef struct CircuitBuilder_with_input_sorted_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} CircuitBuilder_with_input_sorted_result;
    CircuitBuilder_with_input_sorted_result CircuitBuilder_with_input_sorted(nucleation::capi::CircuitBuilder* self, nucleation::diplomat::capi::DiplomatStringView name, const nucleation::capi::IoType* io_type, const nucleation::capi::LayoutFunction* layout, nucleation::diplomat::capi::DiplomatI32View region_positions, const nucleation::capi::SortStrategy* sort);

    typedef struct CircuitBuilder_with_input_auto_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} CircuitBuilder_with_input_auto_result;
    CircuitBuilder_with_input_auto_result CircuitBuilder_with_input_auto(nucleation::capi::CircuitBuilder* self, nucleation::diplomat::capi::DiplomatStringView name, const nucleation::capi::IoType* io_type, nucleation::diplomat::capi::DiplomatI32View region_positions);

    typedef struct CircuitBuilder_with_input_auto_sorted_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} CircuitBuilder_with_input_auto_sorted_result;
    CircuitBuilder_with_input_auto_sorted_result CircuitBuilder_with_input_auto_sorted(nucleation::capi::CircuitBuilder* self, nucleation::diplomat::capi::DiplomatStringView name, const nucleation::capi::IoType* io_type, nucleation::diplomat::capi::DiplomatI32View region_positions, const nucleation::capi::SortStrategy* sort);

    typedef struct CircuitBuilder_with_output_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} CircuitBuilder_with_output_result;
    CircuitBuilder_with_output_result CircuitBuilder_with_output(nucleation::capi::CircuitBuilder* self, nucleation::diplomat::capi::DiplomatStringView name, const nucleation::capi::IoType* io_type, const nucleation::capi::LayoutFunction* layout, nucleation::diplomat::capi::DiplomatI32View region_positions);

    typedef struct CircuitBuilder_with_output_sorted_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} CircuitBuilder_with_output_sorted_result;
    CircuitBuilder_with_output_sorted_result CircuitBuilder_with_output_sorted(nucleation::capi::CircuitBuilder* self, nucleation::diplomat::capi::DiplomatStringView name, const nucleation::capi::IoType* io_type, const nucleation::capi::LayoutFunction* layout, nucleation::diplomat::capi::DiplomatI32View region_positions, const nucleation::capi::SortStrategy* sort);

    typedef struct CircuitBuilder_with_output_auto_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} CircuitBuilder_with_output_auto_result;
    CircuitBuilder_with_output_auto_result CircuitBuilder_with_output_auto(nucleation::capi::CircuitBuilder* self, nucleation::diplomat::capi::DiplomatStringView name, const nucleation::capi::IoType* io_type, nucleation::diplomat::capi::DiplomatI32View region_positions);

    typedef struct CircuitBuilder_with_output_auto_sorted_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} CircuitBuilder_with_output_auto_sorted_result;
    CircuitBuilder_with_output_auto_sorted_result CircuitBuilder_with_output_auto_sorted(nucleation::capi::CircuitBuilder* self, nucleation::diplomat::capi::DiplomatStringView name, const nucleation::capi::IoType* io_type, nucleation::diplomat::capi::DiplomatI32View region_positions, const nucleation::capi::SortStrategy* sort);

    typedef struct CircuitBuilder_with_options_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} CircuitBuilder_with_options_result;
    CircuitBuilder_with_options_result CircuitBuilder_with_options(nucleation::capi::CircuitBuilder* self, bool optimize, bool io_only);

    typedef struct CircuitBuilder_with_state_mode_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} CircuitBuilder_with_state_mode_result;
    CircuitBuilder_with_state_mode_result CircuitBuilder_with_state_mode(nucleation::capi::CircuitBuilder* self, nucleation::diplomat::capi::DiplomatStringView mode);

    typedef struct CircuitBuilder_validate_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} CircuitBuilder_validate_result;
    CircuitBuilder_validate_result CircuitBuilder_validate(const nucleation::capi::CircuitBuilder* self);

    typedef struct CircuitBuilder_build_result {union {nucleation::capi::TypedCircuitExecutor* ok; nucleation::capi::NucleationError err;}; bool is_ok;} CircuitBuilder_build_result;
    CircuitBuilder_build_result CircuitBuilder_build(nucleation::capi::CircuitBuilder* self);

    typedef struct CircuitBuilder_build_validated_result {union {nucleation::capi::TypedCircuitExecutor* ok; nucleation::capi::NucleationError err;}; bool is_ok;} CircuitBuilder_build_validated_result;
    CircuitBuilder_build_validated_result CircuitBuilder_build_validated(nucleation::capi::CircuitBuilder* self);

    uint32_t CircuitBuilder_input_count(const nucleation::capi::CircuitBuilder* self);

    uint32_t CircuitBuilder_output_count(const nucleation::capi::CircuitBuilder* self);

    typedef struct CircuitBuilder_input_names_json_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} CircuitBuilder_input_names_json_result;
    CircuitBuilder_input_names_json_result CircuitBuilder_input_names_json(const nucleation::capi::CircuitBuilder* self, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct CircuitBuilder_output_names_json_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} CircuitBuilder_output_names_json_result;
    CircuitBuilder_output_names_json_result CircuitBuilder_output_names_json(const nucleation::capi::CircuitBuilder* self, nucleation::diplomat::capi::DiplomatWrite* write);

    void CircuitBuilder_destroy(CircuitBuilder* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::unique_ptr<nucleation::CircuitBuilder> nucleation::CircuitBuilder::create(const nucleation::Schematic& schematic) {
    auto result = nucleation::capi::CircuitBuilder_create(schematic.AsFFI());
    return std::unique_ptr<nucleation::CircuitBuilder>(nucleation::CircuitBuilder::FromFFI(result));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::CircuitBuilder>, nucleation::NucleationError> nucleation::CircuitBuilder::from_insign(const nucleation::Schematic& schematic) {
    auto result = nucleation::capi::CircuitBuilder_from_insign(schematic.AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::CircuitBuilder>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::CircuitBuilder>>(std::unique_ptr<nucleation::CircuitBuilder>(nucleation::CircuitBuilder::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::CircuitBuilder>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::CircuitBuilder::with_input(std::string_view name, const nucleation::IoType& io_type, const nucleation::LayoutFunction& layout, nucleation::diplomat::span<const int32_t> region_positions) {
    auto result = nucleation::capi::CircuitBuilder_with_input(this->AsFFI(),
        {name.data(), name.size()},
        io_type.AsFFI(),
        layout.AsFFI(),
        {region_positions.data(), region_positions.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::CircuitBuilder::with_input_sorted(std::string_view name, const nucleation::IoType& io_type, const nucleation::LayoutFunction& layout, nucleation::diplomat::span<const int32_t> region_positions, const nucleation::SortStrategy& sort) {
    auto result = nucleation::capi::CircuitBuilder_with_input_sorted(this->AsFFI(),
        {name.data(), name.size()},
        io_type.AsFFI(),
        layout.AsFFI(),
        {region_positions.data(), region_positions.size()},
        sort.AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::CircuitBuilder::with_input_auto(std::string_view name, const nucleation::IoType& io_type, nucleation::diplomat::span<const int32_t> region_positions) {
    auto result = nucleation::capi::CircuitBuilder_with_input_auto(this->AsFFI(),
        {name.data(), name.size()},
        io_type.AsFFI(),
        {region_positions.data(), region_positions.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::CircuitBuilder::with_input_auto_sorted(std::string_view name, const nucleation::IoType& io_type, nucleation::diplomat::span<const int32_t> region_positions, const nucleation::SortStrategy& sort) {
    auto result = nucleation::capi::CircuitBuilder_with_input_auto_sorted(this->AsFFI(),
        {name.data(), name.size()},
        io_type.AsFFI(),
        {region_positions.data(), region_positions.size()},
        sort.AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::CircuitBuilder::with_output(std::string_view name, const nucleation::IoType& io_type, const nucleation::LayoutFunction& layout, nucleation::diplomat::span<const int32_t> region_positions) {
    auto result = nucleation::capi::CircuitBuilder_with_output(this->AsFFI(),
        {name.data(), name.size()},
        io_type.AsFFI(),
        layout.AsFFI(),
        {region_positions.data(), region_positions.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::CircuitBuilder::with_output_sorted(std::string_view name, const nucleation::IoType& io_type, const nucleation::LayoutFunction& layout, nucleation::diplomat::span<const int32_t> region_positions, const nucleation::SortStrategy& sort) {
    auto result = nucleation::capi::CircuitBuilder_with_output_sorted(this->AsFFI(),
        {name.data(), name.size()},
        io_type.AsFFI(),
        layout.AsFFI(),
        {region_positions.data(), region_positions.size()},
        sort.AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::CircuitBuilder::with_output_auto(std::string_view name, const nucleation::IoType& io_type, nucleation::diplomat::span<const int32_t> region_positions) {
    auto result = nucleation::capi::CircuitBuilder_with_output_auto(this->AsFFI(),
        {name.data(), name.size()},
        io_type.AsFFI(),
        {region_positions.data(), region_positions.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::CircuitBuilder::with_output_auto_sorted(std::string_view name, const nucleation::IoType& io_type, nucleation::diplomat::span<const int32_t> region_positions, const nucleation::SortStrategy& sort) {
    auto result = nucleation::capi::CircuitBuilder_with_output_auto_sorted(this->AsFFI(),
        {name.data(), name.size()},
        io_type.AsFFI(),
        {region_positions.data(), region_positions.size()},
        sort.AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::CircuitBuilder::with_options(bool optimize, bool io_only) {
    auto result = nucleation::capi::CircuitBuilder_with_options(this->AsFFI(),
        optimize,
        io_only);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::CircuitBuilder::with_state_mode(std::string_view mode) {
    auto result = nucleation::capi::CircuitBuilder_with_state_mode(this->AsFFI(),
        {mode.data(), mode.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::CircuitBuilder::validate() const {
    auto result = nucleation::capi::CircuitBuilder_validate(this->AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::TypedCircuitExecutor>, nucleation::NucleationError> nucleation::CircuitBuilder::build() {
    auto result = nucleation::capi::CircuitBuilder_build(this->AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::TypedCircuitExecutor>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::TypedCircuitExecutor>>(std::unique_ptr<nucleation::TypedCircuitExecutor>(nucleation::TypedCircuitExecutor::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::TypedCircuitExecutor>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::TypedCircuitExecutor>, nucleation::NucleationError> nucleation::CircuitBuilder::build_validated() {
    auto result = nucleation::capi::CircuitBuilder_build_validated(this->AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::TypedCircuitExecutor>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::TypedCircuitExecutor>>(std::unique_ptr<nucleation::TypedCircuitExecutor>(nucleation::TypedCircuitExecutor::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::TypedCircuitExecutor>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline uint32_t nucleation::CircuitBuilder::input_count() const {
    auto result = nucleation::capi::CircuitBuilder_input_count(this->AsFFI());
    return result;
}

inline uint32_t nucleation::CircuitBuilder::output_count() const {
    auto result = nucleation::capi::CircuitBuilder_output_count(this->AsFFI());
    return result;
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::CircuitBuilder::input_names_json() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::CircuitBuilder_input_names_json(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::CircuitBuilder::input_names_json_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::CircuitBuilder_input_names_json(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::CircuitBuilder::output_names_json() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::CircuitBuilder_output_names_json(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::CircuitBuilder::output_names_json_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::CircuitBuilder_output_names_json(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline const nucleation::capi::CircuitBuilder* nucleation::CircuitBuilder::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::CircuitBuilder*>(this);
}

inline nucleation::capi::CircuitBuilder* nucleation::CircuitBuilder::AsFFI() {
    return reinterpret_cast<nucleation::capi::CircuitBuilder*>(this);
}

inline const nucleation::CircuitBuilder* nucleation::CircuitBuilder::FromFFI(const nucleation::capi::CircuitBuilder* ptr) {
    return reinterpret_cast<const nucleation::CircuitBuilder*>(ptr);
}

inline nucleation::CircuitBuilder* nucleation::CircuitBuilder::FromFFI(nucleation::capi::CircuitBuilder* ptr) {
    return reinterpret_cast<nucleation::CircuitBuilder*>(ptr);
}

inline void nucleation::CircuitBuilder::operator delete(void* ptr) {
    nucleation::capi::CircuitBuilder_destroy(reinterpret_cast<nucleation::capi::CircuitBuilder*>(ptr));
}


#endif // NUCLEATION_CircuitBuilder_HPP
