#ifndef StoreIo_D_HPP
#define StoreIo_D_HPP

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
    struct StoreIo;
} // namespace capi
} // namespace

/**
 * Namespace type for the URI-based transparent I/O and format-manager
 * queries (PORTING rule 12).
 */
class StoreIo {
public:

  /**
   * Open a schematic from a URI: a local path, `file://...`, or
   * `s3://bucket/key.schem`. The format is auto-detected from the URI's
   * extension. Single-string URIs for `redis://`, `postgres://`, and
   * `mem://` are rejected by the core resolver; use `Store::open_schematic`
   * with an explicit store for those backends.
   */
  inline static diplomat::result<std::unique_ptr<Schematic>, NucleationError> open(std::string_view uri);

  /**
   * Save a schematic to a URI: a local path, `file://...`, or
   * `s3://bucket/key.schem`. The format is auto-detected from the URI's
   * extension; `version` selects the format version (empty string =
   * format default). Single-string URIs for `redis://`, `postgres://`, and
   * `mem://` are rejected by the core resolver; use `Store::save_schematic`
   * with an explicit store for those backends.
   */
  inline static diplomat::result<std::monostate, NucleationError> save(const Schematic& schematic, std::string_view uri, std::string_view version);

  /**
   * The JSON schema describing the export settings of `format`. Errors
   * with `NotFound` for an unknown format.
   */
  inline static diplomat::result<std::string, NucleationError> export_settings_schema(std::string_view format);
  template<typename W>
  inline static diplomat::result<std::monostate, NucleationError> export_settings_schema_write(std::string_view format, W& writeable_output);

  /**
   * The JSON schema describing the import settings of `format`. Errors
   * with `NotFound` for an unknown format.
   */
  inline static diplomat::result<std::string, NucleationError> import_settings_schema(std::string_view format);
  template<typename W>
  inline static diplomat::result<std::monostate, NucleationError> import_settings_schema_write(std::string_view format, W& writeable_output);

  /**
   * The supported import formats, written as a JSON array string.
   */
  inline static diplomat::result<std::string, NucleationError> supported_import_formats();
  template<typename W>
  inline static diplomat::result<std::monostate, NucleationError> supported_import_formats_write(W& writeable_output);

  /**
   * The supported export formats, written as a JSON array string.
   */
  inline static diplomat::result<std::string, NucleationError> supported_export_formats();
  template<typename W>
  inline static diplomat::result<std::monostate, NucleationError> supported_export_formats_write(W& writeable_output);

  /**
   * The known versions of an export format, written as a JSON array string
   * (empty array for an unknown format, matching the old ABI).
   */
  inline static diplomat::result<std::string, NucleationError> format_versions(std::string_view format);
  template<typename W>
  inline static diplomat::result<std::monostate, NucleationError> format_versions_write(std::string_view format, W& writeable_output);

  /**
   * The default version of an export format. Errors with `NotFound` for an
   * unknown format.
   */
  inline static diplomat::result<std::string, NucleationError> default_format_version(std::string_view format);
  template<typename W>
  inline static diplomat::result<std::monostate, NucleationError> default_format_version_write(std::string_view format, W& writeable_output);

    inline const diplomat::capi::StoreIo* AsFFI() const;
    inline diplomat::capi::StoreIo* AsFFI();
    inline static const StoreIo* FromFFI(const diplomat::capi::StoreIo* ptr);
    inline static StoreIo* FromFFI(diplomat::capi::StoreIo* ptr);
    inline static void operator delete(void* ptr);
private:
    StoreIo() = delete;
    StoreIo(const StoreIo&) = delete;
    StoreIo(StoreIo&&) noexcept = delete;
    StoreIo operator=(const StoreIo&) = delete;
    StoreIo operator=(StoreIo&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // StoreIo_D_HPP
