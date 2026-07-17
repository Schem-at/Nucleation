#ifndef TextureAtlas_D_HPP
#define TextureAtlas_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"

namespace diplomat::capi { struct MeshConfig; }
class MeshConfig;
namespace diplomat::capi { struct ResourcePack; }
class ResourcePack;
namespace diplomat::capi { struct Schematic; }
class Schematic;
class NucleationError;




namespace diplomat {
namespace capi {
    struct TextureAtlas;
} // namespace capi
} // namespace

/**
 * A packed texture atlas. Wraps {@link schematic_mesher::TextureAtlas}.
 */
class TextureAtlas {
public:

  /**
   * Build a single shared atlas from every unique block state in the
   * schematic (old ABI: `schematic_build_global_atlas`).
   */
  inline static diplomat::result<std::unique_ptr<TextureAtlas>, NucleationError> build_global(const Schematic& schematic, const ResourcePack& pack, const MeshConfig& config);

  /**
   * Atlas width in pixels.
   */
  inline uint32_t width() const;

  /**
   * Atlas height in pixels.
   */
  inline uint32_t height() const;

  /**
   * Raw RGBA atlas pixels, base64-encoded.
   */
  inline std::string rgba_data_b64() const;
  template<typename W>
  inline void rgba_data_b64_write(W& writeable_output) const;

    inline const diplomat::capi::TextureAtlas* AsFFI() const;
    inline diplomat::capi::TextureAtlas* AsFFI();
    inline static const TextureAtlas* FromFFI(const diplomat::capi::TextureAtlas* ptr);
    inline static TextureAtlas* FromFFI(diplomat::capi::TextureAtlas* ptr);
    inline static void operator delete(void* ptr);
private:
    TextureAtlas() = delete;
    TextureAtlas(const TextureAtlas&) = delete;
    TextureAtlas(TextureAtlas&&) noexcept = delete;
    TextureAtlas operator=(const TextureAtlas&) = delete;
    TextureAtlas operator=(TextureAtlas&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // TextureAtlas_D_HPP
