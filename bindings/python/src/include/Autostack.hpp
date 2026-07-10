#ifndef NUCLEATION_Autostack_HPP
#define NUCLEATION_Autostack_HPP

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


namespace nucleation {
namespace capi {
    extern "C" {

    void Autostack_detect_structures(const nucleation::capi::Schematic* schematic, nucleation::diplomat::capi::DiplomatWrite* write);

    void Autostack_detect_structures_graph(const nucleation::capi::Schematic* schematic, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct Autostack_resize_1d_result {union {nucleation::capi::Schematic* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Autostack_resize_1d_result;
    Autostack_resize_1d_result Autostack_resize_1d(const nucleation::capi::Schematic* schematic, int32_t vx, int32_t vy, int32_t vz, uint32_t units);

    typedef struct Autostack_resize_2d_result {union {nucleation::capi::Schematic* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Autostack_resize_2d_result;
    Autostack_resize_2d_result Autostack_resize_2d(const nucleation::capi::Schematic* schematic, int32_t v1x, int32_t v1y, int32_t v1z, int32_t v2x, int32_t v2y, int32_t v2z, uint32_t n1, uint32_t n2);

    void Autostack_destroy(Autostack* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::string nucleation::Autostack::detect_structures(const nucleation::Schematic& schematic) {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::Autostack_detect_structures(schematic.AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void nucleation::Autostack::detect_structures_write(const nucleation::Schematic& schematic, W& writeable) {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::Autostack_detect_structures(schematic.AsFFI(),
        &write);
}

inline std::string nucleation::Autostack::detect_structures_graph(const nucleation::Schematic& schematic) {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::Autostack_detect_structures_graph(schematic.AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void nucleation::Autostack::detect_structures_graph_write(const nucleation::Schematic& schematic, W& writeable) {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::Autostack_detect_structures_graph(schematic.AsFFI(),
        &write);
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> nucleation::Autostack::resize_1d(const nucleation::Schematic& schematic, int32_t vx, int32_t vy, int32_t vz, uint32_t units) {
    auto result = nucleation::capi::Autostack_resize_1d(schematic.AsFFI(),
        vx,
        vy,
        vz,
        units);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Schematic>>(std::unique_ptr<nucleation::Schematic>(nucleation::Schematic::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> nucleation::Autostack::resize_2d(const nucleation::Schematic& schematic, int32_t v1x, int32_t v1y, int32_t v1z, int32_t v2x, int32_t v2y, int32_t v2z, uint32_t n1, uint32_t n2) {
    auto result = nucleation::capi::Autostack_resize_2d(schematic.AsFFI(),
        v1x,
        v1y,
        v1z,
        v2x,
        v2y,
        v2z,
        n1,
        n2);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Schematic>>(std::unique_ptr<nucleation::Schematic>(nucleation::Schematic::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline const nucleation::capi::Autostack* nucleation::Autostack::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::Autostack*>(this);
}

inline nucleation::capi::Autostack* nucleation::Autostack::AsFFI() {
    return reinterpret_cast<nucleation::capi::Autostack*>(this);
}

inline const nucleation::Autostack* nucleation::Autostack::FromFFI(const nucleation::capi::Autostack* ptr) {
    return reinterpret_cast<const nucleation::Autostack*>(ptr);
}

inline nucleation::Autostack* nucleation::Autostack::FromFFI(nucleation::capi::Autostack* ptr) {
    return reinterpret_cast<nucleation::Autostack*>(ptr);
}

inline void nucleation::Autostack::operator delete(void* ptr) {
    nucleation::capi::Autostack_destroy(reinterpret_cast<nucleation::capi::Autostack*>(ptr));
}


#endif // NUCLEATION_Autostack_HPP
