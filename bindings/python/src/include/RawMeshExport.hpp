#ifndef NUCLEATION_RawMeshExport_HPP
#define NUCLEATION_RawMeshExport_HPP

#include "RawMeshExport.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "MeshConfig.hpp"
#include "NucleationError.hpp"
#include "ResourcePack.hpp"
#include "Schematic.hpp"
#include "diplomat_runtime.hpp"


namespace nucleation {
namespace capi {
    extern "C" {

    typedef struct RawMeshExport_create_result {union {nucleation::capi::RawMeshExport* ok; nucleation::capi::NucleationError err;}; bool is_ok;} RawMeshExport_create_result;
    RawMeshExport_create_result RawMeshExport_create(const nucleation::capi::Schematic* schematic, const nucleation::capi::ResourcePack* pack, const nucleation::capi::MeshConfig* config);

    uint32_t RawMeshExport_vertex_count(const nucleation::capi::RawMeshExport* self);

    uint32_t RawMeshExport_triangle_count(const nucleation::capi::RawMeshExport* self);

    void RawMeshExport_positions_b64(const nucleation::capi::RawMeshExport* self, nucleation::diplomat::capi::DiplomatWrite* write);

    void RawMeshExport_normals_b64(const nucleation::capi::RawMeshExport* self, nucleation::diplomat::capi::DiplomatWrite* write);

    void RawMeshExport_uvs_b64(const nucleation::capi::RawMeshExport* self, nucleation::diplomat::capi::DiplomatWrite* write);

    void RawMeshExport_colors_b64(const nucleation::capi::RawMeshExport* self, nucleation::diplomat::capi::DiplomatWrite* write);

    void RawMeshExport_indices_b64(const nucleation::capi::RawMeshExport* self, nucleation::diplomat::capi::DiplomatWrite* write);

    void RawMeshExport_texture_rgba_b64(const nucleation::capi::RawMeshExport* self, nucleation::diplomat::capi::DiplomatWrite* write);

    uint32_t RawMeshExport_texture_width(const nucleation::capi::RawMeshExport* self);

    uint32_t RawMeshExport_texture_height(const nucleation::capi::RawMeshExport* self);

    void RawMeshExport_destroy(RawMeshExport* self);

    } // extern "C"
} // namespace capi
} // namespace

inline nucleation::diplomat::result<std::unique_ptr<nucleation::RawMeshExport>, nucleation::NucleationError> nucleation::RawMeshExport::create(const nucleation::Schematic& schematic, const nucleation::ResourcePack& pack, const nucleation::MeshConfig& config) {
    auto result = nucleation::capi::RawMeshExport_create(schematic.AsFFI(),
        pack.AsFFI(),
        config.AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::RawMeshExport>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::RawMeshExport>>(std::unique_ptr<nucleation::RawMeshExport>(nucleation::RawMeshExport::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::RawMeshExport>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline uint32_t nucleation::RawMeshExport::vertex_count() const {
    auto result = nucleation::capi::RawMeshExport_vertex_count(this->AsFFI());
    return result;
}

inline uint32_t nucleation::RawMeshExport::triangle_count() const {
    auto result = nucleation::capi::RawMeshExport_triangle_count(this->AsFFI());
    return result;
}

inline std::string nucleation::RawMeshExport::positions_b64() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::RawMeshExport_positions_b64(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void nucleation::RawMeshExport::positions_b64_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::RawMeshExport_positions_b64(this->AsFFI(),
        &write);
}

inline std::string nucleation::RawMeshExport::normals_b64() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::RawMeshExport_normals_b64(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void nucleation::RawMeshExport::normals_b64_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::RawMeshExport_normals_b64(this->AsFFI(),
        &write);
}

inline std::string nucleation::RawMeshExport::uvs_b64() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::RawMeshExport_uvs_b64(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void nucleation::RawMeshExport::uvs_b64_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::RawMeshExport_uvs_b64(this->AsFFI(),
        &write);
}

inline std::string nucleation::RawMeshExport::colors_b64() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::RawMeshExport_colors_b64(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void nucleation::RawMeshExport::colors_b64_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::RawMeshExport_colors_b64(this->AsFFI(),
        &write);
}

inline std::string nucleation::RawMeshExport::indices_b64() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::RawMeshExport_indices_b64(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void nucleation::RawMeshExport::indices_b64_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::RawMeshExport_indices_b64(this->AsFFI(),
        &write);
}

inline std::string nucleation::RawMeshExport::texture_rgba_b64() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::RawMeshExport_texture_rgba_b64(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void nucleation::RawMeshExport::texture_rgba_b64_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::RawMeshExport_texture_rgba_b64(this->AsFFI(),
        &write);
}

inline uint32_t nucleation::RawMeshExport::texture_width() const {
    auto result = nucleation::capi::RawMeshExport_texture_width(this->AsFFI());
    return result;
}

inline uint32_t nucleation::RawMeshExport::texture_height() const {
    auto result = nucleation::capi::RawMeshExport_texture_height(this->AsFFI());
    return result;
}

inline const nucleation::capi::RawMeshExport* nucleation::RawMeshExport::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::RawMeshExport*>(this);
}

inline nucleation::capi::RawMeshExport* nucleation::RawMeshExport::AsFFI() {
    return reinterpret_cast<nucleation::capi::RawMeshExport*>(this);
}

inline const nucleation::RawMeshExport* nucleation::RawMeshExport::FromFFI(const nucleation::capi::RawMeshExport* ptr) {
    return reinterpret_cast<const nucleation::RawMeshExport*>(ptr);
}

inline nucleation::RawMeshExport* nucleation::RawMeshExport::FromFFI(nucleation::capi::RawMeshExport* ptr) {
    return reinterpret_cast<nucleation::RawMeshExport*>(ptr);
}

inline void nucleation::RawMeshExport::operator delete(void* ptr) {
    nucleation::capi::RawMeshExport_destroy(reinterpret_cast<nucleation::capi::RawMeshExport*>(ptr));
}


#endif // NUCLEATION_RawMeshExport_HPP
