#ifndef NUCLEATION_MeshConfig_HPP
#define NUCLEATION_MeshConfig_HPP

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


namespace nucleation {
namespace capi {
    extern "C" {

    nucleation::capi::MeshConfig* MeshConfig_create(void);

    void MeshConfig_set_cull_hidden_faces(nucleation::capi::MeshConfig* self, bool val);

    bool MeshConfig_cull_hidden_faces(const nucleation::capi::MeshConfig* self);

    void MeshConfig_set_ambient_occlusion(nucleation::capi::MeshConfig* self, bool val);

    bool MeshConfig_ambient_occlusion(const nucleation::capi::MeshConfig* self);

    void MeshConfig_set_ao_intensity(nucleation::capi::MeshConfig* self, float val);

    float MeshConfig_ao_intensity(const nucleation::capi::MeshConfig* self);

    typedef struct MeshConfig_set_biome_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} MeshConfig_set_biome_result;
    MeshConfig_set_biome_result MeshConfig_set_biome(nucleation::capi::MeshConfig* self, nucleation::diplomat::capi::DiplomatStringView biome);

    void MeshConfig_clear_biome(nucleation::capi::MeshConfig* self);

    typedef struct MeshConfig_biome_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} MeshConfig_biome_result;
    MeshConfig_biome_result MeshConfig_biome(const nucleation::capi::MeshConfig* self, nucleation::diplomat::capi::DiplomatWrite* write);

    void MeshConfig_set_atlas_max_size(nucleation::capi::MeshConfig* self, uint32_t size);

    uint32_t MeshConfig_atlas_max_size(const nucleation::capi::MeshConfig* self);

    void MeshConfig_set_cull_occluded_blocks(nucleation::capi::MeshConfig* self, bool val);

    bool MeshConfig_cull_occluded_blocks(const nucleation::capi::MeshConfig* self);

    void MeshConfig_set_greedy_meshing(nucleation::capi::MeshConfig* self, bool val);

    bool MeshConfig_greedy_meshing(const nucleation::capi::MeshConfig* self);

    void MeshConfig_destroy(MeshConfig* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::unique_ptr<nucleation::MeshConfig> nucleation::MeshConfig::create() {
    auto result = nucleation::capi::MeshConfig_create();
    return std::unique_ptr<nucleation::MeshConfig>(nucleation::MeshConfig::FromFFI(result));
}

inline void nucleation::MeshConfig::set_cull_hidden_faces(bool val) {
    nucleation::capi::MeshConfig_set_cull_hidden_faces(this->AsFFI(),
        val);
}

inline bool nucleation::MeshConfig::cull_hidden_faces() const {
    auto result = nucleation::capi::MeshConfig_cull_hidden_faces(this->AsFFI());
    return result;
}

inline void nucleation::MeshConfig::set_ambient_occlusion(bool val) {
    nucleation::capi::MeshConfig_set_ambient_occlusion(this->AsFFI(),
        val);
}

inline bool nucleation::MeshConfig::ambient_occlusion() const {
    auto result = nucleation::capi::MeshConfig_ambient_occlusion(this->AsFFI());
    return result;
}

inline void nucleation::MeshConfig::set_ao_intensity(float val) {
    nucleation::capi::MeshConfig_set_ao_intensity(this->AsFFI(),
        val);
}

inline float nucleation::MeshConfig::ao_intensity() const {
    auto result = nucleation::capi::MeshConfig_ao_intensity(this->AsFFI());
    return result;
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::MeshConfig::set_biome(std::string_view biome) {
    auto result = nucleation::capi::MeshConfig_set_biome(this->AsFFI(),
        {biome.data(), biome.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline void nucleation::MeshConfig::clear_biome() {
    nucleation::capi::MeshConfig_clear_biome(this->AsFFI());
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::MeshConfig::biome() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::MeshConfig_biome(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::MeshConfig::biome_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::MeshConfig_biome(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline void nucleation::MeshConfig::set_atlas_max_size(uint32_t size) {
    nucleation::capi::MeshConfig_set_atlas_max_size(this->AsFFI(),
        size);
}

inline uint32_t nucleation::MeshConfig::atlas_max_size() const {
    auto result = nucleation::capi::MeshConfig_atlas_max_size(this->AsFFI());
    return result;
}

inline void nucleation::MeshConfig::set_cull_occluded_blocks(bool val) {
    nucleation::capi::MeshConfig_set_cull_occluded_blocks(this->AsFFI(),
        val);
}

inline bool nucleation::MeshConfig::cull_occluded_blocks() const {
    auto result = nucleation::capi::MeshConfig_cull_occluded_blocks(this->AsFFI());
    return result;
}

inline void nucleation::MeshConfig::set_greedy_meshing(bool val) {
    nucleation::capi::MeshConfig_set_greedy_meshing(this->AsFFI(),
        val);
}

inline bool nucleation::MeshConfig::greedy_meshing() const {
    auto result = nucleation::capi::MeshConfig_greedy_meshing(this->AsFFI());
    return result;
}

inline const nucleation::capi::MeshConfig* nucleation::MeshConfig::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::MeshConfig*>(this);
}

inline nucleation::capi::MeshConfig* nucleation::MeshConfig::AsFFI() {
    return reinterpret_cast<nucleation::capi::MeshConfig*>(this);
}

inline const nucleation::MeshConfig* nucleation::MeshConfig::FromFFI(const nucleation::capi::MeshConfig* ptr) {
    return reinterpret_cast<const nucleation::MeshConfig*>(ptr);
}

inline nucleation::MeshConfig* nucleation::MeshConfig::FromFFI(nucleation::capi::MeshConfig* ptr) {
    return reinterpret_cast<nucleation::MeshConfig*>(ptr);
}

inline void nucleation::MeshConfig::operator delete(void* ptr) {
    nucleation::capi::MeshConfig_destroy(reinterpret_cast<nucleation::capi::MeshConfig*>(ptr));
}


#endif // NUCLEATION_MeshConfig_HPP
