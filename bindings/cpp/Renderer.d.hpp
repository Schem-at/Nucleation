#ifndef Renderer_D_HPP
#define Renderer_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"

namespace diplomat::capi { struct RenderConfig; }
class RenderConfig;
namespace diplomat::capi { struct Schematic; }
class Schematic;
class NucleationError;




namespace diplomat {
namespace capi {
    struct Renderer;
} // namespace capi
} // namespace

/**
 * Namespace type for the render entry points (PORTING rule 12).
 */
class Renderer {
public:

  /**
   * Render a schematic to raw RGBA pixel bytes, written as base64
   * (PORTING rule 6). `pack_zip` is a resource-pack zip in memory.
   */
  inline static diplomat::result<std::string, NucleationError> render_pixels_b64(const Schematic& schematic, diplomat::span<const uint8_t> pack_zip, const RenderConfig& config);
  template<typename W>
  inline static diplomat::result<std::monostate, NucleationError> render_pixels_b64_write(const Schematic& schematic, diplomat::span<const uint8_t> pack_zip, const RenderConfig& config, W& writeable_output);

  /**
   * Render a schematic to PNG bytes, written as base64 (PORTING rule 6).
   * `pack_zip` is a resource-pack zip in memory.
   */
  inline static diplomat::result<std::string, NucleationError> render_png_b64(const Schematic& schematic, diplomat::span<const uint8_t> pack_zip, const RenderConfig& config);
  template<typename W>
  inline static diplomat::result<std::monostate, NucleationError> render_png_b64_write(const Schematic& schematic, diplomat::span<const uint8_t> pack_zip, const RenderConfig& config, W& writeable_output);

  /**
   * Render a schematic to a PNG file at `path`.
   */
  inline static diplomat::result<std::monostate, NucleationError> render_to_file(const Schematic& schematic, diplomat::span<const uint8_t> pack_zip, const RenderConfig& config, std::string_view path);

    inline const diplomat::capi::Renderer* AsFFI() const;
    inline diplomat::capi::Renderer* AsFFI();
    inline static const Renderer* FromFFI(const diplomat::capi::Renderer* ptr);
    inline static Renderer* FromFFI(diplomat::capi::Renderer* ptr);
    inline static void operator delete(void* ptr);
private:
    Renderer() = delete;
    Renderer(const Renderer&) = delete;
    Renderer(Renderer&&) noexcept = delete;
    Renderer operator=(const Renderer&) = delete;
    Renderer operator=(Renderer&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // Renderer_D_HPP
