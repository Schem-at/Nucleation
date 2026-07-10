#ifndef NUCLEATION_TextureAtlas_HPP
#define NUCLEATION_TextureAtlas_HPP

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


namespace nucleation {
namespace capi {
    extern "C" {

    typedef struct TextureAtlas_build_global_result {union {nucleation::capi::TextureAtlas* ok; nucleation::capi::NucleationError err;}; bool is_ok;} TextureAtlas_build_global_result;
    TextureAtlas_build_global_result TextureAtlas_build_global(const nucleation::capi::Schematic* schematic, const nucleation::capi::ResourcePack* pack, const nucleation::capi::MeshConfig* config);

    uint32_t TextureAtlas_width(const nucleation::capi::TextureAtlas* self);

    uint32_t TextureAtlas_height(const nucleation::capi::TextureAtlas* self);

    void TextureAtlas_rgba_data_b64(const nucleation::capi::TextureAtlas* self, nucleation::diplomat::capi::DiplomatWrite* write);

    void TextureAtlas_destroy(TextureAtlas* self);

    } // extern "C"
} // namespace capi
} // namespace

inline nucleation::diplomat::result<std::unique_ptr<nucleation::TextureAtlas>, nucleation::NucleationError> nucleation::TextureAtlas::build_global(const nucleation::Schematic& schematic, const nucleation::ResourcePack& pack, const nucleation::MeshConfig& config) {
    auto result = nucleation::capi::TextureAtlas_build_global(schematic.AsFFI(),
        pack.AsFFI(),
        config.AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::TextureAtlas>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::TextureAtlas>>(std::unique_ptr<nucleation::TextureAtlas>(nucleation::TextureAtlas::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::TextureAtlas>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline uint32_t nucleation::TextureAtlas::width() const {
    auto result = nucleation::capi::TextureAtlas_width(this->AsFFI());
    return result;
}

inline uint32_t nucleation::TextureAtlas::height() const {
    auto result = nucleation::capi::TextureAtlas_height(this->AsFFI());
    return result;
}

inline std::string nucleation::TextureAtlas::rgba_data_b64() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::TextureAtlas_rgba_data_b64(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void nucleation::TextureAtlas::rgba_data_b64_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::TextureAtlas_rgba_data_b64(this->AsFFI(),
        &write);
}

inline const nucleation::capi::TextureAtlas* nucleation::TextureAtlas::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::TextureAtlas*>(this);
}

inline nucleation::capi::TextureAtlas* nucleation::TextureAtlas::AsFFI() {
    return reinterpret_cast<nucleation::capi::TextureAtlas*>(this);
}

inline const nucleation::TextureAtlas* nucleation::TextureAtlas::FromFFI(const nucleation::capi::TextureAtlas* ptr) {
    return reinterpret_cast<const nucleation::TextureAtlas*>(ptr);
}

inline nucleation::TextureAtlas* nucleation::TextureAtlas::FromFFI(nucleation::capi::TextureAtlas* ptr) {
    return reinterpret_cast<nucleation::TextureAtlas*>(ptr);
}

inline void nucleation::TextureAtlas::operator delete(void* ptr) {
    nucleation::capi::TextureAtlas_destroy(reinterpret_cast<nucleation::capi::TextureAtlas*>(ptr));
}


#endif // NUCLEATION_TextureAtlas_HPP
