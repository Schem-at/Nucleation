#ifndef Store_D_HPP
#define Store_D_HPP

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
    struct Store;
} // namespace capi
} // namespace

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
  inline static diplomat::result<std::unique_ptr<Store>, NucleationError> open(std::string_view url);

  /**
   * Fetch `key`, writing the value as base64 (PORTING rule 6). Errors with
   * `NotFound` when the key is absent.
   */
  inline diplomat::result<std::string, NucleationError> get_b64(std::string_view key) const;
  template<typename W>
  inline diplomat::result<std::monostate, NucleationError> get_b64_write(std::string_view key, W& writeable_output) const;

  /**
   * Store `data` at `key`.
   */
  inline diplomat::result<std::monostate, NucleationError> put(std::string_view key, diplomat::span<const uint8_t> data) const;

  /**
   * Whether `key` exists.
   */
  inline diplomat::result<bool, NucleationError> exists(std::string_view key) const;

  /**
   * Delete `key` (idempotent).
   */
  inline diplomat::result<std::monostate, NucleationError> delete_(std::string_view key) const;

  /**
   * List keys under `prefix`, written as a JSON array string.
   */
  inline diplomat::result<std::string, NucleationError> list(std::string_view prefix) const;
  template<typename W>
  inline diplomat::result<std::monostate, NucleationError> list_write(std::string_view prefix, W& writeable_output) const;

  /**
   * Atomically write `data` at `key` only if it does not already exist.
   * Returns `true` if written, `false` if the key existed.
   */
  inline diplomat::result<bool, NucleationError> put_if_absent(std::string_view key, diplomat::span<const uint8_t> data) const;

  /**
   * A keyset page of keys under `prefix`. `after` is the exclusive cursor
   * (empty string for the first page); at most `limit` keys are returned.
   * Writes a JSON object string `{"keys":[...],"next":"…"|null}`.
   */
  inline diplomat::result<std::string, NucleationError> list_paginated(std::string_view prefix, std::string_view after, uint32_t limit) const;
  template<typename W>
  inline diplomat::result<std::monostate, NucleationError> list_paginated_write(std::string_view prefix, std::string_view after, uint32_t limit, W& writeable_output) const;

  /**
   * Health check: `Ok` when the store is usable.
   */
  inline diplomat::result<std::monostate, NucleationError> health() const;

  /**
   * Open a schematic stored at `key` in this store. Works for every
   * backend, including `redis://`/`postgres://`/`mem://` that the
   * single-string URI form (`StoreIo::open`) rejects.
   */
  inline diplomat::result<std::unique_ptr<Schematic>, NucleationError> open_schematic(std::string_view key) const;

  /**
   * Save a schematic at `key` in this store. `version` selects the format
   * version (empty string = format default). Works for every backend,
   * including `redis://`/`postgres://`/`mem://` that the single-string URI
   * form (`StoreIo::save`) rejects.
   */
  inline diplomat::result<std::monostate, NucleationError> save_schematic(const Schematic& schematic, std::string_view key, std::string_view version) const;

    inline const diplomat::capi::Store* AsFFI() const;
    inline diplomat::capi::Store* AsFFI();
    inline static const Store* FromFFI(const diplomat::capi::Store* ptr);
    inline static Store* FromFFI(diplomat::capi::Store* ptr);
    inline static void operator delete(void* ptr);
private:
    Store() = delete;
    Store(const Store&) = delete;
    Store(Store&&) noexcept = delete;
    Store operator=(const Store&) = delete;
    Store operator=(Store&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // Store_D_HPP
