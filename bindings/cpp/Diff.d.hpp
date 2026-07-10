#ifndef Diff_D_HPP
#define Diff_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"

namespace diplomat::capi { struct Schematic; }
class Schematic;
class NucleationError;




namespace diplomat {
namespace capi {
    struct Diff;
} // namespace capi
} // namespace

/**
 * A computed diff between two schematics.
 */
class Diff {
public:

  /**
   * Diff two schematics with the given preset (default cost model).
   * Errors with `InvalidArgument` on an unknown preset.
   */
  inline static diplomat::result<std::unique_ptr<Diff>, NucleationError> compute(const Schematic& a, const Schematic& b, std::string_view preset);

  /**
   * Diff two schematics with optional cost/symmetry overrides. Negative
   * cost ints mean "unset" (use the preset default); an empty `symmetry`
   * string means "unset". Errors with `InvalidArgument` on an unknown
   * preset or symmetry name.
   */
  inline static diplomat::result<std::unique_ptr<Diff>, NucleationError> compute_with_opts(const Schematic& a, const Schematic& b, std::string_view preset, int32_t cost_add, int32_t cost_delete, int32_t cost_change, int32_t cost_swap, std::string_view symmetry);

  /**
   * Reconstruct a diff from its JSON representation.
   */
  inline static diplomat::result<std::unique_ptr<Diff>, NucleationError> from_json(std::string_view json);

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
  inline std::unique_ptr<Schematic> added() const;

  /**
   * A new schematic containing only the blocks removed in this diff.
   */
  inline std::unique_ptr<Schematic> removed() const;

  /**
   * A new schematic containing only the blocks changed in this diff.
   */
  inline std::unique_ptr<Schematic> changed() const;

  /**
   * A new schematic containing only the blocks swapped in this diff.
   */
  inline std::unique_ptr<Schematic> swapped() const;

  /**
   * A new schematic with marker blocks summarizing this diff.
   */
  inline std::unique_ptr<Schematic> markers() const;

  /**
   * Render a diff overlay on top of an "after" GLB buffer, returning the
   * new GLB as base64 (PORTING rule 6). Requires the `meshing` feature;
   * errors with `Mesh` when compiled without it.
   */
  inline diplomat::result<std::string, NucleationError> to_overlay_glb_b64(diplomat::span<const uint8_t> after_glb) const;
  template<typename W>
  inline diplomat::result<std::monostate, NucleationError> to_overlay_glb_b64_write(diplomat::span<const uint8_t> after_glb, W& writeable_output) const;

    inline const diplomat::capi::Diff* AsFFI() const;
    inline diplomat::capi::Diff* AsFFI();
    inline static const Diff* FromFFI(const diplomat::capi::Diff* ptr);
    inline static Diff* FromFFI(diplomat::capi::Diff* ptr);
    inline static void operator delete(void* ptr);
private:
    Diff() = delete;
    Diff(const Diff&) = delete;
    Diff(Diff&&) noexcept = delete;
    Diff operator=(const Diff&) = delete;
    Diff operator=(Diff&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // Diff_D_HPP
