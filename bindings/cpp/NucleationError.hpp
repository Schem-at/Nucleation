#ifndef NucleationError_HPP
#define NucleationError_HPP

#include "NucleationError.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"


namespace diplomat {
namespace capi {
    extern "C" {

    void NucleationError_detail(diplomat::capi::NucleationError self, diplomat::capi::DiplomatWrite* write);

    } // extern "C"
} // namespace capi
} // namespace

inline diplomat::capi::NucleationError NucleationError::AsFFI() const {
    return static_cast<diplomat::capi::NucleationError>(value);
}

inline NucleationError NucleationError::FromFFI(diplomat::capi::NucleationError c_enum) {
    switch (c_enum) {
        case diplomat::capi::NucleationError_NullArgument:
        case diplomat::capi::NucleationError_InvalidArgument:
        case diplomat::capi::NucleationError_Parse:
        case diplomat::capi::NucleationError_Serialize:
        case diplomat::capi::NucleationError_Io:
        case diplomat::capi::NucleationError_Lock:
        case diplomat::capi::NucleationError_Store:
        case diplomat::capi::NucleationError_Mesh:
        case diplomat::capi::NucleationError_Render:
        case diplomat::capi::NucleationError_Simulation:
        case diplomat::capi::NucleationError_AlreadyConsumed:
        case diplomat::capi::NucleationError_NotFound:
        case diplomat::capi::NucleationError_Generation:
            return static_cast<NucleationError::Value>(c_enum);
        default:
            std::abort();
    }
}

inline std::string NucleationError::detail() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::NucleationError_detail(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void NucleationError::detail_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::NucleationError_detail(this->AsFFI(),
        &write);
}
#endif // NucleationError_HPP
