#ifndef NUCLEATION_ItemModelPackBuilder_D_HPP
#define NUCLEATION_ItemModelPackBuilder_D_HPP

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
namespace capi { struct ItemModelPackBuilder; }
class ItemModelPackBuilder;
class NucleationError;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct ItemModelPackBuilder;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * Accumulates {@link ItemModelResult}s to build a combined resource pack ZIP
 * (old ABI: `itemmodel_build_resource_pack` took an array of pointers).
 */
class ItemModelPackBuilder {
public:

  inline static std::unique_ptr<nucleation::ItemModelPackBuilder> create();

  /**
   * Number of results added so far.
   */
  inline uint32_t len() const;

  /**
   * Build a resource pack ZIP from every added result, base64-encoded.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> build_zip_b64() const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> build_zip_b64_write(W& writeable_output) const;

    inline const nucleation::capi::ItemModelPackBuilder* AsFFI() const;
    inline nucleation::capi::ItemModelPackBuilder* AsFFI();
    inline static const nucleation::ItemModelPackBuilder* FromFFI(const nucleation::capi::ItemModelPackBuilder* ptr);
    inline static nucleation::ItemModelPackBuilder* FromFFI(nucleation::capi::ItemModelPackBuilder* ptr);
    inline static void operator delete(void* ptr);
private:
    ItemModelPackBuilder() = delete;
    ItemModelPackBuilder(const nucleation::ItemModelPackBuilder&) = delete;
    ItemModelPackBuilder(nucleation::ItemModelPackBuilder&&) noexcept = delete;
    ItemModelPackBuilder operator=(const nucleation::ItemModelPackBuilder&) = delete;
    ItemModelPackBuilder operator=(nucleation::ItemModelPackBuilder&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_ItemModelPackBuilder_D_HPP
