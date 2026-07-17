#ifndef BuildingTool_HPP
#define BuildingTool_HPP

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


namespace diplomat {
namespace capi {
    extern "C" {

    void BuildingTool_fill(diplomat::capi::Schematic* schematic, const diplomat::capi::Shape* shape, const diplomat::capi::Brush* brush);

    void BuildingTool_rstack(diplomat::capi::Schematic* schematic, const diplomat::capi::Shape* shape, const diplomat::capi::Brush* brush, size_t count, int32_t offset_x, int32_t offset_y, int32_t offset_z);

    void BuildingTool_fill_only_air(diplomat::capi::Schematic* schematic, const diplomat::capi::Shape* shape, const diplomat::capi::Brush* brush);

    typedef struct BuildingTool_fill_replacing_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} BuildingTool_fill_replacing_result;
    BuildingTool_fill_replacing_result BuildingTool_fill_replacing(diplomat::capi::Schematic* schematic, const diplomat::capi::Shape* shape, const diplomat::capi::Brush* brush, diplomat::capi::DiplomatStringView targets_json);

    void BuildingTool_destroy(BuildingTool* self);

    } // extern "C"
} // namespace capi
} // namespace

inline void BuildingTool::fill(Schematic& schematic, const Shape& shape, const Brush& brush) {
    diplomat::capi::BuildingTool_fill(schematic.AsFFI(),
        shape.AsFFI(),
        brush.AsFFI());
}

inline void BuildingTool::rstack(Schematic& schematic, const Shape& shape, const Brush& brush, size_t count, int32_t offset_x, int32_t offset_y, int32_t offset_z) {
    diplomat::capi::BuildingTool_rstack(schematic.AsFFI(),
        shape.AsFFI(),
        brush.AsFFI(),
        count,
        offset_x,
        offset_y,
        offset_z);
}

inline void BuildingTool::fill_only_air(Schematic& schematic, const Shape& shape, const Brush& brush) {
    diplomat::capi::BuildingTool_fill_only_air(schematic.AsFFI(),
        shape.AsFFI(),
        brush.AsFFI());
}

inline diplomat::result<std::monostate, NucleationError> BuildingTool::fill_replacing(Schematic& schematic, const Shape& shape, const Brush& brush, std::string_view targets_json) {
    auto result = diplomat::capi::BuildingTool_fill_replacing(schematic.AsFFI(),
        shape.AsFFI(),
        brush.AsFFI(),
        {targets_json.data(), targets_json.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline const diplomat::capi::BuildingTool* BuildingTool::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::BuildingTool*>(this);
}

inline diplomat::capi::BuildingTool* BuildingTool::AsFFI() {
    return reinterpret_cast<diplomat::capi::BuildingTool*>(this);
}

inline const BuildingTool* BuildingTool::FromFFI(const diplomat::capi::BuildingTool* ptr) {
    return reinterpret_cast<const BuildingTool*>(ptr);
}

inline BuildingTool* BuildingTool::FromFFI(diplomat::capi::BuildingTool* ptr) {
    return reinterpret_cast<BuildingTool*>(ptr);
}

inline void BuildingTool::operator delete(void* ptr) {
    diplomat::capi::BuildingTool_destroy(reinterpret_cast<diplomat::capi::BuildingTool*>(ptr));
}


#endif // BuildingTool_HPP
