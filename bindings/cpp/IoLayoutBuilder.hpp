#ifndef IoLayoutBuilder_HPP
#define IoLayoutBuilder_HPP

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


namespace diplomat {
namespace capi {
    extern "C" {

    diplomat::capi::IoLayoutBuilder* IoLayoutBuilder_create(void);

    typedef struct IoLayoutBuilder_add_input_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} IoLayoutBuilder_add_input_result;
    IoLayoutBuilder_add_input_result IoLayoutBuilder_add_input(diplomat::capi::IoLayoutBuilder* self, diplomat::capi::DiplomatStringView name, const diplomat::capi::IoType* io_type, const diplomat::capi::LayoutFunction* layout, diplomat::capi::DiplomatI32View positions);

    typedef struct IoLayoutBuilder_add_output_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} IoLayoutBuilder_add_output_result;
    IoLayoutBuilder_add_output_result IoLayoutBuilder_add_output(diplomat::capi::IoLayoutBuilder* self, diplomat::capi::DiplomatStringView name, const diplomat::capi::IoType* io_type, const diplomat::capi::LayoutFunction* layout, diplomat::capi::DiplomatI32View positions);

    typedef struct IoLayoutBuilder_add_input_auto_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} IoLayoutBuilder_add_input_auto_result;
    IoLayoutBuilder_add_input_auto_result IoLayoutBuilder_add_input_auto(diplomat::capi::IoLayoutBuilder* self, diplomat::capi::DiplomatStringView name, const diplomat::capi::IoType* io_type, diplomat::capi::DiplomatI32View positions);

    typedef struct IoLayoutBuilder_add_output_auto_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} IoLayoutBuilder_add_output_auto_result;
    IoLayoutBuilder_add_output_auto_result IoLayoutBuilder_add_output_auto(diplomat::capi::IoLayoutBuilder* self, diplomat::capi::DiplomatStringView name, const diplomat::capi::IoType* io_type, diplomat::capi::DiplomatI32View positions);

    typedef struct IoLayoutBuilder_add_input_from_region_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} IoLayoutBuilder_add_input_from_region_result;
    IoLayoutBuilder_add_input_from_region_result IoLayoutBuilder_add_input_from_region(diplomat::capi::IoLayoutBuilder* self, diplomat::capi::DiplomatStringView name, const diplomat::capi::IoType* io_type, const diplomat::capi::LayoutFunction* layout, diplomat::capi::DiplomatI32View region_positions);

    typedef struct IoLayoutBuilder_add_input_from_region_auto_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} IoLayoutBuilder_add_input_from_region_auto_result;
    IoLayoutBuilder_add_input_from_region_auto_result IoLayoutBuilder_add_input_from_region_auto(diplomat::capi::IoLayoutBuilder* self, diplomat::capi::DiplomatStringView name, const diplomat::capi::IoType* io_type, diplomat::capi::DiplomatI32View region_positions);

    typedef struct IoLayoutBuilder_add_output_from_region_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} IoLayoutBuilder_add_output_from_region_result;
    IoLayoutBuilder_add_output_from_region_result IoLayoutBuilder_add_output_from_region(diplomat::capi::IoLayoutBuilder* self, diplomat::capi::DiplomatStringView name, const diplomat::capi::IoType* io_type, const diplomat::capi::LayoutFunction* layout, diplomat::capi::DiplomatI32View region_positions);

    typedef struct IoLayoutBuilder_add_output_from_region_auto_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} IoLayoutBuilder_add_output_from_region_auto_result;
    IoLayoutBuilder_add_output_from_region_auto_result IoLayoutBuilder_add_output_from_region_auto(diplomat::capi::IoLayoutBuilder* self, diplomat::capi::DiplomatStringView name, const diplomat::capi::IoType* io_type, diplomat::capi::DiplomatI32View region_positions);

    typedef struct IoLayoutBuilder_build_result {union {diplomat::capi::IoLayout* ok; diplomat::capi::NucleationError err;}; bool is_ok;} IoLayoutBuilder_build_result;
    IoLayoutBuilder_build_result IoLayoutBuilder_build(diplomat::capi::IoLayoutBuilder* self);

    void IoLayoutBuilder_destroy(IoLayoutBuilder* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::unique_ptr<IoLayoutBuilder> IoLayoutBuilder::create() {
    auto result = diplomat::capi::IoLayoutBuilder_create();
    return std::unique_ptr<IoLayoutBuilder>(IoLayoutBuilder::FromFFI(result));
}

inline diplomat::result<std::monostate, NucleationError> IoLayoutBuilder::add_input(std::string_view name, const IoType& io_type, const LayoutFunction& layout, diplomat::span<const int32_t> positions) {
    auto result = diplomat::capi::IoLayoutBuilder_add_input(this->AsFFI(),
        {name.data(), name.size()},
        io_type.AsFFI(),
        layout.AsFFI(),
        {positions.data(), positions.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> IoLayoutBuilder::add_output(std::string_view name, const IoType& io_type, const LayoutFunction& layout, diplomat::span<const int32_t> positions) {
    auto result = diplomat::capi::IoLayoutBuilder_add_output(this->AsFFI(),
        {name.data(), name.size()},
        io_type.AsFFI(),
        layout.AsFFI(),
        {positions.data(), positions.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> IoLayoutBuilder::add_input_auto(std::string_view name, const IoType& io_type, diplomat::span<const int32_t> positions) {
    auto result = diplomat::capi::IoLayoutBuilder_add_input_auto(this->AsFFI(),
        {name.data(), name.size()},
        io_type.AsFFI(),
        {positions.data(), positions.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> IoLayoutBuilder::add_output_auto(std::string_view name, const IoType& io_type, diplomat::span<const int32_t> positions) {
    auto result = diplomat::capi::IoLayoutBuilder_add_output_auto(this->AsFFI(),
        {name.data(), name.size()},
        io_type.AsFFI(),
        {positions.data(), positions.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> IoLayoutBuilder::add_input_from_region(std::string_view name, const IoType& io_type, const LayoutFunction& layout, diplomat::span<const int32_t> region_positions) {
    auto result = diplomat::capi::IoLayoutBuilder_add_input_from_region(this->AsFFI(),
        {name.data(), name.size()},
        io_type.AsFFI(),
        layout.AsFFI(),
        {region_positions.data(), region_positions.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> IoLayoutBuilder::add_input_from_region_auto(std::string_view name, const IoType& io_type, diplomat::span<const int32_t> region_positions) {
    auto result = diplomat::capi::IoLayoutBuilder_add_input_from_region_auto(this->AsFFI(),
        {name.data(), name.size()},
        io_type.AsFFI(),
        {region_positions.data(), region_positions.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> IoLayoutBuilder::add_output_from_region(std::string_view name, const IoType& io_type, const LayoutFunction& layout, diplomat::span<const int32_t> region_positions) {
    auto result = diplomat::capi::IoLayoutBuilder_add_output_from_region(this->AsFFI(),
        {name.data(), name.size()},
        io_type.AsFFI(),
        layout.AsFFI(),
        {region_positions.data(), region_positions.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> IoLayoutBuilder::add_output_from_region_auto(std::string_view name, const IoType& io_type, diplomat::span<const int32_t> region_positions) {
    auto result = diplomat::capi::IoLayoutBuilder_add_output_from_region_auto(this->AsFFI(),
        {name.data(), name.size()},
        io_type.AsFFI(),
        {region_positions.data(), region_positions.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<IoLayout>, NucleationError> IoLayoutBuilder::build() {
    auto result = diplomat::capi::IoLayoutBuilder_build(this->AsFFI());
    return result.is_ok ? diplomat::result<std::unique_ptr<IoLayout>, NucleationError>(diplomat::Ok<std::unique_ptr<IoLayout>>(std::unique_ptr<IoLayout>(IoLayout::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<IoLayout>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline const diplomat::capi::IoLayoutBuilder* IoLayoutBuilder::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::IoLayoutBuilder*>(this);
}

inline diplomat::capi::IoLayoutBuilder* IoLayoutBuilder::AsFFI() {
    return reinterpret_cast<diplomat::capi::IoLayoutBuilder*>(this);
}

inline const IoLayoutBuilder* IoLayoutBuilder::FromFFI(const diplomat::capi::IoLayoutBuilder* ptr) {
    return reinterpret_cast<const IoLayoutBuilder*>(ptr);
}

inline IoLayoutBuilder* IoLayoutBuilder::FromFFI(diplomat::capi::IoLayoutBuilder* ptr) {
    return reinterpret_cast<IoLayoutBuilder*>(ptr);
}

inline void IoLayoutBuilder::operator delete(void* ptr) {
    diplomat::capi::IoLayoutBuilder_destroy(reinterpret_cast<diplomat::capi::IoLayoutBuilder*>(ptr));
}


#endif // IoLayoutBuilder_HPP
