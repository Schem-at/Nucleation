#ifndef MeshConfig_HPP
#define MeshConfig_HPP

#include "MeshConfig.d.hpp"

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

    diplomat::capi::MeshConfig* MeshConfig_create(void);

    void MeshConfig_set_cull_hidden_faces(diplomat::capi::MeshConfig* self, bool val);

    bool MeshConfig_cull_hidden_faces(const diplomat::capi::MeshConfig* self);

    void MeshConfig_set_ambient_occlusion(diplomat::capi::MeshConfig* self, bool val);

    bool MeshConfig_ambient_occlusion(const diplomat::capi::MeshConfig* self);

    void MeshConfig_set_ao_intensity(diplomat::capi::MeshConfig* self, float val);

    float MeshConfig_ao_intensity(const diplomat::capi::MeshConfig* self);

    typedef struct MeshConfig_set_biome_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} MeshConfig_set_biome_result;
    MeshConfig_set_biome_result MeshConfig_set_biome(diplomat::capi::MeshConfig* self, diplomat::capi::DiplomatStringView biome);

    void MeshConfig_clear_biome(diplomat::capi::MeshConfig* self);

    typedef struct MeshConfig_biome_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} MeshConfig_biome_result;
    MeshConfig_biome_result MeshConfig_biome(const diplomat::capi::MeshConfig* self, diplomat::capi::DiplomatWrite* write);

    void MeshConfig_set_atlas_max_size(diplomat::capi::MeshConfig* self, uint32_t size);

    uint32_t MeshConfig_atlas_max_size(const diplomat::capi::MeshConfig* self);

    void MeshConfig_set_cull_occluded_blocks(diplomat::capi::MeshConfig* self, bool val);

    bool MeshConfig_cull_occluded_blocks(const diplomat::capi::MeshConfig* self);

    void MeshConfig_set_greedy_meshing(diplomat::capi::MeshConfig* self, bool val);

    bool MeshConfig_greedy_meshing(const diplomat::capi::MeshConfig* self);

    void MeshConfig_destroy(MeshConfig* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::unique_ptr<MeshConfig> MeshConfig::create() {
    auto result = diplomat::capi::MeshConfig_create();
    return std::unique_ptr<MeshConfig>(MeshConfig::FromFFI(result));
}

inline void MeshConfig::set_cull_hidden_faces(bool val) {
    diplomat::capi::MeshConfig_set_cull_hidden_faces(this->AsFFI(),
        val);
}

inline bool MeshConfig::cull_hidden_faces() const {
    auto result = diplomat::capi::MeshConfig_cull_hidden_faces(this->AsFFI());
    return result;
}

inline void MeshConfig::set_ambient_occlusion(bool val) {
    diplomat::capi::MeshConfig_set_ambient_occlusion(this->AsFFI(),
        val);
}

inline bool MeshConfig::ambient_occlusion() const {
    auto result = diplomat::capi::MeshConfig_ambient_occlusion(this->AsFFI());
    return result;
}

inline void MeshConfig::set_ao_intensity(float val) {
    diplomat::capi::MeshConfig_set_ao_intensity(this->AsFFI(),
        val);
}

inline float MeshConfig::ao_intensity() const {
    auto result = diplomat::capi::MeshConfig_ao_intensity(this->AsFFI());
    return result;
}

inline diplomat::result<std::monostate, NucleationError> MeshConfig::set_biome(std::string_view biome) {
    auto result = diplomat::capi::MeshConfig_set_biome(this->AsFFI(),
        {biome.data(), biome.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline void MeshConfig::clear_biome() {
    diplomat::capi::MeshConfig_clear_biome(this->AsFFI());
}

inline diplomat::result<std::string, NucleationError> MeshConfig::biome() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::MeshConfig_biome(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> MeshConfig::biome_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::MeshConfig_biome(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline void MeshConfig::set_atlas_max_size(uint32_t size) {
    diplomat::capi::MeshConfig_set_atlas_max_size(this->AsFFI(),
        size);
}

inline uint32_t MeshConfig::atlas_max_size() const {
    auto result = diplomat::capi::MeshConfig_atlas_max_size(this->AsFFI());
    return result;
}

inline void MeshConfig::set_cull_occluded_blocks(bool val) {
    diplomat::capi::MeshConfig_set_cull_occluded_blocks(this->AsFFI(),
        val);
}

inline bool MeshConfig::cull_occluded_blocks() const {
    auto result = diplomat::capi::MeshConfig_cull_occluded_blocks(this->AsFFI());
    return result;
}

inline void MeshConfig::set_greedy_meshing(bool val) {
    diplomat::capi::MeshConfig_set_greedy_meshing(this->AsFFI(),
        val);
}

inline bool MeshConfig::greedy_meshing() const {
    auto result = diplomat::capi::MeshConfig_greedy_meshing(this->AsFFI());
    return result;
}

inline const diplomat::capi::MeshConfig* MeshConfig::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::MeshConfig*>(this);
}

inline diplomat::capi::MeshConfig* MeshConfig::AsFFI() {
    return reinterpret_cast<diplomat::capi::MeshConfig*>(this);
}

inline const MeshConfig* MeshConfig::FromFFI(const diplomat::capi::MeshConfig* ptr) {
    return reinterpret_cast<const MeshConfig*>(ptr);
}

inline MeshConfig* MeshConfig::FromFFI(diplomat::capi::MeshConfig* ptr) {
    return reinterpret_cast<MeshConfig*>(ptr);
}

inline void MeshConfig::operator delete(void* ptr) {
    diplomat::capi::MeshConfig_destroy(reinterpret_cast<diplomat::capi::MeshConfig*>(ptr));
}


#endif // MeshConfig_HPP
