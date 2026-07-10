#ifndef ItemModelPackBuilder_D_HPP
#define ItemModelPackBuilder_D_HPP

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
    struct ItemModelPackBuilder;
} // namespace capi
} // namespace

/**
 * Accumulates {@link ItemModelResult}s to build a combined resource pack ZIP
 * (old ABI: `itemmodel_build_resource_pack` took an array of pointers).
 */
class ItemModelPackBuilder {
public:

  inline static std::unique_ptr<ItemModelPackBuilder> create();

  /**
   * Number of results added so far.
   */
  inline uint32_t len() const;

  /**
   * Build a resource pack ZIP from every added result, base64-encoded.
   */
  inline diplomat::result<std::string, NucleationError> build_zip_b64() const;
  template<typename W>
  inline diplomat::result<std::monostate, NucleationError> build_zip_b64_write(W& writeable_output) const;

    inline const diplomat::capi::ItemModelPackBuilder* AsFFI() const;
    inline diplomat::capi::ItemModelPackBuilder* AsFFI();
    inline static const ItemModelPackBuilder* FromFFI(const diplomat::capi::ItemModelPackBuilder* ptr);
    inline static ItemModelPackBuilder* FromFFI(diplomat::capi::ItemModelPackBuilder* ptr);
    inline static void operator delete(void* ptr);
private:
    ItemModelPackBuilder() = delete;
    ItemModelPackBuilder(const ItemModelPackBuilder&) = delete;
    ItemModelPackBuilder(ItemModelPackBuilder&&) noexcept = delete;
    ItemModelPackBuilder operator=(const ItemModelPackBuilder&) = delete;
    ItemModelPackBuilder operator=(ItemModelPackBuilder&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // ItemModelPackBuilder_D_HPP
