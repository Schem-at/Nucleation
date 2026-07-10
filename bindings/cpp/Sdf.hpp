#ifndef Sdf_HPP
#define Sdf_HPP

#include "Sdf.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "NucleationError.hpp"
#include "Schematic.hpp"
#include "diplomat_runtime.hpp"


namespace diplomat {
namespace capi {
    extern "C" {

    typedef struct Sdf_schematic_from_sdf_result {union {diplomat::capi::Schematic* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Sdf_schematic_from_sdf_result;
    Sdf_schematic_from_sdf_result Sdf_schematic_from_sdf(diplomat::capi::DiplomatStringView sdf_json, diplomat::capi::DiplomatStringView rules_json, bool has_bounds, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

    typedef struct Sdf_eval_result {union {float ok; diplomat::capi::NucleationError err;}; bool is_ok;} Sdf_eval_result;
    Sdf_eval_result Sdf_eval(diplomat::capi::DiplomatStringView sdf_json, float x, float y, float z);

    void Sdf_destroy(Sdf* self);

    } // extern "C"
} // namespace capi
} // namespace

inline diplomat::result<std::unique_ptr<Schematic>, NucleationError> Sdf::schematic_from_sdf(std::string_view sdf_json, std::string_view rules_json, bool has_bounds, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z) {
    auto result = diplomat::capi::Sdf_schematic_from_sdf({sdf_json.data(), sdf_json.size()},
        {rules_json.data(), rules_json.size()},
        has_bounds,
        min_x,
        min_y,
        min_z,
        max_x,
        max_y,
        max_z);
    return result.is_ok ? diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Ok<std::unique_ptr<Schematic>>(std::unique_ptr<Schematic>(Schematic::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<float, NucleationError> Sdf::eval(std::string_view sdf_json, float x, float y, float z) {
    auto result = diplomat::capi::Sdf_eval({sdf_json.data(), sdf_json.size()},
        x,
        y,
        z);
    return result.is_ok ? diplomat::result<float, NucleationError>(diplomat::Ok<float>(result.ok)) : diplomat::result<float, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline const diplomat::capi::Sdf* Sdf::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::Sdf*>(this);
}

inline diplomat::capi::Sdf* Sdf::AsFFI() {
    return reinterpret_cast<diplomat::capi::Sdf*>(this);
}

inline const Sdf* Sdf::FromFFI(const diplomat::capi::Sdf* ptr) {
    return reinterpret_cast<const Sdf*>(ptr);
}

inline Sdf* Sdf::FromFFI(diplomat::capi::Sdf* ptr) {
    return reinterpret_cast<Sdf*>(ptr);
}

inline void Sdf::operator delete(void* ptr) {
    diplomat::capi::Sdf_destroy(reinterpret_cast<diplomat::capi::Sdf*>(ptr));
}


#endif // Sdf_HPP
