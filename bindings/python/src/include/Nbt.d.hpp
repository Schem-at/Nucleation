#ifndef NUCLEATION_Nbt_D_HPP
#define NUCLEATION_Nbt_D_HPP

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
class NucleationError;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct Nbt;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * Namespace type for the free-standing NBT builder helpers (the old
 * `nbt_text_build` / `nbt_chest_build` / `nbt_sign_build`), following the
 * static-methods-on-a-dummy-opaque pattern.
 */
class Nbt {
public:

  /**
   * Build a Minecraft JSON text-component string.
   *
   * `color` may be empty (no color). `bold` and `italic` use `-1` for
   * unset, `0` for false, nonzero for true.
   */
  inline static nucleation::diplomat::result<std::string, nucleation::NucleationError> text_build(std::string_view s, std::string_view color, int32_t bold, int32_t italic);
  template<typename W>
  inline static nucleation::diplomat::result<std::monostate, nucleation::NucleationError> text_build_write(std::string_view s, std::string_view color, int32_t bold, int32_t italic, W& writeable_output);

  /**
   * Build a chest-NBT SNBT string for use as the `{...}` portion of a block
   * string.
   *
   * `items_json` is a JSON array of `{"id": string, "count"?: int,
   * "slot"?: int}` entries (may be empty or `[]`); a missing/non-positive
   * `count` defaults to 1, a missing/negative `slot` auto-assigns
   * positionally. `name` is an optional plain-text custom name (empty = no
   * `CustomName`); it is wrapped in a JSON text component automatically
   * unless it already starts with `{`.
   */
  inline static nucleation::diplomat::result<std::string, nucleation::NucleationError> chest_build(std::string_view items_json, std::string_view name);
  template<typename W>
  inline static nucleation::diplomat::result<std::monostate, nucleation::NucleationError> chest_build_write(std::string_view items_json, std::string_view name, W& writeable_output);

  /**
   * Build a modern (1.20+) sign-NBT SNBT string.
   *
   * `front_json` and `back_json` are JSON arrays of up to 4 line strings
   * (either may be empty or `[]`). Each line may be a plain string
   * (auto-wrapped via `text_build`) or an already-built JSON component
   * (starts with `{`). `color` is the dye color string (empty defaults to
   * `"black"`).
   */
  inline static nucleation::diplomat::result<std::string, nucleation::NucleationError> sign_build(std::string_view front_json, std::string_view back_json, std::string_view color, bool glowing, bool waxed);
  template<typename W>
  inline static nucleation::diplomat::result<std::monostate, nucleation::NucleationError> sign_build_write(std::string_view front_json, std::string_view back_json, std::string_view color, bool glowing, bool waxed, W& writeable_output);

    inline const nucleation::capi::Nbt* AsFFI() const;
    inline nucleation::capi::Nbt* AsFFI();
    inline static const nucleation::Nbt* FromFFI(const nucleation::capi::Nbt* ptr);
    inline static nucleation::Nbt* FromFFI(nucleation::capi::Nbt* ptr);
    inline static void operator delete(void* ptr);
private:
    Nbt() = delete;
    Nbt(const nucleation::Nbt&) = delete;
    Nbt(nucleation::Nbt&&) noexcept = delete;
    Nbt operator=(const nucleation::Nbt&) = delete;
    Nbt operator=(nucleation::Nbt&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_Nbt_D_HPP
