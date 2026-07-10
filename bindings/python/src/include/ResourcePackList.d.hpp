#ifndef NUCLEATION_ResourcePackList_D_HPP
#define NUCLEATION_ResourcePackList_D_HPP

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
namespace capi { struct ResourcePackList; }
class ResourcePackList;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct ResourcePackList;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * Ordered list of raw resource-pack ZIP buffers, lowest priority first.
 * Feed it to {@link ResourcePack::from_list}.
 */
class ResourcePackList {
public:

  inline static std::unique_ptr<nucleation::ResourcePackList> create();

  /**
   * Append one resource-pack ZIP buffer. Later buffers overlay earlier
   * ones on per-key collision (Minecraft pack-ordering semantics).
   */
  inline void add(nucleation::diplomat::span<const uint8_t> data);

  inline uint32_t len() const;

    inline const nucleation::capi::ResourcePackList* AsFFI() const;
    inline nucleation::capi::ResourcePackList* AsFFI();
    inline static const nucleation::ResourcePackList* FromFFI(const nucleation::capi::ResourcePackList* ptr);
    inline static nucleation::ResourcePackList* FromFFI(nucleation::capi::ResourcePackList* ptr);
    inline static void operator delete(void* ptr);
private:
    ResourcePackList() = delete;
    ResourcePackList(const nucleation::ResourcePackList&) = delete;
    ResourcePackList(nucleation::ResourcePackList&&) noexcept = delete;
    ResourcePackList operator=(const nucleation::ResourcePackList&) = delete;
    ResourcePackList operator=(nucleation::ResourcePackList&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_ResourcePackList_D_HPP
