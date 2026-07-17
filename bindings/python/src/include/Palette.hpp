#ifndef NUCLEATION_Palette_HPP
#define NUCLEATION_Palette_HPP

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


namespace nucleation {
namespace capi {
    extern "C" {

    nucleation::capi::Palette* Palette_all(void);

    nucleation::capi::Palette* Palette_solid(void);

    nucleation::capi::Palette* Palette_structural(void);

    nucleation::capi::Palette* Palette_decorative(void);

    nucleation::capi::Palette* Palette_concrete(void);

    nucleation::capi::Palette* Palette_wool(void);

    nucleation::capi::Palette* Palette_terracotta(void);

    nucleation::capi::Palette* Palette_grayscale(void);

    nucleation::capi::Palette* Palette_wood(void);

    nucleation::capi::Palette* Palette_sorted_by_lightness(const nucleation::capi::Palette* self);

    typedef struct Palette_gradient_ids_json_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Palette_gradient_ids_json_result;
    Palette_gradient_ids_json_result Palette_gradient_ids_json(const nucleation::capi::Palette* self, uint8_t r1, uint8_t g1, uint8_t b1, uint8_t r2, uint8_t g2, uint8_t b2, uint32_t steps, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct Palette_from_block_ids_result {union {nucleation::capi::Palette* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Palette_from_block_ids_result;
    Palette_from_block_ids_result Palette_from_block_ids(nucleation::diplomat::capi::DiplomatStringView ids_json);

    size_t Palette_len(const nucleation::capi::Palette* self);

    void Palette_block_ids_json(const nucleation::capi::Palette* self, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct Palette_closest_block_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} Palette_closest_block_result;
    Palette_closest_block_result Palette_closest_block(const nucleation::capi::Palette* self, uint8_t r, uint8_t g, uint8_t b, nucleation::diplomat::capi::DiplomatWrite* write);

    void Palette_destroy(Palette* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::unique_ptr<nucleation::Palette> nucleation::Palette::all() {
    auto result = nucleation::capi::Palette_all();
    return std::unique_ptr<nucleation::Palette>(nucleation::Palette::FromFFI(result));
}

inline std::unique_ptr<nucleation::Palette> nucleation::Palette::solid() {
    auto result = nucleation::capi::Palette_solid();
    return std::unique_ptr<nucleation::Palette>(nucleation::Palette::FromFFI(result));
}

inline std::unique_ptr<nucleation::Palette> nucleation::Palette::structural() {
    auto result = nucleation::capi::Palette_structural();
    return std::unique_ptr<nucleation::Palette>(nucleation::Palette::FromFFI(result));
}

inline std::unique_ptr<nucleation::Palette> nucleation::Palette::decorative() {
    auto result = nucleation::capi::Palette_decorative();
    return std::unique_ptr<nucleation::Palette>(nucleation::Palette::FromFFI(result));
}

inline std::unique_ptr<nucleation::Palette> nucleation::Palette::concrete() {
    auto result = nucleation::capi::Palette_concrete();
    return std::unique_ptr<nucleation::Palette>(nucleation::Palette::FromFFI(result));
}

inline std::unique_ptr<nucleation::Palette> nucleation::Palette::wool() {
    auto result = nucleation::capi::Palette_wool();
    return std::unique_ptr<nucleation::Palette>(nucleation::Palette::FromFFI(result));
}

inline std::unique_ptr<nucleation::Palette> nucleation::Palette::terracotta() {
    auto result = nucleation::capi::Palette_terracotta();
    return std::unique_ptr<nucleation::Palette>(nucleation::Palette::FromFFI(result));
}

inline std::unique_ptr<nucleation::Palette> nucleation::Palette::grayscale() {
    auto result = nucleation::capi::Palette_grayscale();
    return std::unique_ptr<nucleation::Palette>(nucleation::Palette::FromFFI(result));
}

inline std::unique_ptr<nucleation::Palette> nucleation::Palette::wood() {
    auto result = nucleation::capi::Palette_wood();
    return std::unique_ptr<nucleation::Palette>(nucleation::Palette::FromFFI(result));
}

inline std::unique_ptr<nucleation::Palette> nucleation::Palette::sorted_by_lightness() const {
    auto result = nucleation::capi::Palette_sorted_by_lightness(this->AsFFI());
    return std::unique_ptr<nucleation::Palette>(nucleation::Palette::FromFFI(result));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::Palette::gradient_ids_json(uint8_t r1, uint8_t g1, uint8_t b1, uint8_t r2, uint8_t g2, uint8_t b2, uint32_t steps) const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::Palette_gradient_ids_json(this->AsFFI(),
        r1,
        g1,
        b1,
        r2,
        g2,
        b2,
        steps,
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Palette::gradient_ids_json_write(uint8_t r1, uint8_t g1, uint8_t b1, uint8_t r2, uint8_t g2, uint8_t b2, uint32_t steps, W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::Palette_gradient_ids_json(this->AsFFI(),
        r1,
        g1,
        b1,
        r2,
        g2,
        b2,
        steps,
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Palette>, nucleation::NucleationError> nucleation::Palette::from_block_ids(std::string_view ids_json) {
    auto result = nucleation::capi::Palette_from_block_ids({ids_json.data(), ids_json.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Palette>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Palette>>(std::unique_ptr<nucleation::Palette>(nucleation::Palette::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Palette>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline size_t nucleation::Palette::len() const {
    auto result = nucleation::capi::Palette_len(this->AsFFI());
    return result;
}

inline std::string nucleation::Palette::block_ids_json() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::Palette_block_ids_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void nucleation::Palette::block_ids_json_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::Palette_block_ids_json(this->AsFFI(),
        &write);
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::Palette::closest_block(uint8_t r, uint8_t g, uint8_t b) const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::Palette_closest_block(this->AsFFI(),
        r,
        g,
        b,
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::Palette::closest_block_write(uint8_t r, uint8_t g, uint8_t b, W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::Palette_closest_block(this->AsFFI(),
        r,
        g,
        b,
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline const nucleation::capi::Palette* nucleation::Palette::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::Palette*>(this);
}

inline nucleation::capi::Palette* nucleation::Palette::AsFFI() {
    return reinterpret_cast<nucleation::capi::Palette*>(this);
}

inline const nucleation::Palette* nucleation::Palette::FromFFI(const nucleation::capi::Palette* ptr) {
    return reinterpret_cast<const nucleation::Palette*>(ptr);
}

inline nucleation::Palette* nucleation::Palette::FromFFI(nucleation::capi::Palette* ptr) {
    return reinterpret_cast<nucleation::Palette*>(ptr);
}

inline void nucleation::Palette::operator delete(void* ptr) {
    nucleation::capi::Palette_destroy(reinterpret_cast<nucleation::capi::Palette*>(ptr));
}


#endif // NUCLEATION_Palette_HPP
