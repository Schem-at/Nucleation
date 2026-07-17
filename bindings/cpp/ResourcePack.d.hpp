#ifndef ResourcePack_D_HPP
#define ResourcePack_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"

namespace diplomat::capi { struct ResourcePackList; }
class ResourcePackList;
struct TextureInfo;
class NucleationError;




namespace diplomat {
namespace capi {
    struct ResourcePack;
} // namespace capi
} // namespace

/**
 * A loaded (possibly merged) Minecraft resource pack.
 * Wraps {@link crate::meshing::ResourcePackSource}.
 */
class ResourcePack {
public:

  /**
   * Load a resource pack from an in-memory ZIP buffer.
   */
  inline static diplomat::result<std::unique_ptr<ResourcePack>, NucleationError> from_bytes(diplomat::span<const uint8_t> data);

  /**
   * Load and merge multiple resource packs, lowest priority first.
   */
  inline static diplomat::result<std::unique_ptr<ResourcePack>, NucleationError> from_list(const ResourcePackList& list);

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
  inline diplomat::result<std::string, NucleationError> get_blockstate_json(std::string_view name) const;
  template<typename W>
  inline diplomat::result<std::monostate, NucleationError> get_blockstate_json_write(std::string_view name, W& writeable_output) const;

  /**
   * The model definition JSON for `name`; `NotFound` if absent.
   */
  inline diplomat::result<std::string, NucleationError> get_model_json(std::string_view name) const;
  template<typename W>
  inline diplomat::result<std::monostate, NucleationError> get_model_json_write(std::string_view name, W& writeable_output) const;

  /**
   * Texture metadata (size, animation flag, frame count).
   */
  inline diplomat::result<TextureInfo, NucleationError> get_texture_info(std::string_view name) const;

  /**
   * Raw RGBA pixels of a texture, base64-encoded.
   */
  inline diplomat::result<std::string, NucleationError> get_texture_pixels_b64(std::string_view name) const;
  template<typename W>
  inline diplomat::result<std::monostate, NucleationError> get_texture_pixels_b64_write(std::string_view name, W& writeable_output) const;

  /**
   * Add (or override) a blockstate definition from a JSON string.
   */
  inline diplomat::result<std::monostate, NucleationError> add_blockstate_json(std::string_view name, std::string_view json);

  /**
   * Add (or override) a model definition from a JSON string.
   */
  inline diplomat::result<std::monostate, NucleationError> add_model_json(std::string_view name, std::string_view json);

  /**
   * Add a raw RGBA texture (`pixels` length must be `width * height * 4`).
   */
  inline diplomat::result<std::monostate, NucleationError> add_texture(std::string_view name, uint32_t width, uint32_t height, diplomat::span<const uint8_t> pixels);

  /**
   * Register a MeshExporter with the FormatManager so `save_as("mesh", ...)`
   * works. (Old ABI: `schematic_register_mesh_exporter`.)
   */
  inline diplomat::result<std::monostate, NucleationError> register_mesh_exporter() const;

    inline const diplomat::capi::ResourcePack* AsFFI() const;
    inline diplomat::capi::ResourcePack* AsFFI();
    inline static const ResourcePack* FromFFI(const diplomat::capi::ResourcePack* ptr);
    inline static ResourcePack* FromFFI(diplomat::capi::ResourcePack* ptr);
    inline static void operator delete(void* ptr);
private:
    ResourcePack() = delete;
    ResourcePack(const ResourcePack&) = delete;
    ResourcePack(ResourcePack&&) noexcept = delete;
    ResourcePack operator=(const ResourcePack&) = delete;
    ResourcePack operator=(ResourcePack&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // ResourcePack_D_HPP
