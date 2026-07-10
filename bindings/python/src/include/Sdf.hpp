#ifndef NUCLEATION_Sdf_HPP
#define NUCLEATION_Sdf_HPP

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


namespace nucleation {
namespace capi {
    extern "C" {

    typedef struct Sdf_schematic_from_sdf_result {union {nucleation::capi::Schematic* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Sdf_schematic_from_sdf_result;
    Sdf_schematic_from_sdf_result Sdf_schematic_from_sdf(nucleation::diplomat::capi::DiplomatStringView sdf_json, nucleation::diplomat::capi::DiplomatStringView rules_json, bool has_bounds, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

    typedef struct Sdf_eval_result {union {float ok; nucleation::capi::NucleationError err;}; bool is_ok;} Sdf_eval_result;
    Sdf_eval_result Sdf_eval(nucleation::diplomat::capi::DiplomatStringView sdf_json, float x, float y, float z);

    void Sdf_destroy(Sdf* self);

    } // extern "C"
} // namespace capi
} // namespace

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> nucleation::Sdf::schematic_from_sdf(std::string_view sdf_json, std::string_view rules_json, bool has_bounds, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z) {
    auto result = nucleation::capi::Sdf_schematic_from_sdf({sdf_json.data(), sdf_json.size()},
        {rules_json.data(), rules_json.size()},
        has_bounds,
        min_x,
        min_y,
        min_z,
        max_x,
        max_y,
        max_z);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Schematic>>(std::unique_ptr<nucleation::Schematic>(nucleation::Schematic::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<float, nucleation::NucleationError> nucleation::Sdf::eval(std::string_view sdf_json, float x, float y, float z) {
    auto result = nucleation::capi::Sdf_eval({sdf_json.data(), sdf_json.size()},
        x,
        y,
        z);
    return result.is_ok ? nucleation::diplomat::result<float, nucleation::NucleationError>(nucleation::diplomat::Ok<float>(result.ok)) : nucleation::diplomat::result<float, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline const nucleation::capi::Sdf* nucleation::Sdf::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::Sdf*>(this);
}

inline nucleation::capi::Sdf* nucleation::Sdf::AsFFI() {
    return reinterpret_cast<nucleation::capi::Sdf*>(this);
}

inline const nucleation::Sdf* nucleation::Sdf::FromFFI(const nucleation::capi::Sdf* ptr) {
    return reinterpret_cast<const nucleation::Sdf*>(ptr);
}

inline nucleation::Sdf* nucleation::Sdf::FromFFI(nucleation::capi::Sdf* ptr) {
    return reinterpret_cast<nucleation::Sdf*>(ptr);
}

inline void nucleation::Sdf::operator delete(void* ptr) {
    nucleation::capi::Sdf_destroy(reinterpret_cast<nucleation::capi::Sdf*>(ptr));
}


#endif // NUCLEATION_Sdf_HPP
