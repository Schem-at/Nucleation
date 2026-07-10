#ifndef NUCLEATION_Fingerprint_D_HPP
#define NUCLEATION_Fingerprint_D_HPP

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
namespace capi { struct Schematic; }
class Schematic;
class NucleationError;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct Fingerprint;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * Namespace type for the fingerprint free functions (PORTING rule 12).
 */
class Fingerprint {
public:

  /**
   * The fingerprint of a schematic for the given preset, as a hex string.
   * Errors with `InvalidArgument` on an unknown preset.
   */
  inline static nucleation::diplomat::result<std::string, nucleation::NucleationError> compute(const nucleation::Schematic& schematic, std::string_view preset);
  template<typename W>
  inline static nucleation::diplomat::result<std::monostate, nucleation::NucleationError> compute_write(const nucleation::Schematic& schematic, std::string_view preset, W& writeable_output);

  /**
   * The structural signature (JSON) of a schematic for the given preset.
   */
  inline static nucleation::diplomat::result<std::string, nucleation::NucleationError> signature_json(const nucleation::Schematic& schematic, std::string_view preset);
  template<typename W>
  inline static nucleation::diplomat::result<std::monostate, nucleation::NucleationError> signature_json_write(const nucleation::Schematic& schematic, std::string_view preset, W& writeable_output);

  /**
   * Translation-invariant fuzzy distance between two builds' footprints.
   */
  inline static nucleation::diplomat::result<float, nucleation::NucleationError> footprint_distance(const nucleation::Schematic& a, const nucleation::Schematic& b, std::string_view preset);

  /**
   * The schematic's translation/scale-invariant FFT shape footprint as a
   * JSON array of floats.
   */
  inline static nucleation::diplomat::result<std::string, nucleation::NucleationError> footprint_json(const nucleation::Schematic& schematic, std::string_view preset);
  template<typename W>
  inline static nucleation::diplomat::result<std::monostate, nucleation::NucleationError> footprint_json_write(const nucleation::Schematic& schematic, std::string_view preset, W& writeable_output);

  /**
   * Whether two schematics share the same fingerprint for the given preset.
   */
  inline static nucleation::diplomat::result<bool, nucleation::NucleationError> is_duplicate(const nucleation::Schematic& a, const nucleation::Schematic& b, std::string_view preset);

    inline const nucleation::capi::Fingerprint* AsFFI() const;
    inline nucleation::capi::Fingerprint* AsFFI();
    inline static const nucleation::Fingerprint* FromFFI(const nucleation::capi::Fingerprint* ptr);
    inline static nucleation::Fingerprint* FromFFI(nucleation::capi::Fingerprint* ptr);
    inline static void operator delete(void* ptr);
private:
    Fingerprint() = delete;
    Fingerprint(const nucleation::Fingerprint&) = delete;
    Fingerprint(nucleation::Fingerprint&&) noexcept = delete;
    Fingerprint operator=(const nucleation::Fingerprint&) = delete;
    Fingerprint operator=(nucleation::Fingerprint&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_Fingerprint_D_HPP
