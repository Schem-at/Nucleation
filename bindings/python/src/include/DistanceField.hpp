#ifndef NUCLEATION_DistanceField_HPP
#define NUCLEATION_DistanceField_HPP

#include "DistanceField.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "Schematic.hpp"
#include "diplomat_runtime.hpp"


namespace nucleation {
namespace capi {
    extern "C" {

    nucleation::capi::DistanceField* DistanceField_from_schematic(const nucleation::capi::Schematic* schematic);

    int32_t DistanceField_depth(const nucleation::capi::DistanceField* self, int32_t x, int32_t y, int32_t z);

    float DistanceField_slope(const nucleation::capi::DistanceField* self, int32_t x, int32_t y, int32_t z);

    void DistanceField_normal_json(const nucleation::capi::DistanceField* self, int32_t x, int32_t y, int32_t z, nucleation::diplomat::capi::DiplomatWrite* write);

    void DistanceField_destroy(DistanceField* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::unique_ptr<nucleation::DistanceField> nucleation::DistanceField::from_schematic(const nucleation::Schematic& schematic) {
    auto result = nucleation::capi::DistanceField_from_schematic(schematic.AsFFI());
    return std::unique_ptr<nucleation::DistanceField>(nucleation::DistanceField::FromFFI(result));
}

inline int32_t nucleation::DistanceField::depth(int32_t x, int32_t y, int32_t z) const {
    auto result = nucleation::capi::DistanceField_depth(this->AsFFI(),
        x,
        y,
        z);
    return result;
}

inline float nucleation::DistanceField::slope(int32_t x, int32_t y, int32_t z) const {
    auto result = nucleation::capi::DistanceField_slope(this->AsFFI(),
        x,
        y,
        z);
    return result;
}

inline std::string nucleation::DistanceField::normal_json(int32_t x, int32_t y, int32_t z) const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::DistanceField_normal_json(this->AsFFI(),
        x,
        y,
        z,
        &write);
    return output;
}
template<typename W>
inline void nucleation::DistanceField::normal_json_write(int32_t x, int32_t y, int32_t z, W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::DistanceField_normal_json(this->AsFFI(),
        x,
        y,
        z,
        &write);
}

inline const nucleation::capi::DistanceField* nucleation::DistanceField::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::DistanceField*>(this);
}

inline nucleation::capi::DistanceField* nucleation::DistanceField::AsFFI() {
    return reinterpret_cast<nucleation::capi::DistanceField*>(this);
}

inline const nucleation::DistanceField* nucleation::DistanceField::FromFFI(const nucleation::capi::DistanceField* ptr) {
    return reinterpret_cast<const nucleation::DistanceField*>(ptr);
}

inline nucleation::DistanceField* nucleation::DistanceField::FromFFI(nucleation::capi::DistanceField* ptr) {
    return reinterpret_cast<nucleation::DistanceField*>(ptr);
}

inline void nucleation::DistanceField::operator delete(void* ptr) {
    nucleation::capi::DistanceField_destroy(reinterpret_cast<nucleation::capi::DistanceField*>(ptr));
}


#endif // NUCLEATION_DistanceField_HPP
