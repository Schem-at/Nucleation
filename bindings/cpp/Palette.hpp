#ifndef Palette_HPP
#define Palette_HPP

#include "Palette.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "NucleationError.hpp"
#include "diplomat_runtime.hpp"


namespace diplomat {
namespace capi {
    extern "C" {

    diplomat::capi::Palette* Palette_all(void);

    diplomat::capi::Palette* Palette_solid(void);

    diplomat::capi::Palette* Palette_structural(void);

    diplomat::capi::Palette* Palette_decorative(void);

    diplomat::capi::Palette* Palette_concrete(void);

    diplomat::capi::Palette* Palette_wool(void);

    diplomat::capi::Palette* Palette_terracotta(void);

    diplomat::capi::Palette* Palette_grayscale(void);

    typedef struct Palette_from_block_ids_result {union {diplomat::capi::Palette* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Palette_from_block_ids_result;
    Palette_from_block_ids_result Palette_from_block_ids(diplomat::capi::DiplomatStringView ids_json);

    size_t Palette_len(const diplomat::capi::Palette* self);

    void Palette_block_ids_json(const diplomat::capi::Palette* self, diplomat::capi::DiplomatWrite* write);

    typedef struct Palette_closest_block_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} Palette_closest_block_result;
    Palette_closest_block_result Palette_closest_block(const diplomat::capi::Palette* self, uint8_t r, uint8_t g, uint8_t b, diplomat::capi::DiplomatWrite* write);

    void Palette_destroy(Palette* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::unique_ptr<Palette> Palette::all() {
    auto result = diplomat::capi::Palette_all();
    return std::unique_ptr<Palette>(Palette::FromFFI(result));
}

inline std::unique_ptr<Palette> Palette::solid() {
    auto result = diplomat::capi::Palette_solid();
    return std::unique_ptr<Palette>(Palette::FromFFI(result));
}

inline std::unique_ptr<Palette> Palette::structural() {
    auto result = diplomat::capi::Palette_structural();
    return std::unique_ptr<Palette>(Palette::FromFFI(result));
}

inline std::unique_ptr<Palette> Palette::decorative() {
    auto result = diplomat::capi::Palette_decorative();
    return std::unique_ptr<Palette>(Palette::FromFFI(result));
}

inline std::unique_ptr<Palette> Palette::concrete() {
    auto result = diplomat::capi::Palette_concrete();
    return std::unique_ptr<Palette>(Palette::FromFFI(result));
}

inline std::unique_ptr<Palette> Palette::wool() {
    auto result = diplomat::capi::Palette_wool();
    return std::unique_ptr<Palette>(Palette::FromFFI(result));
}

inline std::unique_ptr<Palette> Palette::terracotta() {
    auto result = diplomat::capi::Palette_terracotta();
    return std::unique_ptr<Palette>(Palette::FromFFI(result));
}

inline std::unique_ptr<Palette> Palette::grayscale() {
    auto result = diplomat::capi::Palette_grayscale();
    return std::unique_ptr<Palette>(Palette::FromFFI(result));
}

inline diplomat::result<std::unique_ptr<Palette>, NucleationError> Palette::from_block_ids(std::string_view ids_json) {
    auto result = diplomat::capi::Palette_from_block_ids({ids_json.data(), ids_json.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<Palette>, NucleationError>(diplomat::Ok<std::unique_ptr<Palette>>(std::unique_ptr<Palette>(Palette::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Palette>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline size_t Palette::len() const {
    auto result = diplomat::capi::Palette_len(this->AsFFI());
    return result;
}

inline std::string Palette::block_ids_json() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::Palette_block_ids_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void Palette::block_ids_json_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::Palette_block_ids_json(this->AsFFI(),
        &write);
}

inline diplomat::result<std::string, NucleationError> Palette::closest_block(uint8_t r, uint8_t g, uint8_t b) const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::Palette_closest_block(this->AsFFI(),
        r,
        g,
        b,
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> Palette::closest_block_write(uint8_t r, uint8_t g, uint8_t b, W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::Palette_closest_block(this->AsFFI(),
        r,
        g,
        b,
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline const diplomat::capi::Palette* Palette::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::Palette*>(this);
}

inline diplomat::capi::Palette* Palette::AsFFI() {
    return reinterpret_cast<diplomat::capi::Palette*>(this);
}

inline const Palette* Palette::FromFFI(const diplomat::capi::Palette* ptr) {
    return reinterpret_cast<const Palette*>(ptr);
}

inline Palette* Palette::FromFFI(diplomat::capi::Palette* ptr) {
    return reinterpret_cast<Palette*>(ptr);
}

inline void Palette::operator delete(void* ptr) {
    diplomat::capi::Palette_destroy(reinterpret_cast<diplomat::capi::Palette*>(ptr));
}


#endif // Palette_HPP
