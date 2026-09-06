#ifndef NUCLEATION_VoxelModel_HPP
#define NUCLEATION_VoxelModel_HPP

#include "VoxelModel.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "NucleationError.hpp"
#include "Palette.hpp"
#include "Schematic.hpp"
#include "diplomat_runtime.hpp"


namespace nucleation {
namespace capi {
    extern "C" {

    typedef struct VoxelModel_plan_json_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} VoxelModel_plan_json_result;
    VoxelModel_plan_json_result VoxelModel_plan_json(const nucleation::capi::VoxelModel* self, nucleation::diplomat::capi::DiplomatStringView options_json, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct VoxelModel_to_schematic_result {union {nucleation::capi::Schematic* ok; nucleation::capi::NucleationError err;}; bool is_ok;} VoxelModel_to_schematic_result;
    VoxelModel_to_schematic_result VoxelModel_to_schematic(const nucleation::capi::VoxelModel* self, nucleation::diplomat::capi::DiplomatStringView options_json, const nucleation::capi::Palette* palette, nucleation::diplomat::capi::DiplomatStringView name);

    void VoxelModel_destroy(VoxelModel* self);

    } // extern "C"
} // namespace capi
} // namespace

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::VoxelModel::plan_json(std::string_view options_json) const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::VoxelModel_plan_json(this->AsFFI(),
        {options_json.data(), options_json.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::VoxelModel::plan_json_write(std::string_view options_json, W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::VoxelModel_plan_json(this->AsFFI(),
        {options_json.data(), options_json.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> nucleation::VoxelModel::to_schematic(std::string_view options_json, const nucleation::Palette& palette, std::string_view name) const {
    auto result = nucleation::capi::VoxelModel_to_schematic(this->AsFFI(),
        {options_json.data(), options_json.size()},
        palette.AsFFI(),
        {name.data(), name.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Schematic>>(std::unique_ptr<nucleation::Schematic>(nucleation::Schematic::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline const nucleation::capi::VoxelModel* nucleation::VoxelModel::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::VoxelModel*>(this);
}

inline nucleation::capi::VoxelModel* nucleation::VoxelModel::AsFFI() {
    return reinterpret_cast<nucleation::capi::VoxelModel*>(this);
}

inline const nucleation::VoxelModel* nucleation::VoxelModel::FromFFI(const nucleation::capi::VoxelModel* ptr) {
    return reinterpret_cast<const nucleation::VoxelModel*>(ptr);
}

inline nucleation::VoxelModel* nucleation::VoxelModel::FromFFI(nucleation::capi::VoxelModel* ptr) {
    return reinterpret_cast<nucleation::VoxelModel*>(ptr);
}

inline void nucleation::VoxelModel::operator delete(void* ptr) {
    nucleation::capi::VoxelModel_destroy(reinterpret_cast<nucleation::capi::VoxelModel*>(ptr));
}


#endif // NUCLEATION_VoxelModel_HPP
