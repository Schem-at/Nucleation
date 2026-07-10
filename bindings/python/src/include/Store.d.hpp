#ifndef NUCLEATION_Store_D_HPP
#define NUCLEATION_Store_D_HPP

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
namespace capi { struct Store; }
class Store;
class NucleationError;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct Store;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * A key/value store opened from a URL (e.g. `mem://`, `file:///path`,
 * `s3://bucket/prefix`, `redis://…`, `postgres://…`).
 */
class Store {
public:

  /**
   * Open a store from a URL. Errors with `Store` on an unknown scheme or
   * connection failure.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Store>, nucleation::NucleationError> open(std::string_view url);

  /**
   * Fetch `key`, writing the value as base64 (PORTING rule 6). Errors with
   * `NotFound` when the key is absent.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> get_b64(std::string_view key) const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> get_b64_write(std::string_view key, W& writeable_output) const;

  /**
   * Store `data` at `key`.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> put(std::string_view key, nucleation::diplomat::span<const uint8_t> data) const;

  /**
   * Whether `key` exists.
   */
  inline nucleation::diplomat::result<bool, nucleation::NucleationError> exists(std::string_view key) const;

  /**
   * Delete `key` (idempotent).
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> delete_(std::string_view key) const;

  /**
   * List keys under `prefix`, written as a JSON array string.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> list(std::string_view prefix) const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> list_write(std::string_view prefix, W& writeable_output) const;

  /**
   * Atomically write `data` at `key` only if it does not already exist.
   * Returns `true` if written, `false` if the key existed.
   */
  inline nucleation::diplomat::result<bool, nucleation::NucleationError> put_if_absent(std::string_view key, nucleation::diplomat::span<const uint8_t> data) const;

  /**
   * A keyset page of keys under `prefix`. `after` is the exclusive cursor
   * (empty string for the first page); at most `limit` keys are returned.
   * Writes a JSON object string `{"keys":[...],"next":"…"|null}`.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> list_paginated(std::string_view prefix, std::string_view after, uint32_t limit) const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> list_paginated_write(std::string_view prefix, std::string_view after, uint32_t limit, W& writeable_output) const;

  /**
   * Health check: `Ok` when the store is usable.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> health() const;

  /**
   * Open a schematic stored at `key` in this store. Works for every
   * backend, including `redis://`/`postgres://`/`mem://` that the
   * single-string URI form (`StoreIo::open`) rejects.
   */
  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> open_schematic(std::string_view key) const;

  /**
   * Save a schematic at `key` in this store. `version` selects the format
   * version (empty string = format default). Works for every backend,
   * including `redis://`/`postgres://`/`mem://` that the single-string URI
   * form (`StoreIo::save`) rejects.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> save_schematic(const nucleation::Schematic& schematic, std::string_view key, std::string_view version) const;

    inline const nucleation::capi::Store* AsFFI() const;
    inline nucleation::capi::Store* AsFFI();
    inline static const nucleation::Store* FromFFI(const nucleation::capi::Store* ptr);
    inline static nucleation::Store* FromFFI(nucleation::capi::Store* ptr);
    inline static void operator delete(void* ptr);
private:
    Store() = delete;
    Store(const nucleation::Store&) = delete;
    Store(nucleation::Store&&) noexcept = delete;
    Store operator=(const nucleation::Store&) = delete;
    Store operator=(nucleation::Store&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_Store_D_HPP
