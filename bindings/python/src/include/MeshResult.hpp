#ifndef NUCLEATION_MeshResult_HPP
#define NUCLEATION_MeshResult_HPP

#include "MeshResult.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "MeshBounds.hpp"
#include "MeshConfig.hpp"
#include "NucleationError.hpp"
#include "ResourcePack.hpp"
#include "Schematic.hpp"
#include "diplomat_runtime.hpp"


namespace nucleation {
namespace capi {
    extern "C" {

    typedef struct MeshResult_create_result {union {nucleation::capi::MeshResult* ok; nucleation::capi::NucleationError err;}; bool is_ok;} MeshResult_create_result;
    MeshResult_create_result MeshResult_create(const nucleation::capi::Schematic* schematic, const nucleation::capi::ResourcePack* pack, const nucleation::capi::MeshConfig* config);

    typedef struct MeshResult_create_usdz_result {union {nucleation::capi::MeshResult* ok; nucleation::capi::NucleationError err;}; bool is_ok;} MeshResult_create_usdz_result;
    MeshResult_create_usdz_result MeshResult_create_usdz(const nucleation::capi::Schematic* schematic, const nucleation::capi::ResourcePack* pack, const nucleation::capi::MeshConfig* config);

    typedef struct MeshResult_glb_data_b64_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} MeshResult_glb_data_b64_result;
    MeshResult_glb_data_b64_result MeshResult_glb_data_b64(const nucleation::capi::MeshResult* self, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct MeshResult_usdz_data_b64_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} MeshResult_usdz_data_b64_result;
    MeshResult_usdz_data_b64_result MeshResult_usdz_data_b64(const nucleation::capi::MeshResult* self, nucleation::diplomat::capi::DiplomatWrite* write);

    void MeshResult_nucm_data_b64(const nucleation::capi::MeshResult* self, nucleation::diplomat::capi::DiplomatWrite* write);

    uint32_t MeshResult_vertex_count(const nucleation::capi::MeshResult* self);

    uint32_t MeshResult_triangle_count(const nucleation::capi::MeshResult* self);

    bool MeshResult_has_transparency(const nucleation::capi::MeshResult* self);

    nucleation::capi::MeshBounds MeshResult_bounds(const nucleation::capi::MeshResult* self);

    void MeshResult_destroy(MeshResult* self);

    } // extern "C"
} // namespace capi
} // namespace

inline nucleation::diplomat::result<std::unique_ptr<nucleation::MeshResult>, nucleation::NucleationError> nucleation::MeshResult::create(const nucleation::Schematic& schematic, const nucleation::ResourcePack& pack, const nucleation::MeshConfig& config) {
    auto result = nucleation::capi::MeshResult_create(schematic.AsFFI(),
        pack.AsFFI(),
        config.AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::MeshResult>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::MeshResult>>(std::unique_ptr<nucleation::MeshResult>(nucleation::MeshResult::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::MeshResult>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::MeshResult>, nucleation::NucleationError> nucleation::MeshResult::create_usdz(const nucleation::Schematic& schematic, const nucleation::ResourcePack& pack, const nucleation::MeshConfig& config) {
    auto result = nucleation::capi::MeshResult_create_usdz(schematic.AsFFI(),
        pack.AsFFI(),
        config.AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::MeshResult>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::MeshResult>>(std::unique_ptr<nucleation::MeshResult>(nucleation::MeshResult::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::MeshResult>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::MeshResult::glb_data_b64() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::MeshResult_glb_data_b64(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::MeshResult::glb_data_b64_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::MeshResult_glb_data_b64(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::MeshResult::usdz_data_b64() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::MeshResult_usdz_data_b64(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::MeshResult::usdz_data_b64_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::MeshResult_usdz_data_b64(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline std::string nucleation::MeshResult::nucm_data_b64() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::MeshResult_nucm_data_b64(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void nucleation::MeshResult::nucm_data_b64_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::MeshResult_nucm_data_b64(this->AsFFI(),
        &write);
}

inline uint32_t nucleation::MeshResult::vertex_count() const {
    auto result = nucleation::capi::MeshResult_vertex_count(this->AsFFI());
    return result;
}

inline uint32_t nucleation::MeshResult::triangle_count() const {
    auto result = nucleation::capi::MeshResult_triangle_count(this->AsFFI());
    return result;
}

inline bool nucleation::MeshResult::has_transparency() const {
    auto result = nucleation::capi::MeshResult_has_transparency(this->AsFFI());
    return result;
}

inline nucleation::MeshBounds nucleation::MeshResult::bounds() const {
    auto result = nucleation::capi::MeshResult_bounds(this->AsFFI());
    return nucleation::MeshBounds::FromFFI(result);
}

inline const nucleation::capi::MeshResult* nucleation::MeshResult::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::MeshResult*>(this);
}

inline nucleation::capi::MeshResult* nucleation::MeshResult::AsFFI() {
    return reinterpret_cast<nucleation::capi::MeshResult*>(this);
}

inline const nucleation::MeshResult* nucleation::MeshResult::FromFFI(const nucleation::capi::MeshResult* ptr) {
    return reinterpret_cast<const nucleation::MeshResult*>(ptr);
}

inline nucleation::MeshResult* nucleation::MeshResult::FromFFI(nucleation::capi::MeshResult* ptr) {
    return reinterpret_cast<nucleation::MeshResult*>(ptr);
}

inline void nucleation::MeshResult::operator delete(void* ptr) {
    nucleation::capi::MeshResult_destroy(reinterpret_cast<nucleation::capi::MeshResult*>(ptr));
}


#endif // NUCLEATION_MeshResult_HPP
