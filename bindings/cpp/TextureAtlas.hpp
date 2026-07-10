#ifndef TextureAtlas_HPP
#define TextureAtlas_HPP

#include "TextureAtlas.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "MeshConfig.hpp"
#include "NucleationError.hpp"
#include "ResourcePack.hpp"
#include "Schematic.hpp"
#include "diplomat_runtime.hpp"


namespace diplomat {
namespace capi {
    extern "C" {

    typedef struct TextureAtlas_build_global_result {union {diplomat::capi::TextureAtlas* ok; diplomat::capi::NucleationError err;}; bool is_ok;} TextureAtlas_build_global_result;
    TextureAtlas_build_global_result TextureAtlas_build_global(const diplomat::capi::Schematic* schematic, const diplomat::capi::ResourcePack* pack, const diplomat::capi::MeshConfig* config);

    uint32_t TextureAtlas_width(const diplomat::capi::TextureAtlas* self);

    uint32_t TextureAtlas_height(const diplomat::capi::TextureAtlas* self);

    void TextureAtlas_rgba_data_b64(const diplomat::capi::TextureAtlas* self, diplomat::capi::DiplomatWrite* write);

    void TextureAtlas_destroy(TextureAtlas* self);

    } // extern "C"
} // namespace capi
} // namespace

inline diplomat::result<std::unique_ptr<TextureAtlas>, NucleationError> TextureAtlas::build_global(const Schematic& schematic, const ResourcePack& pack, const MeshConfig& config) {
    auto result = diplomat::capi::TextureAtlas_build_global(schematic.AsFFI(),
        pack.AsFFI(),
        config.AsFFI());
    return result.is_ok ? diplomat::result<std::unique_ptr<TextureAtlas>, NucleationError>(diplomat::Ok<std::unique_ptr<TextureAtlas>>(std::unique_ptr<TextureAtlas>(TextureAtlas::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<TextureAtlas>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline uint32_t TextureAtlas::width() const {
    auto result = diplomat::capi::TextureAtlas_width(this->AsFFI());
    return result;
}

inline uint32_t TextureAtlas::height() const {
    auto result = diplomat::capi::TextureAtlas_height(this->AsFFI());
    return result;
}

inline std::string TextureAtlas::rgba_data_b64() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::TextureAtlas_rgba_data_b64(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void TextureAtlas::rgba_data_b64_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::TextureAtlas_rgba_data_b64(this->AsFFI(),
        &write);
}

inline const diplomat::capi::TextureAtlas* TextureAtlas::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::TextureAtlas*>(this);
}

inline diplomat::capi::TextureAtlas* TextureAtlas::AsFFI() {
    return reinterpret_cast<diplomat::capi::TextureAtlas*>(this);
}

inline const TextureAtlas* TextureAtlas::FromFFI(const diplomat::capi::TextureAtlas* ptr) {
    return reinterpret_cast<const TextureAtlas*>(ptr);
}

inline TextureAtlas* TextureAtlas::FromFFI(diplomat::capi::TextureAtlas* ptr) {
    return reinterpret_cast<TextureAtlas*>(ptr);
}

inline void TextureAtlas::operator delete(void* ptr) {
    diplomat::capi::TextureAtlas_destroy(reinterpret_cast<diplomat::capi::TextureAtlas*>(ptr));
}


#endif // TextureAtlas_HPP
