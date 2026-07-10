#ifndef Fingerprint_D_HPP
#define Fingerprint_D_HPP

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
    struct Fingerprint;
} // namespace capi
} // namespace

/**
 * Namespace type for the fingerprint free functions (PORTING rule 12).
 */
class Fingerprint {
public:

  /**
   * The fingerprint of a schematic for the given preset, as a hex string.
   * Errors with `InvalidArgument` on an unknown preset.
   */
  inline static diplomat::result<std::string, NucleationError> compute(const Schematic& schematic, std::string_view preset);
  template<typename W>
  inline static diplomat::result<std::monostate, NucleationError> compute_write(const Schematic& schematic, std::string_view preset, W& writeable_output);

  /**
   * The structural signature (JSON) of a schematic for the given preset.
   */
  inline static diplomat::result<std::string, NucleationError> signature_json(const Schematic& schematic, std::string_view preset);
  template<typename W>
  inline static diplomat::result<std::monostate, NucleationError> signature_json_write(const Schematic& schematic, std::string_view preset, W& writeable_output);

  /**
   * Translation-invariant fuzzy distance between two builds' footprints.
   */
  inline static diplomat::result<float, NucleationError> footprint_distance(const Schematic& a, const Schematic& b, std::string_view preset);

  /**
   * The schematic's translation/scale-invariant FFT shape footprint as a
   * JSON array of floats.
   */
  inline static diplomat::result<std::string, NucleationError> footprint_json(const Schematic& schematic, std::string_view preset);
  template<typename W>
  inline static diplomat::result<std::monostate, NucleationError> footprint_json_write(const Schematic& schematic, std::string_view preset, W& writeable_output);

  /**
   * Whether two schematics share the same fingerprint for the given preset.
   */
  inline static diplomat::result<bool, NucleationError> is_duplicate(const Schematic& a, const Schematic& b, std::string_view preset);

    inline const diplomat::capi::Fingerprint* AsFFI() const;
    inline diplomat::capi::Fingerprint* AsFFI();
    inline static const Fingerprint* FromFFI(const diplomat::capi::Fingerprint* ptr);
    inline static Fingerprint* FromFFI(diplomat::capi::Fingerprint* ptr);
    inline static void operator delete(void* ptr);
private:
    Fingerprint() = delete;
    Fingerprint(const Fingerprint&) = delete;
    Fingerprint(Fingerprint&&) noexcept = delete;
    Fingerprint operator=(const Fingerprint&) = delete;
    Fingerprint operator=(Fingerprint&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // Fingerprint_D_HPP
