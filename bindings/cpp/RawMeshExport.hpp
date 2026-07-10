#ifndef RawMeshExport_HPP
#define RawMeshExport_HPP

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


namespace diplomat {
namespace capi {
    extern "C" {

    typedef struct RawMeshExport_create_result {union {diplomat::capi::RawMeshExport* ok; diplomat::capi::NucleationError err;}; bool is_ok;} RawMeshExport_create_result;
    RawMeshExport_create_result RawMeshExport_create(const diplomat::capi::Schematic* schematic, const diplomat::capi::ResourcePack* pack, const diplomat::capi::MeshConfig* config);

    uint32_t RawMeshExport_vertex_count(const diplomat::capi::RawMeshExport* self);

    uint32_t RawMeshExport_triangle_count(const diplomat::capi::RawMeshExport* self);

    void RawMeshExport_positions_b64(const diplomat::capi::RawMeshExport* self, diplomat::capi::DiplomatWrite* write);

    void RawMeshExport_normals_b64(const diplomat::capi::RawMeshExport* self, diplomat::capi::DiplomatWrite* write);

    void RawMeshExport_uvs_b64(const diplomat::capi::RawMeshExport* self, diplomat::capi::DiplomatWrite* write);

    void RawMeshExport_colors_b64(const diplomat::capi::RawMeshExport* self, diplomat::capi::DiplomatWrite* write);

    void RawMeshExport_indices_b64(const diplomat::capi::RawMeshExport* self, diplomat::capi::DiplomatWrite* write);

    void RawMeshExport_texture_rgba_b64(const diplomat::capi::RawMeshExport* self, diplomat::capi::DiplomatWrite* write);

    uint32_t RawMeshExport_texture_width(const diplomat::capi::RawMeshExport* self);

    uint32_t RawMeshExport_texture_height(const diplomat::capi::RawMeshExport* self);

    void RawMeshExport_destroy(RawMeshExport* self);

    } // extern "C"
} // namespace capi
} // namespace

inline diplomat::result<std::unique_ptr<RawMeshExport>, NucleationError> RawMeshExport::create(const Schematic& schematic, const ResourcePack& pack, const MeshConfig& config) {
    auto result = diplomat::capi::RawMeshExport_create(schematic.AsFFI(),
        pack.AsFFI(),
        config.AsFFI());
    return result.is_ok ? diplomat::result<std::unique_ptr<RawMeshExport>, NucleationError>(diplomat::Ok<std::unique_ptr<RawMeshExport>>(std::unique_ptr<RawMeshExport>(RawMeshExport::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<RawMeshExport>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline uint32_t RawMeshExport::vertex_count() const {
    auto result = diplomat::capi::RawMeshExport_vertex_count(this->AsFFI());
    return result;
}

inline uint32_t RawMeshExport::triangle_count() const {
    auto result = diplomat::capi::RawMeshExport_triangle_count(this->AsFFI());
    return result;
}

inline std::string RawMeshExport::positions_b64() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::RawMeshExport_positions_b64(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void RawMeshExport::positions_b64_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::RawMeshExport_positions_b64(this->AsFFI(),
        &write);
}

inline std::string RawMeshExport::normals_b64() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::RawMeshExport_normals_b64(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void RawMeshExport::normals_b64_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::RawMeshExport_normals_b64(this->AsFFI(),
        &write);
}

inline std::string RawMeshExport::uvs_b64() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::RawMeshExport_uvs_b64(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void RawMeshExport::uvs_b64_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::RawMeshExport_uvs_b64(this->AsFFI(),
        &write);
}

inline std::string RawMeshExport::colors_b64() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::RawMeshExport_colors_b64(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void RawMeshExport::colors_b64_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::RawMeshExport_colors_b64(this->AsFFI(),
        &write);
}

inline std::string RawMeshExport::indices_b64() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::RawMeshExport_indices_b64(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void RawMeshExport::indices_b64_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::RawMeshExport_indices_b64(this->AsFFI(),
        &write);
}

inline std::string RawMeshExport::texture_rgba_b64() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::RawMeshExport_texture_rgba_b64(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void RawMeshExport::texture_rgba_b64_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::RawMeshExport_texture_rgba_b64(this->AsFFI(),
        &write);
}

inline uint32_t RawMeshExport::texture_width() const {
    auto result = diplomat::capi::RawMeshExport_texture_width(this->AsFFI());
    return result;
}

inline uint32_t RawMeshExport::texture_height() const {
    auto result = diplomat::capi::RawMeshExport_texture_height(this->AsFFI());
    return result;
}

inline const diplomat::capi::RawMeshExport* RawMeshExport::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::RawMeshExport*>(this);
}

inline diplomat::capi::RawMeshExport* RawMeshExport::AsFFI() {
    return reinterpret_cast<diplomat::capi::RawMeshExport*>(this);
}

inline const RawMeshExport* RawMeshExport::FromFFI(const diplomat::capi::RawMeshExport* ptr) {
    return reinterpret_cast<const RawMeshExport*>(ptr);
}

inline RawMeshExport* RawMeshExport::FromFFI(diplomat::capi::RawMeshExport* ptr) {
    return reinterpret_cast<RawMeshExport*>(ptr);
}

inline void RawMeshExport::operator delete(void* ptr) {
    diplomat::capi::RawMeshExport_destroy(reinterpret_cast<diplomat::capi::RawMeshExport*>(ptr));
}


#endif // RawMeshExport_HPP
