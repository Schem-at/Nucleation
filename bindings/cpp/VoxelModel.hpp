#ifndef VoxelModel_HPP
#define VoxelModel_HPP

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


namespace diplomat {
namespace capi {
    extern "C" {

    typedef struct VoxelModel_plan_json_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} VoxelModel_plan_json_result;
    VoxelModel_plan_json_result VoxelModel_plan_json(const diplomat::capi::VoxelModel* self, diplomat::capi::DiplomatStringView options_json, diplomat::capi::DiplomatWrite* write);

    typedef struct VoxelModel_to_schematic_result {union {diplomat::capi::Schematic* ok; diplomat::capi::NucleationError err;}; bool is_ok;} VoxelModel_to_schematic_result;
    VoxelModel_to_schematic_result VoxelModel_to_schematic(const diplomat::capi::VoxelModel* self, diplomat::capi::DiplomatStringView options_json, const diplomat::capi::Palette* palette, diplomat::capi::DiplomatStringView name);

    void VoxelModel_destroy(VoxelModel* self);

    } // extern "C"
} // namespace capi
} // namespace

inline diplomat::result<std::string, NucleationError> VoxelModel::plan_json(std::string_view options_json) const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::VoxelModel_plan_json(this->AsFFI(),
        {options_json.data(), options_json.size()},
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> VoxelModel::plan_json_write(std::string_view options_json, W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::VoxelModel_plan_json(this->AsFFI(),
        {options_json.data(), options_json.size()},
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Schematic>, NucleationError> VoxelModel::to_schematic(std::string_view options_json, const Palette& palette, std::string_view name) const {
    auto result = diplomat::capi::VoxelModel_to_schematic(this->AsFFI(),
        {options_json.data(), options_json.size()},
        palette.AsFFI(),
        {name.data(), name.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Ok<std::unique_ptr<Schematic>>(std::unique_ptr<Schematic>(Schematic::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline const diplomat::capi::VoxelModel* VoxelModel::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::VoxelModel*>(this);
}

inline diplomat::capi::VoxelModel* VoxelModel::AsFFI() {
    return reinterpret_cast<diplomat::capi::VoxelModel*>(this);
}

inline const VoxelModel* VoxelModel::FromFFI(const diplomat::capi::VoxelModel* ptr) {
    return reinterpret_cast<const VoxelModel*>(ptr);
}

inline VoxelModel* VoxelModel::FromFFI(diplomat::capi::VoxelModel* ptr) {
    return reinterpret_cast<VoxelModel*>(ptr);
}

inline void VoxelModel::operator delete(void* ptr) {
    diplomat::capi::VoxelModel_destroy(reinterpret_cast<diplomat::capi::VoxelModel*>(ptr));
}


#endif // VoxelModel_HPP
