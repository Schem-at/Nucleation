#ifndef NUCLEATION_MeshBytes_D_HPP
#define NUCLEATION_MeshBytes_D_HPP

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
namespace capi {
    struct MeshBytes;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * Owned binary byte buffer returned by zero-Base64 binary export methods
 * (e.g. {@link MeshResult::to_glb_bytes}). Query {@link Self::len} and copy straight
 * into a caller-allocated byte buffer / `Uint8Array` via {@link Self::copy_into}.
 */
class MeshBytes {
public:

  /**
   * Byte length of the encoded payload.
   */
  inline uint32_t len() const;

  /**
   * Copy up to `out.len()` bytes starting at `offset` into `out`.
   * Returns the number of bytes written.
   */
  inline uint32_t copy_into(uint32_t offset, nucleation::diplomat::span<uint8_t> out) const;

    inline const nucleation::capi::MeshBytes* AsFFI() const;
    inline nucleation::capi::MeshBytes* AsFFI();
    inline static const nucleation::MeshBytes* FromFFI(const nucleation::capi::MeshBytes* ptr);
    inline static nucleation::MeshBytes* FromFFI(nucleation::capi::MeshBytes* ptr);
    inline static void operator delete(void* ptr);
private:
    MeshBytes() = delete;
    MeshBytes(const nucleation::MeshBytes&) = delete;
    MeshBytes(nucleation::MeshBytes&&) noexcept = delete;
    MeshBytes operator=(const nucleation::MeshBytes&) = delete;
    MeshBytes operator=(nucleation::MeshBytes&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_MeshBytes_D_HPP
