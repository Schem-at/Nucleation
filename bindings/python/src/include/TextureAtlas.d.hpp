#ifndef NUCLEATION_TextureAtlas_D_HPP
#define NUCLEATION_TextureAtlas_D_HPP

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
namespace capi { struct MeshConfig; }
class MeshConfig;
namespace capi { struct ResourcePack; }
class ResourcePack;
namespace capi { struct Schematic; }
class Schematic;
namespace capi { struct TextureAtlas; }
class TextureAtlas;
class NucleationError;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct TextureAtlas;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * A packed texture atlas. Wraps {@link schematic_mesher::TextureAtlas}.
 */
class TextureAtlas {
public:

  /**
   * Build a single shared atlas from every unique block state in the
   * schematic (old ABI: `schematic_build_global_atlas`).
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::TextureAtlas>, nucleation::NucleationError> build_global(const nucleation::Schematic& schematic, const nucleation::ResourcePack& pack, const nucleation::MeshConfig& config);

  inline uint32_t width() const;

  inline uint32_t height() const;

  /**
   * Raw RGBA atlas pixels, base64-encoded.
   */
  inline std::string rgba_data_b64() const;
  template<typename W>
  inline void rgba_data_b64_write(W& writeable_output) const;

    inline const nucleation::capi::TextureAtlas* AsFFI() const;
    inline nucleation::capi::TextureAtlas* AsFFI();
    inline static const nucleation::TextureAtlas* FromFFI(const nucleation::capi::TextureAtlas* ptr);
    inline static nucleation::TextureAtlas* FromFFI(nucleation::capi::TextureAtlas* ptr);
    inline static void operator delete(void* ptr);
private:
    TextureAtlas() = delete;
    TextureAtlas(const nucleation::TextureAtlas&) = delete;
    TextureAtlas(nucleation::TextureAtlas&&) noexcept = delete;
    TextureAtlas operator=(const nucleation::TextureAtlas&) = delete;
    TextureAtlas operator=(nucleation::TextureAtlas&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_TextureAtlas_D_HPP
