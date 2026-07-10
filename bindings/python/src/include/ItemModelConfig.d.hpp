#ifndef NUCLEATION_ItemModelConfig_D_HPP
#define NUCLEATION_ItemModelConfig_D_HPP

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
class NucleationError;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct ItemModelConfig;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * Item model generation configuration. Wraps {@link crate::meshing::ItemModelConfig}.
 */
class ItemModelConfig {
public:

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::ItemModelConfig>, nucleation::NucleationError> create(std::string_view model_name);

  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> set_namespace(std::string_view namespace_);

  inline void set_center(bool center);

  inline void set_texture_resolution(uint32_t resolution);

  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> set_item(std::string_view item);

  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> set_custom_model_data(std::string_view cmd);

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

    inline const nucleation::capi::ItemModelConfig* AsFFI() const;
    inline nucleation::capi::ItemModelConfig* AsFFI();
    inline static const nucleation::ItemModelConfig* FromFFI(const nucleation::capi::ItemModelConfig* ptr);
    inline static nucleation::ItemModelConfig* FromFFI(nucleation::capi::ItemModelConfig* ptr);
    inline static void operator delete(void* ptr);
private:
    ItemModelConfig() = delete;
    ItemModelConfig(const nucleation::ItemModelConfig&) = delete;
    ItemModelConfig(nucleation::ItemModelConfig&&) noexcept = delete;
    ItemModelConfig operator=(const nucleation::ItemModelConfig&) = delete;
    ItemModelConfig operator=(nucleation::ItemModelConfig&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_ItemModelConfig_D_HPP
