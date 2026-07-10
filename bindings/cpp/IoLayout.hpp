#ifndef IoLayout_HPP
#define IoLayout_HPP

#include "IoLayout.d.hpp"

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

    void IoLayout_input_names_json(const diplomat::capi::IoLayout* self, diplomat::capi::DiplomatWrite* write);

    void IoLayout_output_names_json(const diplomat::capi::IoLayout* self, diplomat::capi::DiplomatWrite* write);

    void IoLayout_destroy(IoLayout* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::string IoLayout::input_names_json() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::IoLayout_input_names_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void IoLayout::input_names_json_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::IoLayout_input_names_json(this->AsFFI(),
        &write);
}

inline std::string IoLayout::output_names_json() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::IoLayout_output_names_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void IoLayout::output_names_json_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::IoLayout_output_names_json(this->AsFFI(),
        &write);
}

inline const diplomat::capi::IoLayout* IoLayout::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::IoLayout*>(this);
}

inline diplomat::capi::IoLayout* IoLayout::AsFFI() {
    return reinterpret_cast<diplomat::capi::IoLayout*>(this);
}

inline const IoLayout* IoLayout::FromFFI(const diplomat::capi::IoLayout* ptr) {
    return reinterpret_cast<const IoLayout*>(ptr);
}

inline IoLayout* IoLayout::FromFFI(diplomat::capi::IoLayout* ptr) {
    return reinterpret_cast<IoLayout*>(ptr);
}

inline void IoLayout::operator delete(void* ptr) {
    diplomat::capi::IoLayout_destroy(reinterpret_cast<diplomat::capi::IoLayout*>(ptr));
}


#endif // IoLayout_HPP
