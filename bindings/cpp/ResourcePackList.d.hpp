#ifndef ResourcePackList_D_HPP
#define ResourcePackList_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"


namespace diplomat {
namespace capi {
    struct ResourcePackList;
} // namespace capi
} // namespace

/**
 * Ordered list of raw resource-pack ZIP buffers, lowest priority first.
 * Feed it to {@link ResourcePack::from_list}.
 */
class ResourcePackList {
public:

  /**
   * Create an empty resource-pack list.
   */
  inline static std::unique_ptr<ResourcePackList> create();

  /**
   * Append one resource-pack ZIP buffer. Later buffers overlay earlier
   * ones on per-key collision (Minecraft pack-ordering semantics).
   */
  inline void add(diplomat::span<const uint8_t> data);

  /**
   * Number of pack buffers added so far.
   */
  inline uint32_t len() const;

    inline const diplomat::capi::ResourcePackList* AsFFI() const;
    inline diplomat::capi::ResourcePackList* AsFFI();
    inline static const ResourcePackList* FromFFI(const diplomat::capi::ResourcePackList* ptr);
    inline static ResourcePackList* FromFFI(diplomat::capi::ResourcePackList* ptr);
    inline static void operator delete(void* ptr);
private:
    ResourcePackList() = delete;
    ResourcePackList(const ResourcePackList&) = delete;
    ResourcePackList(ResourcePackList&&) noexcept = delete;
    ResourcePackList operator=(const ResourcePackList&) = delete;
    ResourcePackList operator=(ResourcePackList&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // ResourcePackList_D_HPP
