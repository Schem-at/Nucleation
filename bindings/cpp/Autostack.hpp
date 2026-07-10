#ifndef Autostack_HPP
#define Autostack_HPP

#include "Autostack.d.hpp"

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

    void Autostack_detect_structures(const diplomat::capi::Schematic* schematic, diplomat::capi::DiplomatWrite* write);

    void Autostack_detect_structures_graph(const diplomat::capi::Schematic* schematic, diplomat::capi::DiplomatWrite* write);

    typedef struct Autostack_resize_1d_result {union {diplomat::capi::Schematic* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Autostack_resize_1d_result;
    Autostack_resize_1d_result Autostack_resize_1d(const diplomat::capi::Schematic* schematic, int32_t vx, int32_t vy, int32_t vz, uint32_t units);

    typedef struct Autostack_resize_2d_result {union {diplomat::capi::Schematic* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Autostack_resize_2d_result;
    Autostack_resize_2d_result Autostack_resize_2d(const diplomat::capi::Schematic* schematic, int32_t v1x, int32_t v1y, int32_t v1z, int32_t v2x, int32_t v2y, int32_t v2z, uint32_t n1, uint32_t n2);

    void Autostack_destroy(Autostack* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::string Autostack::detect_structures(const Schematic& schematic) {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::Autostack_detect_structures(schematic.AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void Autostack::detect_structures_write(const Schematic& schematic, W& writeable) {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::Autostack_detect_structures(schematic.AsFFI(),
        &write);
}

inline std::string Autostack::detect_structures_graph(const Schematic& schematic) {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::Autostack_detect_structures_graph(schematic.AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void Autostack::detect_structures_graph_write(const Schematic& schematic, W& writeable) {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::Autostack_detect_structures_graph(schematic.AsFFI(),
        &write);
}

inline diplomat::result<std::unique_ptr<Schematic>, NucleationError> Autostack::resize_1d(const Schematic& schematic, int32_t vx, int32_t vy, int32_t vz, uint32_t units) {
    auto result = diplomat::capi::Autostack_resize_1d(schematic.AsFFI(),
        vx,
        vy,
        vz,
        units);
    return result.is_ok ? diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Ok<std::unique_ptr<Schematic>>(std::unique_ptr<Schematic>(Schematic::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Schematic>, NucleationError> Autostack::resize_2d(const Schematic& schematic, int32_t v1x, int32_t v1y, int32_t v1z, int32_t v2x, int32_t v2y, int32_t v2z, uint32_t n1, uint32_t n2) {
    auto result = diplomat::capi::Autostack_resize_2d(schematic.AsFFI(),
        v1x,
        v1y,
        v1z,
        v2x,
        v2y,
        v2z,
        n1,
        n2);
    return result.is_ok ? diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Ok<std::unique_ptr<Schematic>>(std::unique_ptr<Schematic>(Schematic::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline const diplomat::capi::Autostack* Autostack::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::Autostack*>(this);
}

inline diplomat::capi::Autostack* Autostack::AsFFI() {
    return reinterpret_cast<diplomat::capi::Autostack*>(this);
}

inline const Autostack* Autostack::FromFFI(const diplomat::capi::Autostack* ptr) {
    return reinterpret_cast<const Autostack*>(ptr);
}

inline Autostack* Autostack::FromFFI(diplomat::capi::Autostack* ptr) {
    return reinterpret_cast<Autostack*>(ptr);
}

inline void Autostack::operator delete(void* ptr) {
    diplomat::capi::Autostack_destroy(reinterpret_cast<diplomat::capi::Autostack*>(ptr));
}


#endif // Autostack_HPP
