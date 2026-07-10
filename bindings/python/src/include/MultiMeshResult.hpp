#ifndef NUCLEATION_MultiMeshResult_HPP
#define NUCLEATION_MultiMeshResult_HPP

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


namespace nucleation {
namespace capi {
    extern "C" {

    typedef struct MultiMeshResult_create_result {union {nucleation::capi::MultiMeshResult* ok; nucleation::capi::NucleationError err;}; bool is_ok;} MultiMeshResult_create_result;
    MultiMeshResult_create_result MultiMeshResult_create(const nucleation::capi::Schematic* schematic, const nucleation::capi::ResourcePack* pack, const nucleation::capi::MeshConfig* config);

    void MultiMeshResult_region_names_json(const nucleation::capi::MultiMeshResult* self, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct MultiMeshResult_get_mesh_result {union {nucleation::capi::MeshResult* ok; nucleation::capi::NucleationError err;}; bool is_ok;} MultiMeshResult_get_mesh_result;
    MultiMeshResult_get_mesh_result MultiMeshResult_get_mesh(const nucleation::capi::MultiMeshResult* self, nucleation::diplomat::capi::DiplomatStringView region_name);

    uint32_t MultiMeshResult_total_vertex_count(const nucleation::capi::MultiMeshResult* self);

    uint32_t MultiMeshResult_total_triangle_count(const nucleation::capi::MultiMeshResult* self);

    uint32_t MultiMeshResult_mesh_count(const nucleation::capi::MultiMeshResult* self);

    void MultiMeshResult_destroy(MultiMeshResult* self);

    } // extern "C"
} // namespace capi
} // namespace

inline nucleation::diplomat::result<std::unique_ptr<nucleation::MultiMeshResult>, nucleation::NucleationError> nucleation::MultiMeshResult::create(const nucleation::Schematic& schematic, const nucleation::ResourcePack& pack, const nucleation::MeshConfig& config) {
    auto result = nucleation::capi::MultiMeshResult_create(schematic.AsFFI(),
        pack.AsFFI(),
        config.AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::MultiMeshResult>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::MultiMeshResult>>(std::unique_ptr<nucleation::MultiMeshResult>(nucleation::MultiMeshResult::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::MultiMeshResult>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline std::string nucleation::MultiMeshResult::region_names_json() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::MultiMeshResult_region_names_json(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void nucleation::MultiMeshResult::region_names_json_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::MultiMeshResult_region_names_json(this->AsFFI(),
        &write);
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::MeshResult>, nucleation::NucleationError> nucleation::MultiMeshResult::get_mesh(std::string_view region_name) const {
    auto result = nucleation::capi::MultiMeshResult_get_mesh(this->AsFFI(),
        {region_name.data(), region_name.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::MeshResult>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::MeshResult>>(std::unique_ptr<nucleation::MeshResult>(nucleation::MeshResult::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::MeshResult>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline uint32_t nucleation::MultiMeshResult::total_vertex_count() const {
    auto result = nucleation::capi::MultiMeshResult_total_vertex_count(this->AsFFI());
    return result;
}

inline uint32_t nucleation::MultiMeshResult::total_triangle_count() const {
    auto result = nucleation::capi::MultiMeshResult_total_triangle_count(this->AsFFI());
    return result;
}

inline uint32_t nucleation::MultiMeshResult::mesh_count() const {
    auto result = nucleation::capi::MultiMeshResult_mesh_count(this->AsFFI());
    return result;
}

inline const nucleation::capi::MultiMeshResult* nucleation::MultiMeshResult::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::MultiMeshResult*>(this);
}

inline nucleation::capi::MultiMeshResult* nucleation::MultiMeshResult::AsFFI() {
    return reinterpret_cast<nucleation::capi::MultiMeshResult*>(this);
}

inline const nucleation::MultiMeshResult* nucleation::MultiMeshResult::FromFFI(const nucleation::capi::MultiMeshResult* ptr) {
    return reinterpret_cast<const nucleation::MultiMeshResult*>(ptr);
}

inline nucleation::MultiMeshResult* nucleation::MultiMeshResult::FromFFI(nucleation::capi::MultiMeshResult* ptr) {
    return reinterpret_cast<nucleation::MultiMeshResult*>(ptr);
}

inline void nucleation::MultiMeshResult::operator delete(void* ptr) {
    nucleation::capi::MultiMeshResult_destroy(reinterpret_cast<nucleation::capi::MultiMeshResult*>(ptr));
}


#endif // NUCLEATION_MultiMeshResult_HPP
