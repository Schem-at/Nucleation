#ifndef NUCLEATION_Renderer_D_HPP
#define NUCLEATION_Renderer_D_HPP

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
namespace capi { struct RenderConfig; }
class RenderConfig;
namespace capi { struct Schematic; }
class Schematic;
class NucleationError;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct Renderer;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * Namespace type for the render entry points (PORTING rule 12).
 */
class Renderer {
public:

  /**
   * Render a schematic to raw RGBA pixel bytes, written as base64
   * (PORTING rule 6). `pack_zip` is a resource-pack zip in memory.
   */
  inline static nucleation::diplomat::result<std::string, nucleation::NucleationError> render_pixels_b64(const nucleation::Schematic& schematic, nucleation::diplomat::span<const uint8_t> pack_zip, const nucleation::RenderConfig& config);
  template<typename W>
  inline static nucleation::diplomat::result<std::monostate, nucleation::NucleationError> render_pixels_b64_write(const nucleation::Schematic& schematic, nucleation::diplomat::span<const uint8_t> pack_zip, const nucleation::RenderConfig& config, W& writeable_output);

  /**
   * Render a schematic to PNG bytes, written as base64 (PORTING rule 6).
   * `pack_zip` is a resource-pack zip in memory.
   */
  inline static nucleation::diplomat::result<std::string, nucleation::NucleationError> render_png_b64(const nucleation::Schematic& schematic, nucleation::diplomat::span<const uint8_t> pack_zip, const nucleation::RenderConfig& config);
  template<typename W>
  inline static nucleation::diplomat::result<std::monostate, nucleation::NucleationError> render_png_b64_write(const nucleation::Schematic& schematic, nucleation::diplomat::span<const uint8_t> pack_zip, const nucleation::RenderConfig& config, W& writeable_output);

  /**
   * Render a schematic to a PNG file at `path`.
   */
  inline static nucleation::diplomat::result<std::monostate, nucleation::NucleationError> render_to_file(const nucleation::Schematic& schematic, nucleation::diplomat::span<const uint8_t> pack_zip, const nucleation::RenderConfig& config, std::string_view path);

    inline const nucleation::capi::Renderer* AsFFI() const;
    inline nucleation::capi::Renderer* AsFFI();
    inline static const nucleation::Renderer* FromFFI(const nucleation::capi::Renderer* ptr);
    inline static nucleation::Renderer* FromFFI(nucleation::capi::Renderer* ptr);
    inline static void operator delete(void* ptr);
private:
    Renderer() = delete;
    Renderer(const nucleation::Renderer&) = delete;
    Renderer(nucleation::Renderer&&) noexcept = delete;
    Renderer operator=(const nucleation::Renderer&) = delete;
    Renderer operator=(nucleation::Renderer&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_Renderer_D_HPP
