#ifndef NUCLEATION_BlockState_HPP
#define NUCLEATION_BlockState_HPP

#include "BlockState.d.hpp"

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

    nucleation::capi::BlockState* BlockState_create(nucleation::diplomat::capi::DiplomatStringView name);

    typedef struct BlockState_with_property_result {union {nucleation::capi::BlockState* ok; nucleation::capi::NucleationError err;}; bool is_ok;} BlockState_with_property_result;
    BlockState_with_property_result BlockState_with_property(const nucleation::capi::BlockState* self, nucleation::diplomat::capi::DiplomatStringView key, nucleation::diplomat::capi::DiplomatStringView value);

    void BlockState_name(const nucleation::capi::BlockState* self, nucleation::diplomat::capi::DiplomatWrite* write);

    void BlockState_properties_json(const nucleation::capi::BlockState* self, nucleation::diplomat::capi::DiplomatWrite* write);

    void BlockState_destroy(BlockState* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::unique_ptr<nucleation::BlockState> nucleation::BlockState::create(std::string_view name) {
    auto result = nucleation::capi::BlockState_create({name.data(), name.size()});
    return std::unique_ptr<nucleation::BlockState>(nucleation::BlockState::FromFFI(result));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::BlockState>, nucleation::NucleationError> nucleation::BlockState::with_property(std::string_view key, std::string_view value) const {
    auto result = nucleation::capi::BlockState_with_property(this->AsFFI(),
        {key.data(), key.size()},
        {value.data(), value.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::BlockState>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::BlockState>>(std::unique_ptr<nucleation::BlockState>(nucleation::BlockState::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::BlockState>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline std::string nucleation::BlockState::name() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::BlockState_name(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void nucleation::BlockState::name_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::BlockState_name(this->AsFFI(),
        &write);
}

inline std::string nucleation::BlockState::properties_json() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::BlockState_properties_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void nucleation::BlockState::properties_json_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::BlockState_properties_json(this->AsFFI(),
        &write);
}

inline const nucleation::capi::BlockState* nucleation::BlockState::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::BlockState*>(this);
}

inline nucleation::capi::BlockState* nucleation::BlockState::AsFFI() {
    return reinterpret_cast<nucleation::capi::BlockState*>(this);
}

inline const nucleation::BlockState* nucleation::BlockState::FromFFI(const nucleation::capi::BlockState* ptr) {
    return reinterpret_cast<const nucleation::BlockState*>(ptr);
}

inline nucleation::BlockState* nucleation::BlockState::FromFFI(nucleation::capi::BlockState* ptr) {
    return reinterpret_cast<nucleation::BlockState*>(ptr);
}

inline void nucleation::BlockState::operator delete(void* ptr) {
    nucleation::capi::BlockState_destroy(reinterpret_cast<nucleation::capi::BlockState*>(ptr));
}


#endif // NUCLEATION_BlockState_HPP
