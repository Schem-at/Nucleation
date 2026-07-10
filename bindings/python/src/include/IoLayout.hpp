#ifndef NUCLEATION_IoLayout_HPP
#define NUCLEATION_IoLayout_HPP

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


namespace nucleation {
namespace capi {
    extern "C" {

    void IoLayout_input_names_json(const nucleation::capi::IoLayout* self, nucleation::diplomat::capi::DiplomatWrite* write);

    void IoLayout_output_names_json(const nucleation::capi::IoLayout* self, nucleation::diplomat::capi::DiplomatWrite* write);

    void IoLayout_destroy(IoLayout* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::string nucleation::IoLayout::input_names_json() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::IoLayout_input_names_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void nucleation::IoLayout::input_names_json_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::IoLayout_input_names_json(this->AsFFI(),
        &write);
}

inline std::string nucleation::IoLayout::output_names_json() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::IoLayout_output_names_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void nucleation::IoLayout::output_names_json_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::IoLayout_output_names_json(this->AsFFI(),
        &write);
}

inline const nucleation::capi::IoLayout* nucleation::IoLayout::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::IoLayout*>(this);
}

inline nucleation::capi::IoLayout* nucleation::IoLayout::AsFFI() {
    return reinterpret_cast<nucleation::capi::IoLayout*>(this);
}

inline const nucleation::IoLayout* nucleation::IoLayout::FromFFI(const nucleation::capi::IoLayout* ptr) {
    return reinterpret_cast<const nucleation::IoLayout*>(ptr);
}

inline nucleation::IoLayout* nucleation::IoLayout::FromFFI(nucleation::capi::IoLayout* ptr) {
    return reinterpret_cast<nucleation::IoLayout*>(ptr);
}

inline void nucleation::IoLayout::operator delete(void* ptr) {
    nucleation::capi::IoLayout_destroy(reinterpret_cast<nucleation::capi::IoLayout*>(ptr));
}


#endif // NUCLEATION_IoLayout_HPP
