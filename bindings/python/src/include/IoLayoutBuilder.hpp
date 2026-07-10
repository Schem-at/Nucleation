#ifndef NUCLEATION_IoLayoutBuilder_HPP
#define NUCLEATION_IoLayoutBuilder_HPP

#include "IoLayoutBuilder.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "IoLayout.hpp"
#include "IoType.hpp"
#include "LayoutFunction.hpp"
#include "NucleationError.hpp"
#include "diplomat_runtime.hpp"


namespace nucleation {
namespace capi {
    extern "C" {

    nucleation::capi::IoLayoutBuilder* IoLayoutBuilder_create(void);

    typedef struct IoLayoutBuilder_add_input_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} IoLayoutBuilder_add_input_result;
    IoLayoutBuilder_add_input_result IoLayoutBuilder_add_input(nucleation::capi::IoLayoutBuilder* self, nucleation::diplomat::capi::DiplomatStringView name, const nucleation::capi::IoType* io_type, const nucleation::capi::LayoutFunction* layout, nucleation::diplomat::capi::DiplomatI32View positions);

    typedef struct IoLayoutBuilder_add_output_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} IoLayoutBuilder_add_output_result;
    IoLayoutBuilder_add_output_result IoLayoutBuilder_add_output(nucleation::capi::IoLayoutBuilder* self, nucleation::diplomat::capi::DiplomatStringView name, const nucleation::capi::IoType* io_type, const nucleation::capi::LayoutFunction* layout, nucleation::diplomat::capi::DiplomatI32View positions);

