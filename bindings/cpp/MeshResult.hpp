#ifndef MeshResult_HPP
#define MeshResult_HPP

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


namespace diplomat {
namespace capi {
    extern "C" {

    typedef struct MeshResult_create_result {union {diplomat::capi::MeshResult* ok; diplomat::capi::NucleationError err;}; bool is_ok;} MeshResult_create_result;
    MeshResult_create_result MeshResult_create(const diplomat::capi::Schematic* schematic, const diplomat::capi::ResourcePack* pack, const diplomat::capi::MeshConfig* config);

    typedef struct MeshResult_create_usdz_result {union {diplomat::capi::MeshResult* ok; diplomat::capi::NucleationError err;}; bool is_ok;} MeshResult_create_usdz_result;
    MeshResult_create_usdz_result MeshResult_create_usdz(const diplomat::capi::Schematic* schematic, const diplomat::capi::ResourcePack* pack, const diplomat::capi::MeshConfig* config);

    typedef struct MeshResult_glb_data_b64_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} MeshResult_glb_data_b64_result;
    MeshResult_glb_data_b64_result MeshResult_glb_data_b64(const diplomat::capi::MeshResult* self, diplomat::capi::DiplomatWrite* write);

    typedef struct MeshResult_usdz_data_b64_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} MeshResult_usdz_data_b64_result;
    MeshResult_usdz_data_b64_result MeshResult_usdz_data_b64(const diplomat::capi::MeshResult* self, diplomat::capi::DiplomatWrite* write);

    void MeshResult_nucm_data_b64(const diplomat::capi::MeshResult* self, diplomat::capi::DiplomatWrite* write);

    uint32_t MeshResult_vertex_count(const diplomat::capi::MeshResult* self);

    uint32_t MeshResult_triangle_count(const diplomat::capi::MeshResult* self);

    bool MeshResult_has_transparency(const diplomat::capi::MeshResult* self);

    diplomat::capi::MeshBounds MeshResult_bounds(const diplomat::capi::MeshResult* self);

    void MeshResult_destroy(MeshResult* self);

    } // extern "C"
} // namespace capi
} // namespace

inline diplomat::result<std::unique_ptr<MeshResult>, NucleationError> MeshResult::create(const Schematic& schematic, const ResourcePack& pack, const MeshConfig& config) {
    auto result = diplomat::capi::MeshResult_create(schematic.AsFFI(),
        pack.AsFFI(),
        config.AsFFI());
    return result.is_ok ? diplomat::result<std::unique_ptr<MeshResult>, NucleationError>(diplomat::Ok<std::unique_ptr<MeshResult>>(std::unique_ptr<MeshResult>(MeshResult::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<MeshResult>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<MeshResult>, NucleationError> MeshResult::create_usdz(const Schematic& schematic, const ResourcePack& pack, const MeshConfig& config) {
    auto result = diplomat::capi::MeshResult_create_usdz(schematic.AsFFI(),
        pack.AsFFI(),
        config.AsFFI());
    return result.is_ok ? diplomat::result<std::unique_ptr<MeshResult>, NucleationError>(diplomat::Ok<std::unique_ptr<MeshResult>>(std::unique_ptr<MeshResult>(MeshResult::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<MeshResult>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> MeshResult::glb_data_b64() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::MeshResult_glb_data_b64(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> MeshResult::glb_data_b64_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::MeshResult_glb_data_b64(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> MeshResult::usdz_data_b64() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::MeshResult_usdz_data_b64(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> MeshResult::usdz_data_b64_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::MeshResult_usdz_data_b64(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline std::string MeshResult::nucm_data_b64() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::MeshResult_nucm_data_b64(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void MeshResult::nucm_data_b64_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::MeshResult_nucm_data_b64(this->AsFFI(),
        &write);
}

inline uint32_t MeshResult::vertex_count() const {
    auto result = diplomat::capi::MeshResult_vertex_count(this->AsFFI());
    return result;
}

inline uint32_t MeshResult::triangle_count() const {
    auto result = diplomat::capi::MeshResult_triangle_count(this->AsFFI());
    return result;
}

inline bool MeshResult::has_transparency() const {
    auto result = diplomat::capi::MeshResult_has_transparency(this->AsFFI());
    return result;
}

inline MeshBounds MeshResult::bounds() const {
    auto result = diplomat::capi::MeshResult_bounds(this->AsFFI());
    return MeshBounds::FromFFI(result);
}

inline const diplomat::capi::MeshResult* MeshResult::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::MeshResult*>(this);
}

inline diplomat::capi::MeshResult* MeshResult::AsFFI() {
    return reinterpret_cast<diplomat::capi::MeshResult*>(this);
}

inline const MeshResult* MeshResult::FromFFI(const diplomat::capi::MeshResult* ptr) {
    return reinterpret_cast<const MeshResult*>(ptr);
}

inline MeshResult* MeshResult::FromFFI(diplomat::capi::MeshResult* ptr) {
    return reinterpret_cast<MeshResult*>(ptr);
}

inline void MeshResult::operator delete(void* ptr) {
    diplomat::capi::MeshResult_destroy(reinterpret_cast<diplomat::capi::MeshResult*>(ptr));
}


#endif // MeshResult_HPP
