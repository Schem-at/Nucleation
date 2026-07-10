#ifndef NUCLEATION_StoreIo_D_HPP
#define NUCLEATION_StoreIo_D_HPP

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
    struct StoreIo;
} // namespace capi
} // namespace

namespace nucleation {
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
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> open(std::string_view uri);

  /**
   * Save a schematic to a URI: a local path, `file://...`, or
   * `s3://bucket/key.schem`. The format is auto-detected from the URI's
   * extension; `version` selects the format version (empty string =
   * format default). Single-string URIs for `redis://`, `postgres://`, and
   * `mem://` are rejected by the core resolver; use `Store::save_schematic`
   * with an explicit store for those backends.
   */
  inline static nucleation::diplomat::result<std::monostate, nucleation::NucleationError> save(const nucleation::Schematic& schematic, std::string_view uri, std::string_view version);

  /**
   * The JSON schema describing the export settings of `format`. Errors
   * with `NotFound` for an unknown format.
   */
  inline static nucleation::diplomat::result<std::string, nucleation::NucleationError> export_settings_schema(std::string_view format);
  template<typename W>
  inline static nucleation::diplomat::result<std::monostate, nucleation::NucleationError> export_settings_schema_write(std::string_view format, W& writeable_output);

  /**
   * The JSON schema describing the import settings of `format`. Errors
   * with `NotFound` for an unknown format.
   */
  inline static nucleation::diplomat::result<std::string, nucleation::NucleationError> import_settings_schema(std::string_view format);
  template<typename W>
  inline static nucleation::diplomat::result<std::monostate, nucleation::NucleationError> import_settings_schema_write(std::string_view format, W& writeable_output);

  /**
   * The supported import formats, written as a JSON array string.
   */
  inline static nucleation::diplomat::result<std::string, nucleation::NucleationError> supported_import_formats();
  template<typename W>
  inline static nucleation::diplomat::result<std::monostate, nucleation::NucleationError> supported_import_formats_write(W& writeable_output);

  /**
   * The supported export formats, written as a JSON array string.
   */
  inline static nucleation::diplomat::result<std::string, nucleation::NucleationError> supported_export_formats();
  template<typename W>
  inline static nucleation::diplomat::result<std::monostate, nucleation::NucleationError> supported_export_formats_write(W& writeable_output);

  /**
   * The known versions of an export format, written as a JSON array string
   * (empty array for an unknown format, matching the old ABI).
   */
  inline static nucleation::diplomat::result<std::string, nucleation::NucleationError> format_versions(std::string_view format);
  template<typename W>
  inline static nucleation::diplomat::result<std::monostate, nucleation::NucleationError> format_versions_write(std::string_view format, W& writeable_output);

  /**
   * The default version of an export format. Errors with `NotFound` for an
   * unknown format.
   */
  inline static nucleation::diplomat::result<std::string, nucleation::NucleationError> default_format_version(std::string_view format);
  template<typename W>
  inline static nucleation::diplomat::result<std::monostate, nucleation::NucleationError> default_format_version_write(std::string_view format, W& writeable_output);

    inline const nucleation::capi::StoreIo* AsFFI() const;
    inline nucleation::capi::StoreIo* AsFFI();
    inline static const nucleation::StoreIo* FromFFI(const nucleation::capi::StoreIo* ptr);
    inline static nucleation::StoreIo* FromFFI(nucleation::capi::StoreIo* ptr);
    inline static void operator delete(void* ptr);
private:
    StoreIo() = delete;
    StoreIo(const nucleation::StoreIo&) = delete;
    StoreIo(nucleation::StoreIo&&) noexcept = delete;
    StoreIo operator=(const nucleation::StoreIo&) = delete;
    StoreIo operator=(nucleation::StoreIo&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_StoreIo_D_HPP
