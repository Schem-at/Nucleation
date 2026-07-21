#ifndef Curve3D_HPP
#define Curve3D_HPP

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


namespace diplomat {
namespace capi {
    extern "C" {

    typedef struct Curve3D_from_points_result {union {diplomat::capi::Curve3D* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Curve3D_from_points_result;
    Curve3D_from_points_result Curve3D_from_points(diplomat::capi::DiplomatF64View coordinates, bool closed);

    uint32_t Curve3D_point_count(const diplomat::capi::Curve3D* self);

    bool Curve3D_is_closed(const diplomat::capi::Curve3D* self);

    void Curve3D_destroy(Curve3D* self);

    } // extern "C"
} // namespace capi
} // namespace

inline diplomat::result<std::unique_ptr<Curve3D>, NucleationError> Curve3D::from_points(diplomat::span<const double> coordinates, bool closed) {
    auto result = diplomat::capi::Curve3D_from_points({coordinates.data(), coordinates.size()},
        closed);
    return result.is_ok ? diplomat::result<std::unique_ptr<Curve3D>, NucleationError>(diplomat::Ok<std::unique_ptr<Curve3D>>(std::unique_ptr<Curve3D>(Curve3D::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Curve3D>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline uint32_t Curve3D::point_count() const {
    auto result = diplomat::capi::Curve3D_point_count(this->AsFFI());
    return result;
}

inline bool Curve3D::is_closed() const {
    auto result = diplomat::capi::Curve3D_is_closed(this->AsFFI());
    return result;
}

inline const diplomat::capi::Curve3D* Curve3D::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::Curve3D*>(this);
}

inline diplomat::capi::Curve3D* Curve3D::AsFFI() {
    return reinterpret_cast<diplomat::capi::Curve3D*>(this);
}

inline const Curve3D* Curve3D::FromFFI(const diplomat::capi::Curve3D* ptr) {
    return reinterpret_cast<const Curve3D*>(ptr);
}

inline Curve3D* Curve3D::FromFFI(diplomat::capi::Curve3D* ptr) {
    return reinterpret_cast<Curve3D*>(ptr);
}

inline void Curve3D::operator delete(void* ptr) {
    diplomat::capi::Curve3D_destroy(reinterpret_cast<diplomat::capi::Curve3D*>(ptr));
}


#endif // Curve3D_HPP
