#ifndef NUCLEATION_ItemModelResult_D_HPP
#define NUCLEATION_ItemModelResult_D_HPP

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
namespace capi { struct ItemModelConfig; }
class ItemModelConfig;
namespace capi { struct ItemModelPackBuilder; }
class ItemModelPackBuilder;
namespace capi { struct ItemModelResult; }
class ItemModelResult;
namespace capi { struct ResourcePack; }
class ResourcePack;
namespace capi { struct Schematic; }
class Schematic;
struct Dimensions;
struct ItemScale;
class NucleationError;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct ItemModelResult;
} // namespace capi
} // namespace

namespace nucleation {
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
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::ItemModelResult>, nucleation::NucleationError> create(const nucleation::Schematic& schematic, const nucleation::ResourcePack& pack, const nucleation::ItemModelConfig& config);

  /**
   * The Minecraft item model JSON.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> model_json() const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> model_json_write(W& writeable_output) const;

  inline nucleation::diplomat::result<uint32_t, nucleation::NucleationError> element_count() const;

  inline nucleation::diplomat::result<uint32_t, nucleation::NucleationError> texture_count() const;

  inline nucleation::diplomat::result<uint32_t, nucleation::NucleationError> plane_count() const;

  inline nucleation::diplomat::result<nucleation::Dimensions, nucleation::NucleationError> dimensions() const;

  /**
   * The applied model scale (old ABI: `itemmodel_result_scale`).
   */
  inline nucleation::diplomat::result<nucleation::ItemScale, nucleation::NucleationError> scale() const;

  /**
   * A single-model resource pack ZIP, base64-encoded.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> to_resource_pack_zip_b64() const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> to_resource_pack_zip_b64_write(W& writeable_output) const;

  /**
   * Move this result into a pack builder. Consumes the result: further
   * accessor calls return `AlreadyConsumed`.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> add_to_pack(const nucleation::ItemModelPackBuilder& builder);

    inline const nucleation::capi::ItemModelResult* AsFFI() const;
    inline nucleation::capi::ItemModelResult* AsFFI();
    inline static const nucleation::ItemModelResult* FromFFI(const nucleation::capi::ItemModelResult* ptr);
    inline static nucleation::ItemModelResult* FromFFI(nucleation::capi::ItemModelResult* ptr);
    inline static void operator delete(void* ptr);
private:
    ItemModelResult() = delete;
    ItemModelResult(const nucleation::ItemModelResult&) = delete;
    ItemModelResult(nucleation::ItemModelResult&&) noexcept = delete;
    ItemModelResult operator=(const nucleation::ItemModelResult&) = delete;
    ItemModelResult operator=(nucleation::ItemModelResult&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_ItemModelResult_D_HPP
