#ifndef NUCLEATION_Diff_D_HPP
#define NUCLEATION_Diff_D_HPP

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
namespace capi { struct Diff; }
class Diff;
namespace capi { struct Schematic; }
class Schematic;
class NucleationError;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct Diff;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * A computed diff between two schematics.
 */
class Diff {
public:

  /**
   * Diff two schematics with the given preset (default cost model).
   * Errors with `InvalidArgument` on an unknown preset.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Diff>, nucleation::NucleationError> compute(const nucleation::Schematic& a, const nucleation::Schematic& b, std::string_view preset);

  /**
   * Diff two schematics with optional cost/symmetry overrides. Negative
   * cost ints mean "unset" (use the preset default); an empty `symmetry`
   * string means "unset". Errors with `InvalidArgument` on an unknown
   * preset or symmetry name.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Diff>, nucleation::NucleationError> compute_with_opts(const nucleation::Schematic& a, const nucleation::Schematic& b, std::string_view preset, int32_t cost_add, int32_t cost_delete, int32_t cost_change, int32_t cost_swap, std::string_view symmetry);

  /**
   * Reconstruct a diff from its JSON representation.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Diff>, nucleation::NucleationError> from_json(std::string_view json);

  /**
   * The edit distance of the diff.
   */
  inline uint64_t distance() const;

  /**
   * The support (alignment confidence) of the diff.
   */
  inline float support() const;

  /**
   * Serialize the diff to its full JSON representation.
   */
  inline std::string to_json() const;
  template<typename W>
  inline void to_json_write(W& writeable_output) const;

  /**
   * Serialize the diff to its compact summary JSON.
   */
  inline std::string summary_json() const;
  template<typename W>
  inline void summary_json_write(W& writeable_output) const;

  /**
   * A new schematic containing only the blocks added in this diff.
   */
  inline std::unique_ptr<nucleation::Schematic> added() const;

  /**
   * A new schematic containing only the blocks removed in this diff.
   */
  inline std::unique_ptr<nucleation::Schematic> removed() const;

  /**
   * A new schematic containing only the blocks changed in this diff.
   */
  inline std::unique_ptr<nucleation::Schematic> changed() const;

  /**
   * A new schematic containing only the blocks swapped in this diff.
   */
  inline std::unique_ptr<nucleation::Schematic> swapped() const;

  /**
   * A new schematic with marker blocks summarizing this diff.
   */
  inline std::unique_ptr<nucleation::Schematic> markers() const;

  /**
   * Render a diff overlay on top of an "after" GLB buffer, returning the
   * new GLB as base64 (PORTING rule 6). Requires the `meshing` feature;
   * errors with `Mesh` when compiled without it.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> to_overlay_glb_b64(nucleation::diplomat::span<const uint8_t> after_glb) const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> to_overlay_glb_b64_write(nucleation::diplomat::span<const uint8_t> after_glb, W& writeable_output) const;

    inline const nucleation::capi::Diff* AsFFI() const;
    inline nucleation::capi::Diff* AsFFI();
    inline static const nucleation::Diff* FromFFI(const nucleation::capi::Diff* ptr);
    inline static nucleation::Diff* FromFFI(nucleation::capi::Diff* ptr);
    inline static void operator delete(void* ptr);
private:
    Diff() = delete;
    Diff(const nucleation::Diff&) = delete;
    Diff(nucleation::Diff&&) noexcept = delete;
    Diff operator=(const nucleation::Diff&) = delete;
    Diff operator=(nucleation::Diff&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_Diff_D_HPP
