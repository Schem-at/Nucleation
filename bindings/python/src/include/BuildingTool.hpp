#ifndef NUCLEATION_BuildingTool_HPP
#define NUCLEATION_BuildingTool_HPP

#include "BuildingTool.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "Brush.hpp"
#include "NucleationError.hpp"
#include "Schematic.hpp"
#include "Shape.hpp"
#include "diplomat_runtime.hpp"


namespace nucleation {
namespace capi {
    extern "C" {

    void BuildingTool_fill(nucleation::capi::Schematic* schematic, const nucleation::capi::Shape* shape, const nucleation::capi::Brush* brush);

    void BuildingTool_rstack(nucleation::capi::Schematic* schematic, const nucleation::capi::Shape* shape, const nucleation::capi::Brush* brush, size_t count, int32_t offset_x, int32_t offset_y, int32_t offset_z);

    void BuildingTool_fill_only_air(nucleation::capi::Schematic* schematic, const nucleation::capi::Shape* shape, const nucleation::capi::Brush* brush);

    typedef struct BuildingTool_fill_replacing_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} BuildingTool_fill_replacing_result;
    BuildingTool_fill_replacing_result BuildingTool_fill_replacing(nucleation::capi::Schematic* schematic, const nucleation::capi::Shape* shape, const nucleation::capi::Brush* brush, nucleation::diplomat::capi::DiplomatStringView targets_json);

    void BuildingTool_destroy(BuildingTool* self);

    } // extern "C"
} // namespace capi
} // namespace

inline void nucleation::BuildingTool::fill(nucleation::Schematic& schematic, const nucleation::Shape& shape, const nucleation::Brush& brush) {
    nucleation::capi::BuildingTool_fill(schematic.AsFFI(),
        shape.AsFFI(),
        brush.AsFFI());
}

inline void nucleation::BuildingTool::rstack(nucleation::Schematic& schematic, const nucleation::Shape& shape, const nucleation::Brush& brush, size_t count, int32_t offset_x, int32_t offset_y, int32_t offset_z) {
    nucleation::capi::BuildingTool_rstack(schematic.AsFFI(),
        shape.AsFFI(),
        brush.AsFFI(),
        count,
        offset_x,
        offset_y,
        offset_z);
}

inline void nucleation::BuildingTool::fill_only_air(nucleation::Schematic& schematic, const nucleation::Shape& shape, const nucleation::Brush& brush) {
    nucleation::capi::BuildingTool_fill_only_air(schematic.AsFFI(),
        shape.AsFFI(),
        brush.AsFFI());
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::BuildingTool::fill_replacing(nucleation::Schematic& schematic, const nucleation::Shape& shape, const nucleation::Brush& brush, std::string_view targets_json) {
    auto result = nucleation::capi::BuildingTool_fill_replacing(schematic.AsFFI(),
        shape.AsFFI(),
        brush.AsFFI(),
        {targets_json.data(), targets_json.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline const nucleation::capi::BuildingTool* nucleation::BuildingTool::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::BuildingTool*>(this);
}

inline nucleation::capi::BuildingTool* nucleation::BuildingTool::AsFFI() {
    return reinterpret_cast<nucleation::capi::BuildingTool*>(this);
}

inline const nucleation::BuildingTool* nucleation::BuildingTool::FromFFI(const nucleation::capi::BuildingTool* ptr) {
    return reinterpret_cast<const nucleation::BuildingTool*>(ptr);
}

inline nucleation::BuildingTool* nucleation::BuildingTool::FromFFI(nucleation::capi::BuildingTool* ptr) {
    return reinterpret_cast<nucleation::BuildingTool*>(ptr);
}

inline void nucleation::BuildingTool::operator delete(void* ptr) {
    nucleation::capi::BuildingTool_destroy(reinterpret_cast<nucleation::capi::BuildingTool*>(ptr));
}


#endif // NUCLEATION_BuildingTool_HPP
