#ifndef BlockState_D_HPP
#define BlockState_D_HPP

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
    struct BlockState;
} // namespace capi
} // namespace

/**
 * A block state: a block name plus its properties. Port of the old
 * `BlockStateWrapper` / `blockstate_*` fns.
 */
class BlockState {
public:

  /**
   * Create a block state with the given name and no properties.
   */
  inline static std::unique_ptr<BlockState> create(std::string_view name);

  /**
   * A copy of this block state with `key=value` set; the original is
   * unchanged.
   */
  inline diplomat::result<std::unique_ptr<BlockState>, NucleationError> with_property(std::string_view key, std::string_view value) const;

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

    inline const diplomat::capi::BlockState* AsFFI() const;
    inline diplomat::capi::BlockState* AsFFI();
    inline static const BlockState* FromFFI(const diplomat::capi::BlockState* ptr);
    inline static BlockState* FromFFI(diplomat::capi::BlockState* ptr);
    inline static void operator delete(void* ptr);
private:
    BlockState() = delete;
    BlockState(const BlockState&) = delete;
    BlockState(BlockState&&) noexcept = delete;
    BlockState operator=(const BlockState&) = delete;
    BlockState operator=(BlockState&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // BlockState_D_HPP
