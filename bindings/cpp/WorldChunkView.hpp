#ifndef WorldChunkView_HPP
#define WorldChunkView_HPP

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


namespace diplomat {
namespace capi {
    extern "C" {

    diplomat::capi::WorldChunkView* WorldChunkView_create(int32_t cx, int32_t cz);

    int32_t WorldChunkView_cx(const diplomat::capi::WorldChunkView* self);

    int32_t WorldChunkView_cz(const diplomat::capi::WorldChunkView* self);

    diplomat::capi::Schematic* WorldChunkView_to_schematic(const diplomat::capi::WorldChunkView* self);

    typedef struct WorldChunkView_set_block_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} WorldChunkView_set_block_result;
    WorldChunkView_set_block_result WorldChunkView_set_block(diplomat::capi::WorldChunkView* self, int32_t x, int32_t y, int32_t z, diplomat::capi::DiplomatStringView block_name);

    typedef struct WorldChunkView_set_biome_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} WorldChunkView_set_biome_result;
    WorldChunkView_set_biome_result WorldChunkView_set_biome(diplomat::capi::WorldChunkView* self, diplomat::capi::DiplomatStringView biome_name);

    typedef struct WorldChunkView_biome_palette_json_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} WorldChunkView_biome_palette_json_result;
    WorldChunkView_biome_palette_json_result WorldChunkView_biome_palette_json(const diplomat::capi::WorldChunkView* self, diplomat::capi::DiplomatWrite* write);

    void WorldChunkView_destroy(WorldChunkView* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::unique_ptr<WorldChunkView> WorldChunkView::create(int32_t cx, int32_t cz) {
    auto result = diplomat::capi::WorldChunkView_create(cx,
        cz);
    return std::unique_ptr<WorldChunkView>(WorldChunkView::FromFFI(result));
}

inline int32_t WorldChunkView::cx() const {
    auto result = diplomat::capi::WorldChunkView_cx(this->AsFFI());
    return result;
}

inline int32_t WorldChunkView::cz() const {
    auto result = diplomat::capi::WorldChunkView_cz(this->AsFFI());
    return result;
}

inline std::unique_ptr<Schematic> WorldChunkView::to_schematic() const {
    auto result = diplomat::capi::WorldChunkView_to_schematic(this->AsFFI());
    return std::unique_ptr<Schematic>(Schematic::FromFFI(result));
}

inline diplomat::result<std::monostate, NucleationError> WorldChunkView::set_block(int32_t x, int32_t y, int32_t z, std::string_view block_name) {
    auto result = diplomat::capi::WorldChunkView_set_block(this->AsFFI(),
        x,
        y,
        z,
        {block_name.data(), block_name.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> WorldChunkView::set_biome(std::string_view biome_name) {
    auto result = diplomat::capi::WorldChunkView_set_biome(this->AsFFI(),
        {biome_name.data(), biome_name.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> WorldChunkView::biome_palette_json() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::WorldChunkView_biome_palette_json(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> WorldChunkView::biome_palette_json_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::WorldChunkView_biome_palette_json(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline const diplomat::capi::WorldChunkView* WorldChunkView::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::WorldChunkView*>(this);
}

inline diplomat::capi::WorldChunkView* WorldChunkView::AsFFI() {
    return reinterpret_cast<diplomat::capi::WorldChunkView*>(this);
}

inline const WorldChunkView* WorldChunkView::FromFFI(const diplomat::capi::WorldChunkView* ptr) {
    return reinterpret_cast<const WorldChunkView*>(ptr);
}

inline WorldChunkView* WorldChunkView::FromFFI(diplomat::capi::WorldChunkView* ptr) {
    return reinterpret_cast<WorldChunkView*>(ptr);
}

inline void WorldChunkView::operator delete(void* ptr) {
    diplomat::capi::WorldChunkView_destroy(reinterpret_cast<diplomat::capi::WorldChunkView*>(ptr));
}


#endif // WorldChunkView_HPP
