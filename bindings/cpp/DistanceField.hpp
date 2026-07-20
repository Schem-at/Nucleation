#ifndef DistanceField_HPP
#define DistanceField_HPP

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


namespace diplomat {
namespace capi {
    extern "C" {

    diplomat::capi::DistanceField* DistanceField_from_schematic(const diplomat::capi::Schematic* schematic);

    int32_t DistanceField_depth(const diplomat::capi::DistanceField* self, int32_t x, int32_t y, int32_t z);

    float DistanceField_slope(const diplomat::capi::DistanceField* self, int32_t x, int32_t y, int32_t z);

    void DistanceField_normal_json(const diplomat::capi::DistanceField* self, int32_t x, int32_t y, int32_t z, diplomat::capi::DiplomatWrite* write);

    void DistanceField_destroy(DistanceField* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::unique_ptr<DistanceField> DistanceField::from_schematic(const Schematic& schematic) {
    auto result = diplomat::capi::DistanceField_from_schematic(schematic.AsFFI());
    return std::unique_ptr<DistanceField>(DistanceField::FromFFI(result));
}

inline int32_t DistanceField::depth(int32_t x, int32_t y, int32_t z) const {
    auto result = diplomat::capi::DistanceField_depth(this->AsFFI(),
        x,
        y,
        z);
    return result;
}

inline float DistanceField::slope(int32_t x, int32_t y, int32_t z) const {
    auto result = diplomat::capi::DistanceField_slope(this->AsFFI(),
        x,
        y,
        z);
    return result;
}

inline std::string DistanceField::normal_json(int32_t x, int32_t y, int32_t z) const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::DistanceField_normal_json(this->AsFFI(),
        x,
        y,
        z,
        &write);
    return output;
}
template<typename W>
inline void DistanceField::normal_json_write(int32_t x, int32_t y, int32_t z, W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::DistanceField_normal_json(this->AsFFI(),
        x,
        y,
        z,
        &write);
}

inline const diplomat::capi::DistanceField* DistanceField::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::DistanceField*>(this);
}

inline diplomat::capi::DistanceField* DistanceField::AsFFI() {
    return reinterpret_cast<diplomat::capi::DistanceField*>(this);
}

inline const DistanceField* DistanceField::FromFFI(const diplomat::capi::DistanceField* ptr) {
    return reinterpret_cast<const DistanceField*>(ptr);
}

inline DistanceField* DistanceField::FromFFI(diplomat::capi::DistanceField* ptr) {
    return reinterpret_cast<DistanceField*>(ptr);
}

inline void DistanceField::operator delete(void* ptr) {
    diplomat::capi::DistanceField_destroy(reinterpret_cast<diplomat::capi::DistanceField*>(ptr));
}


#endif // DistanceField_HPP
