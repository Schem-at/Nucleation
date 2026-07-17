#ifndef NUCLEATION_BlockState_D_HPP
#define NUCLEATION_BlockState_D_HPP

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
namespace capi { struct BlockState; }
class BlockState;
class NucleationError;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct BlockState;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * A block state: a block name plus its properties. Port of the old
 * `BlockStateWrapper` / `blockstate_*` fns.
 */
class BlockState {
public:

  /**
   * Create a block state with the given name and no properties.
   */
  inline static std::unique_ptr<nucleation::BlockState> create(std::string_view name);

  /**
   * A copy of this block state with `key=value` set; the original is
   * unchanged.
   */
  inline nucleation::diplomat::result<std::unique_ptr<nucleation::BlockState>, nucleation::NucleationError> with_property(std::string_view key, std::string_view value) const;

  /**
   * The block name (e.g. `minecraft:stone`).
   */
  inline std::string name() const;
  template<typename W>
  inline void name_write(W& writeable_output) const;

  /**
   * The properties as a JSON object of string→string (the old
   * `CPropertyArray`).
   */
  inline std::string properties_json() const;
  template<typename W>
  inline void properties_json_write(W& writeable_output) const;

    inline const nucleation::capi::BlockState* AsFFI() const;
    inline nucleation::capi::BlockState* AsFFI();
    inline static const nucleation::BlockState* FromFFI(const nucleation::capi::BlockState* ptr);
    inline static nucleation::BlockState* FromFFI(nucleation::capi::BlockState* ptr);
    inline static void operator delete(void* ptr);
private:
    BlockState() = delete;
    BlockState(const nucleation::BlockState&) = delete;
    BlockState(nucleation::BlockState&&) noexcept = delete;
    BlockState operator=(const nucleation::BlockState&) = delete;
    BlockState operator=(nucleation::BlockState&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_BlockState_D_HPP
