#ifndef NUCLEATION_Curve3D_HPP
#define NUCLEATION_Curve3D_HPP

#include "Curve3D.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "NucleationError.hpp"
#include "diplomat_runtime.hpp"


namespace nucleation {
namespace capi {
    extern "C" {

    typedef struct Curve3D_from_points_result {union {nucleation::capi::Curve3D* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Curve3D_from_points_result;
    Curve3D_from_points_result Curve3D_from_points(nucleation::diplomat::capi::DiplomatF64View coordinates, bool closed);

    uint32_t Curve3D_point_count(const nucleation::capi::Curve3D* self);

    bool Curve3D_is_closed(const nucleation::capi::Curve3D* self);

    void Curve3D_destroy(Curve3D* self);

    } // extern "C"
} // namespace capi
} // namespace

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Curve3D>, nucleation::NucleationError> nucleation::Curve3D::from_points(nucleation::diplomat::span<const double> coordinates, bool closed) {
    auto result = nucleation::capi::Curve3D_from_points({coordinates.data(), coordinates.size()},
        closed);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Curve3D>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Curve3D>>(std::unique_ptr<nucleation::Curve3D>(nucleation::Curve3D::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Curve3D>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline uint32_t nucleation::Curve3D::point_count() const {
    auto result = nucleation::capi::Curve3D_point_count(this->AsFFI());
    return result;
}

inline bool nucleation::Curve3D::is_closed() const {
    auto result = nucleation::capi::Curve3D_is_closed(this->AsFFI());
    return result;
}

inline const nucleation::capi::Curve3D* nucleation::Curve3D::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::Curve3D*>(this);
}

inline nucleation::capi::Curve3D* nucleation::Curve3D::AsFFI() {
    return reinterpret_cast<nucleation::capi::Curve3D*>(this);
}

inline const nucleation::Curve3D* nucleation::Curve3D::FromFFI(const nucleation::capi::Curve3D* ptr) {
    return reinterpret_cast<const nucleation::Curve3D*>(ptr);
}

inline nucleation::Curve3D* nucleation::Curve3D::FromFFI(nucleation::capi::Curve3D* ptr) {
    return reinterpret_cast<nucleation::Curve3D*>(ptr);
}

inline void nucleation::Curve3D::operator delete(void* ptr) {
    nucleation::capi::Curve3D_destroy(reinterpret_cast<nucleation::capi::Curve3D*>(ptr));
}


#endif // NUCLEATION_Curve3D_HPP
