#ifndef MultiMeshResult_HPP
#define MultiMeshResult_HPP

#include "MultiMeshResult.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "MeshConfig.hpp"
#include "MeshResult.hpp"
#include "NucleationError.hpp"
#include "ResourcePack.hpp"
#include "Schematic.hpp"
#include "diplomat_runtime.hpp"


namespace diplomat {
namespace capi {
    extern "C" {

    typedef struct MultiMeshResult_create_result {union {diplomat::capi::MultiMeshResult* ok; diplomat::capi::NucleationError err;}; bool is_ok;} MultiMeshResult_create_result;
    MultiMeshResult_create_result MultiMeshResult_create(const diplomat::capi::Schematic* schematic, const diplomat::capi::ResourcePack* pack, const diplomat::capi::MeshConfig* config);

    void MultiMeshResult_region_names_json(const diplomat::capi::MultiMeshResult* self, diplomat::capi::DiplomatWrite* write);

    typedef struct MultiMeshResult_get_mesh_result {union {diplomat::capi::MeshResult* ok; diplomat::capi::NucleationError err;}; bool is_ok;} MultiMeshResult_get_mesh_result;
    MultiMeshResult_get_mesh_result MultiMeshResult_get_mesh(const diplomat::capi::MultiMeshResult* self, diplomat::capi::DiplomatStringView region_name);

    uint32_t MultiMeshResult_total_vertex_count(const diplomat::capi::MultiMeshResult* self);

    uint32_t MultiMeshResult_total_triangle_count(const diplomat::capi::MultiMeshResult* self);

    uint32_t MultiMeshResult_mesh_count(const diplomat::capi::MultiMeshResult* self);

    void MultiMeshResult_destroy(MultiMeshResult* self);

    } // extern "C"
} // namespace capi
} // namespace

inline diplomat::result<std::unique_ptr<MultiMeshResult>, NucleationError> MultiMeshResult::create(const Schematic& schematic, const ResourcePack& pack, const MeshConfig& config) {
    auto result = diplomat::capi::MultiMeshResult_create(schematic.AsFFI(),
        pack.AsFFI(),
        config.AsFFI());
    return result.is_ok ? diplomat::result<std::unique_ptr<MultiMeshResult>, NucleationError>(diplomat::Ok<std::unique_ptr<MultiMeshResult>>(std::unique_ptr<MultiMeshResult>(MultiMeshResult::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<MultiMeshResult>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline std::string MultiMeshResult::region_names_json() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::MultiMeshResult_region_names_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void MultiMeshResult::region_names_json_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::MultiMeshResult_region_names_json(this->AsFFI(),
        &write);
}

inline diplomat::result<std::unique_ptr<MeshResult>, NucleationError> MultiMeshResult::get_mesh(std::string_view region_name) const {
    auto result = diplomat::capi::MultiMeshResult_get_mesh(this->AsFFI(),
        {region_name.data(), region_name.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<MeshResult>, NucleationError>(diplomat::Ok<std::unique_ptr<MeshResult>>(std::unique_ptr<MeshResult>(MeshResult::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<MeshResult>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline uint32_t MultiMeshResult::total_vertex_count() const {
    auto result = diplomat::capi::MultiMeshResult_total_vertex_count(this->AsFFI());
    return result;
}

inline uint32_t MultiMeshResult::total_triangle_count() const {
    auto result = diplomat::capi::MultiMeshResult_total_triangle_count(this->AsFFI());
    return result;
}

inline uint32_t MultiMeshResult::mesh_count() const {
    auto result = diplomat::capi::MultiMeshResult_mesh_count(this->AsFFI());
    return result;
}

inline const diplomat::capi::MultiMeshResult* MultiMeshResult::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::MultiMeshResult*>(this);
}

inline diplomat::capi::MultiMeshResult* MultiMeshResult::AsFFI() {
    return reinterpret_cast<diplomat::capi::MultiMeshResult*>(this);
}

inline const MultiMeshResult* MultiMeshResult::FromFFI(const diplomat::capi::MultiMeshResult* ptr) {
    return reinterpret_cast<const MultiMeshResult*>(ptr);
}

inline MultiMeshResult* MultiMeshResult::FromFFI(diplomat::capi::MultiMeshResult* ptr) {
    return reinterpret_cast<MultiMeshResult*>(ptr);
}

inline void MultiMeshResult::operator delete(void* ptr) {
    diplomat::capi::MultiMeshResult_destroy(reinterpret_cast<diplomat::capi::MultiMeshResult*>(ptr));
}


#endif // MultiMeshResult_HPP
