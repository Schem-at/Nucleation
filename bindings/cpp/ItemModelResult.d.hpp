#ifndef ItemModelResult_D_HPP
#define ItemModelResult_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"

namespace diplomat::capi { struct ItemModelConfig; }
class ItemModelConfig;
namespace diplomat::capi { struct ItemModelPackBuilder; }
class ItemModelPackBuilder;
namespace diplomat::capi { struct ResourcePack; }
class ResourcePack;
namespace diplomat::capi { struct Schematic; }
class Schematic;
struct Dimensions;
struct ItemScale;
class NucleationError;




namespace diplomat {
namespace capi {
    struct ItemModelResult;
} // namespace capi
} // namespace

/**
 * A generated item model. Wraps {@link crate::meshing::ItemModelResult}.
 *
 * Holds `Option<...>` because {@link ItemModelResult::add_to_pack} moves the
 * inner value into an {@link ItemModelPackBuilder} (PORTING rule 11); accessors
 * return `AlreadyConsumed` afterwards.
 */
class ItemModelResult {
public:

  /**
   * Generate an item model from a schematic (old ABI: `schematic_to_item_model`).
   */
  inline static diplomat::result<std::unique_ptr<ItemModelResult>, NucleationError> create(const Schematic& schematic, const ResourcePack& pack, const ItemModelConfig& config);

  /**
   * The Minecraft item model JSON.
   */
  inline diplomat::result<std::string, NucleationError> model_json() const;
  template<typename W>
  inline diplomat::result<std::monostate, NucleationError> model_json_write(W& writeable_output) const;

  /**
   * Number of elements (cuboids) in the generated model.
   */
  inline diplomat::result<uint32_t, NucleationError> element_count() const;

  /**
   * Number of textures the generated model uses.
   */
  inline diplomat::result<uint32_t, NucleationError> texture_count() const;

  /**
   * Number of textured planes in the generated model.
   */
  inline diplomat::result<uint32_t, NucleationError> plane_count() const;

  /**
   * Source schematic dimensions in blocks.
   */
  inline diplomat::result<Dimensions, NucleationError> dimensions() const;

  /**
   * The applied model scale (old ABI: `itemmodel_result_scale`).
   */
  inline diplomat::result<ItemScale, NucleationError> scale() const;

  /**
   * A single-model resource pack ZIP, base64-encoded.
   */
  inline diplomat::result<std::string, NucleationError> to_resource_pack_zip_b64() const;
  template<typename W>
  inline diplomat::result<std::monostate, NucleationError> to_resource_pack_zip_b64_write(W& writeable_output) const;

  /**
   * Move this result into a pack builder. Consumes the result: further
   * accessor calls return `AlreadyConsumed`.
   */
  inline diplomat::result<std::monostate, NucleationError> add_to_pack(const ItemModelPackBuilder& builder);

    inline const diplomat::capi::ItemModelResult* AsFFI() const;
    inline diplomat::capi::ItemModelResult* AsFFI();
    inline static const ItemModelResult* FromFFI(const diplomat::capi::ItemModelResult* ptr);
    inline static ItemModelResult* FromFFI(diplomat::capi::ItemModelResult* ptr);
    inline static void operator delete(void* ptr);
private:
    ItemModelResult() = delete;
    ItemModelResult(const ItemModelResult&) = delete;
    ItemModelResult(ItemModelResult&&) noexcept = delete;
    ItemModelResult operator=(const ItemModelResult&) = delete;
    ItemModelResult operator=(ItemModelResult&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // ItemModelResult_D_HPP
