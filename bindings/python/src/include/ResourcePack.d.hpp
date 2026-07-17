#ifndef NUCLEATION_ResourcePack_D_HPP
#define NUCLEATION_ResourcePack_D_HPP

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
namespace capi { struct ResourcePack; }
class ResourcePack;
namespace capi { struct ResourcePackList; }
class ResourcePackList;
struct TextureInfo;
class NucleationError;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct ResourcePack;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * A loaded (possibly merged) Minecraft resource pack.
 * Wraps {@link crate::meshing::ResourcePackSource}.
 */
class ResourcePack {
public:

  /**
   * Load a resource pack from an in-memory ZIP buffer.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::ResourcePack>, nucleation::NucleationError> from_bytes(nucleation::diplomat::span<const uint8_t> data);

  /**
   * Load and merge multiple resource packs, lowest priority first.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::ResourcePack>, nucleation::NucleationError> from_list(const nucleation::ResourcePackList& list);

  /**
   * Number of blockstate definitions in the pack.
   */
  inline uint32_t blockstate_count() const;

  /**
   * Number of model definitions in the pack.
   */
  inline uint32_t model_count() const;

  /**
   * Number of textures in the pack.
   */
  inline uint32_t texture_count() const;

  /**
   * Namespaces present in the pack, as a JSON array string.
   */
  inline std::string namespaces_json() const;
  template<typename W>
  inline void namespaces_json_write(W& writeable_output) const;

  /**
   * All blockstate identifiers, as a JSON array string.
   */
  inline std::string list_blockstates_json() const;
  template<typename W>
  inline void list_blockstates_json_write(W& writeable_output) const;

  /**
   * All model identifiers, as a JSON array string.
   */
  inline std::string list_models_json() const;
  template<typename W>
  inline void list_models_json_write(W& writeable_output) const;

  /**
   * All texture identifiers, as a JSON array string.
   */
  inline std::string list_textures_json() const;
  template<typename W>
  inline void list_textures_json_write(W& writeable_output) const;

  /**
   * The blockstate definition JSON for `name`; `NotFound` if absent.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> get_blockstate_json(std::string_view name) const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> get_blockstate_json_write(std::string_view name, W& writeable_output) const;

  /**
   * The model definition JSON for `name`; `NotFound` if absent.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> get_model_json(std::string_view name) const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> get_model_json_write(std::string_view name, W& writeable_output) const;

  /**
   * Texture metadata (size, animation flag, frame count).
   */
  inline nucleation::diplomat::result<nucleation::TextureInfo, nucleation::NucleationError> get_texture_info(std::string_view name) const;

  /**
   * Raw RGBA pixels of a texture, base64-encoded.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> get_texture_pixels_b64(std::string_view name) const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> get_texture_pixels_b64_write(std::string_view name, W& writeable_output) const;

  /**
   * Add (or override) a blockstate definition from a JSON string.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> add_blockstate_json(std::string_view name, std::string_view json);

  /**
   * Add (or override) a model definition from a JSON string.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> add_model_json(std::string_view name, std::string_view json);

  /**
   * Add a raw RGBA texture (`pixels` length must be `width * height * 4`).
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> add_texture(std::string_view name, uint32_t width, uint32_t height, nucleation::diplomat::span<const uint8_t> pixels);

  /**
   * Register a MeshExporter with the FormatManager so `save_as("mesh", ...)`
   * works. (Old ABI: `schematic_register_mesh_exporter`.)
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> register_mesh_exporter() const;

    inline const nucleation::capi::ResourcePack* AsFFI() const;
    inline nucleation::capi::ResourcePack* AsFFI();
    inline static const nucleation::ResourcePack* FromFFI(const nucleation::capi::ResourcePack* ptr);
    inline static nucleation::ResourcePack* FromFFI(nucleation::capi::ResourcePack* ptr);
    inline static void operator delete(void* ptr);
private:
    ResourcePack() = delete;
    ResourcePack(const nucleation::ResourcePack&) = delete;
    ResourcePack(nucleation::ResourcePack&&) noexcept = delete;
    ResourcePack operator=(const nucleation::ResourcePack&) = delete;
    ResourcePack operator=(nucleation::ResourcePack&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_ResourcePack_D_HPP
