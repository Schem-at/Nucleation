#ifndef NUCLEATION_WorldChunkView_HPP
#define NUCLEATION_WorldChunkView_HPP

#include "WorldChunkView.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "NucleationError.hpp"
#include "Schematic.hpp"
#include "diplomat_runtime.hpp"


namespace nucleation {
namespace capi {
    extern "C" {

    nucleation::capi::WorldChunkView* WorldChunkView_create(int32_t cx, int32_t cz);

    int32_t WorldChunkView_cx(const nucleation::capi::WorldChunkView* self);

    int32_t WorldChunkView_cz(const nucleation::capi::WorldChunkView* self);

    nucleation::capi::Schematic* WorldChunkView_to_schematic(const nucleation::capi::WorldChunkView* self);

    typedef struct WorldChunkView_set_block_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} WorldChunkView_set_block_result;
    WorldChunkView_set_block_result WorldChunkView_set_block(nucleation::capi::WorldChunkView* self, int32_t x, int32_t y, int32_t z, nucleation::diplomat::capi::DiplomatStringView block_name);

    typedef struct WorldChunkView_set_biome_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} WorldChunkView_set_biome_result;
    WorldChunkView_set_biome_result WorldChunkView_set_biome(nucleation::capi::WorldChunkView* self, nucleation::diplomat::capi::DiplomatStringView biome_name);

    typedef struct WorldChunkView_biome_palette_json_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} WorldChunkView_biome_palette_json_result;
    WorldChunkView_biome_palette_json_result WorldChunkView_biome_palette_json(const nucleation::capi::WorldChunkView* self, nucleation::diplomat::capi::DiplomatWrite* write);

    void WorldChunkView_destroy(WorldChunkView* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::unique_ptr<nucleation::WorldChunkView> nucleation::WorldChunkView::create(int32_t cx, int32_t cz) {
    auto result = nucleation::capi::WorldChunkView_create(cx,
        cz);
    return std::unique_ptr<nucleation::WorldChunkView>(nucleation::WorldChunkView::FromFFI(result));
}

inline int32_t nucleation::WorldChunkView::cx() const {
    auto result = nucleation::capi::WorldChunkView_cx(this->AsFFI());
    return result;
}

inline int32_t nucleation::WorldChunkView::cz() const {
    auto result = nucleation::capi::WorldChunkView_cz(this->AsFFI());
    return result;
}

inline std::unique_ptr<nucleation::Schematic> nucleation::WorldChunkView::to_schematic() const {
    auto result = nucleation::capi::WorldChunkView_to_schematic(this->AsFFI());
    return std::unique_ptr<nucleation::Schematic>(nucleation::Schematic::FromFFI(result));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::WorldChunkView::set_block(int32_t x, int32_t y, int32_t z, std::string_view block_name) {
    auto result = nucleation::capi::WorldChunkView_set_block(this->AsFFI(),
        x,
        y,
        z,
        {block_name.data(), block_name.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::WorldChunkView::set_biome(std::string_view biome_name) {
    auto result = nucleation::capi::WorldChunkView_set_biome(this->AsFFI(),
        {biome_name.data(), biome_name.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::WorldChunkView::biome_palette_json() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::WorldChunkView_biome_palette_json(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::WorldChunkView::biome_palette_json_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::WorldChunkView_biome_palette_json(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline const nucleation::capi::WorldChunkView* nucleation::WorldChunkView::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::WorldChunkView*>(this);
}

inline nucleation::capi::WorldChunkView* nucleation::WorldChunkView::AsFFI() {
    return reinterpret_cast<nucleation::capi::WorldChunkView*>(this);
}

inline const nucleation::WorldChunkView* nucleation::WorldChunkView::FromFFI(const nucleation::capi::WorldChunkView* ptr) {
    return reinterpret_cast<const nucleation::WorldChunkView*>(ptr);
}

inline nucleation::WorldChunkView* nucleation::WorldChunkView::FromFFI(nucleation::capi::WorldChunkView* ptr) {
    return reinterpret_cast<nucleation::WorldChunkView*>(ptr);
}

inline void nucleation::WorldChunkView::operator delete(void* ptr) {
    nucleation::capi::WorldChunkView_destroy(reinterpret_cast<nucleation::capi::WorldChunkView*>(ptr));
}


#endif // NUCLEATION_WorldChunkView_HPP