    typedef struct IoLayoutBuilder_add_input_auto_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} IoLayoutBuilder_add_input_auto_result;
    IoLayoutBuilder_add_input_auto_result IoLayoutBuilder_add_input_auto(nucleation::capi::IoLayoutBuilder* self, nucleation::diplomat::capi::DiplomatStringView name, const nucleation::capi::IoType* io_type, nucleation::diplomat::capi::DiplomatI32View positions);

    typedef struct IoLayoutBuilder_add_output_auto_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} IoLayoutBuilder_add_output_auto_result;
    IoLayoutBuilder_add_output_auto_result IoLayoutBuilder_add_output_auto(nucleation::capi::IoLayoutBuilder* self, nucleation::diplomat::capi::DiplomatStringView name, const nucleation::capi::IoType* io_type, nucleation::diplomat::capi::DiplomatI32View positions);

    typedef struct IoLayoutBuilder_add_input_from_region_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} IoLayoutBuilder_add_input_from_region_result;
    IoLayoutBuilder_add_input_from_region_result IoLayoutBuilder_add_input_from_region(nucleation::capi::IoLayoutBuilder* self, nucleation::diplomat::capi::DiplomatStringView name, const nucleation::capi::IoType* io_type, const nucleation::capi::LayoutFunction* layout, nucleation::diplomat::capi::DiplomatI32View region_positions);

    typedef struct IoLayoutBuilder_add_input_from_region_auto_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} IoLayoutBuilder_add_input_from_region_auto_result;
    IoLayoutBuilder_add_input_from_region_auto_result IoLayoutBuilder_add_input_from_region_auto(nucleation::capi::IoLayoutBuilder* self, nucleation::diplomat::capi::DiplomatStringView name, const nucleation::capi::IoType* io_type, nucleation::diplomat::capi::DiplomatI32View region_positions);

    typedef struct IoLayoutBuilder_add_output_from_region_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} IoLayoutBuilder_add_output_from_region_result;
    IoLayoutBuilder_add_output_from_region_result IoLayoutBuilder_add_output_from_region(nucleation::capi::IoLayoutBuilder* self, nucleation::diplomat::capi::DiplomatStringView name, const nucleation::capi::IoType* io_type, const nucleation::capi::LayoutFunction* layout, nucleation::diplomat::capi::DiplomatI32View region_positions);

    typedef struct IoLayoutBuilder_add_output_from_region_auto_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} IoLayoutBuilder_add_output_from_region_auto_result;
    IoLayoutBuilder_add_output_from_region_auto_result IoLayoutBuilder_add_output_from_region_auto(nucleation::capi::IoLayoutBuilder* self, nucleation::diplomat::capi::DiplomatStringView name, const nucleation::capi::IoType* io_type, nucleation::diplomat::capi::DiplomatI32View region_positions);

    typedef struct IoLayoutBuilder_build_result {union {nucleation::capi::IoLayout* ok; nucleation::capi::NucleationError err;}; bool is_ok;} IoLayoutBuilder_build_result;
    IoLayoutBuilder_build_result IoLayoutBuilder_build(nucleation::capi::IoLayoutBuilder* self);

    void IoLayoutBuilder_destroy(IoLayoutBuilder* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::unique_ptr<nucleation::IoLayoutBuilder> nucleation::IoLayoutBuilder::create() {
    auto result = nucleation::capi::IoLayoutBuilder_create();
    return std::unique_ptr<nucleation::IoLayoutBuilder>(nucleation::IoLayoutBuilder::FromFFI(result));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::IoLayoutBuilder::add_input(std::string_view name, const nucleation::IoType& io_type, const nucleation::LayoutFunction& layout, nucleation::diplomat::span<const int32_t> positions) {
    auto result = nucleation::capi::IoLayoutBuilder_add_input(this->AsFFI(),
        {name.data(), name.size()},
        io_type.AsFFI(),
        layout.AsFFI(),
        {positions.data(), positions.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::IoLayoutBuilder::add_output(std::string_view name, const nucleation::IoType& io_type, const nucleation::LayoutFunction& layout, nucleation::diplomat::span<const int32_t> positions) {
    auto result = nucleation::capi::IoLayoutBuilder_add_output(this->AsFFI(),
        {name.data(), name.size()},
        io_type.AsFFI(),
        layout.AsFFI(),
        {positions.data(), positions.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::IoLayoutBuilder::add_input_auto(std::string_view name, const nucleation::IoType& io_type, nucleation::diplomat::span<const int32_t> positions) {
    auto result = nucleation::capi::IoLayoutBuilder_add_input_auto(this->AsFFI(),
        {name.data(), name.size()},
        io_type.AsFFI(),
        {positions.data(), positions.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::IoLayoutBuilder::add_output_auto(std::string_view name, const nucleation::IoType& io_type, nucleation::diplomat::span<const int32_t> positions) {
    auto result = nucleation::capi::IoLayoutBuilder_add_output_auto(this->AsFFI(),
        {name.data(), name.size()},
        io_type.AsFFI(),
        {positions.data(), positions.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::IoLayoutBuilder::add_input_from_region(std::string_view name, const nucleation::IoType& io_type, const nucleation::LayoutFunction& layout, nucleation::diplomat::span<const int32_t> region_positions) {
    auto result = nucleation::capi::IoLayoutBuilder_add_input_from_region(this->AsFFI(),
        {name.data(), name.size()},
        io_type.AsFFI(),
        layout.AsFFI(),
        {region_positions.data(), region_positions.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::IoLayoutBuilder::add_input_from_region_auto(std::string_view name, const nucleation::IoType& io_type, nucleation::diplomat::span<const int32_t> region_positions) {
    auto result = nucleation::capi::IoLayoutBuilder_add_input_from_region_auto(this->AsFFI(),
        {name.data(), name.size()},
        io_type.AsFFI(),
        {region_positions.data(), region_positions.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::IoLayoutBuilder::add_output_from_region(std::string_view name, const nucleation::IoType& io_type, const nucleation::LayoutFunction& layout, nucleation::diplomat::span<const int32_t> region_positions) {
    auto result = nucleation::capi::IoLayoutBuilder_add_output_from_region(this->AsFFI(),
        {name.data(), name.size()},
        io_type.AsFFI(),
        layout.AsFFI(),
        {region_positions.data(), region_positions.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::IoLayoutBuilder::add_output_from_region_auto(std::string_view name, const nucleation::IoType& io_type, nucleation::diplomat::span<const int32_t> region_positions) {
    auto result = nucleation::capi::IoLayoutBuilder_add_output_from_region_auto(this->AsFFI(),
        {name.data(), name.size()},
        io_type.AsFFI(),
        {region_positions.data(), region_positions.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::IoLayout>, nucleation::NucleationError> nucleation::IoLayoutBuilder::build() {
    auto result = nucleation::capi::IoLayoutBuilder_build(this->AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::IoLayout>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::IoLayout>>(std::unique_ptr<nucleation::IoLayout>(nucleation::IoLayout::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::IoLayout>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline const nucleation::capi::IoLayoutBuilder* nucleation::IoLayoutBuilder::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::IoLayoutBuilder*>(this);
}

inline nucleation::capi::IoLayoutBuilder* nucleation::IoLayoutBuilder::AsFFI() {
    return reinterpret_cast<nucleation::capi::IoLayoutBuilder*>(this);
}

inline const nucleation::IoLayoutBuilder* nucleation::IoLayoutBuilder::FromFFI(const nucleation::capi::IoLayoutBuilder* ptr) {
    return reinterpret_cast<const nucleation::IoLayoutBuilder*>(ptr);
}

inline nucleation::IoLayoutBuilder* nucleation::IoLayoutBuilder::FromFFI(nucleation::capi::IoLayoutBuilder* ptr) {
    return reinterpret_cast<nucleation::IoLayoutBuilder*>(ptr);
}

inline void nucleation::IoLayoutBuilder::operator delete(void* ptr) {
    nucleation::capi::IoLayoutBuilder_destroy(reinterpret_cast<nucleation::capi::IoLayoutBuilder*>(ptr));
}


#endif // NUCLEATION_IoLayoutBuilder_HPP
