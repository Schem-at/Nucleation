#ifndef BlockState_HPP
#define BlockState_HPP

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


namespace diplomat {
namespace capi {
    extern "C" {

    diplomat::capi::BlockState* BlockState_create(diplomat::capi::DiplomatStringView name);

    typedef struct BlockState_with_property_result {union {diplomat::capi::BlockState* ok; diplomat::capi::NucleationError err;}; bool is_ok;} BlockState_with_property_result;
    BlockState_with_property_result BlockState_with_property(const diplomat::capi::BlockState* self, diplomat::capi::DiplomatStringView key, diplomat::capi::DiplomatStringView value);

    void BlockState_name(const diplomat::capi::BlockState* self, diplomat::capi::DiplomatWrite* write);

    void BlockState_properties_json(const diplomat::capi::BlockState* self, diplomat::capi::DiplomatWrite* write);

    void BlockState_destroy(BlockState* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::unique_ptr<BlockState> BlockState::create(std::string_view name) {
    auto result = diplomat::capi::BlockState_create({name.data(), name.size()});
    return std::unique_ptr<BlockState>(BlockState::FromFFI(result));
}

inline diplomat::result<std::unique_ptr<BlockState>, NucleationError> BlockState::with_property(std::string_view key, std::string_view value) const {
    auto result = diplomat::capi::BlockState_with_property(this->AsFFI(),
        {key.data(), key.size()},
        {value.data(), value.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<BlockState>, NucleationError>(diplomat::Ok<std::unique_ptr<BlockState>>(std::unique_ptr<BlockState>(BlockState::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<BlockState>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline std::string BlockState::name() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::BlockState_name(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void BlockState::name_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::BlockState_name(this->AsFFI(),
        &write);
}

inline std::string BlockState::properties_json() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::BlockState_properties_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void BlockState::properties_json_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::BlockState_properties_json(this->AsFFI(),
        &write);
}

inline const diplomat::capi::BlockState* BlockState::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::BlockState*>(this);
}

inline diplomat::capi::BlockState* BlockState::AsFFI() {
    return reinterpret_cast<diplomat::capi::BlockState*>(this);
}

inline const BlockState* BlockState::FromFFI(const diplomat::capi::BlockState* ptr) {
    return reinterpret_cast<const BlockState*>(ptr);
}

inline BlockState* BlockState::FromFFI(diplomat::capi::BlockState* ptr) {
    return reinterpret_cast<BlockState*>(ptr);
}

inline void BlockState::operator delete(void* ptr) {
    diplomat::capi::BlockState_destroy(reinterpret_cast<diplomat::capi::BlockState*>(ptr));
}


#endif // BlockState_HPP
