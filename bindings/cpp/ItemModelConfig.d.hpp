#ifndef ItemModelConfig_D_HPP
#define ItemModelConfig_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"

class NucleationError;




namespace diplomat {
namespace capi {
    struct ItemModelConfig;
} // namespace capi
} // namespace

/**
 * Item model generation configuration. Wraps {@link crate::meshing::ItemModelConfig}.
 */
class ItemModelConfig {
public:

  inline static diplomat::result<std::unique_ptr<ItemModelConfig>, NucleationError> create(std::string_view model_name);

  inline diplomat::result<std::monostate, NucleationError> set_namespace(std::string_view namespace_);

  inline void set_center(bool center);

  inline void set_texture_resolution(uint32_t resolution);

  inline diplomat::result<std::monostate, NucleationError> set_item(std::string_view item);

  inline diplomat::result<std::monostate, NucleationError> set_custom_model_data(std::string_view cmd);

  /**
   * Uniform scale.
   */
  inline void set_scale(float scale);

  /**
   * Per-axis scale.
   */
  inline void set_scale_xyz(float sx, float sy, float sz);

  /**
   * Auto-fit scale.
   */
  inline void set_scale_auto();

    inline const diplomat::capi::ItemModelConfig* AsFFI() const;
    inline diplomat::capi::ItemModelConfig* AsFFI();
    inline static const ItemModelConfig* FromFFI(const diplomat::capi::ItemModelConfig* ptr);
    inline static ItemModelConfig* FromFFI(diplomat::capi::ItemModelConfig* ptr);
    inline static void operator delete(void* ptr);
private:
    ItemModelConfig() = delete;
    ItemModelConfig(const ItemModelConfig&) = delete;
    ItemModelConfig(ItemModelConfig&&) noexcept = delete;
    ItemModelConfig operator=(const ItemModelConfig&) = delete;
    ItemModelConfig operator=(ItemModelConfig&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // ItemModelConfig_D_HPP
