#ifndef NUCLEATION_RedstoneGraph_HPP
#define NUCLEATION_RedstoneGraph_HPP

#include "RedstoneGraph.d.hpp"

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

    uint32_t RedstoneGraph_node_count(const nucleation::capi::RedstoneGraph* self);

    uint32_t RedstoneGraph_edge_count(const nucleation::capi::RedstoneGraph* self);

    typedef struct RedstoneGraph_nodes_json_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} RedstoneGraph_nodes_json_result;
    RedstoneGraph_nodes_json_result RedstoneGraph_nodes_json(const nucleation::capi::RedstoneGraph* self, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct RedstoneGraph_edges_json_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} RedstoneGraph_edges_json_result;
    RedstoneGraph_edges_json_result RedstoneGraph_edges_json(const nucleation::capi::RedstoneGraph* self, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct RedstoneGraph_features_json_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} RedstoneGraph_features_json_result;
    RedstoneGraph_features_json_result RedstoneGraph_features_json(const nucleation::capi::RedstoneGraph* self, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct RedstoneGraph_fingerprint_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} RedstoneGraph_fingerprint_result;
    RedstoneGraph_fingerprint_result RedstoneGraph_fingerprint(const nucleation::capi::RedstoneGraph* self, nucleation::diplomat::capi::DiplomatStringView preset, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct RedstoneGraph_to_json_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} RedstoneGraph_to_json_result;
    RedstoneGraph_to_json_result RedstoneGraph_to_json(const nucleation::capi::RedstoneGraph* self, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct RedstoneGraph_from_json_result {union {nucleation::capi::RedstoneGraph* ok; nucleation::capi::NucleationError err;}; bool is_ok;} RedstoneGraph_from_json_result;
    RedstoneGraph_from_json_result RedstoneGraph_from_json(nucleation::diplomat::capi::DiplomatStringView json);

    void RedstoneGraph_destroy(RedstoneGraph* self);

    } // extern "C"
} // namespace capi
} // namespace

inline uint32_t nucleation::RedstoneGraph::node_count() const {
    auto result = nucleation::capi::RedstoneGraph_node_count(this->AsFFI());
    return result;
}

inline uint32_t nucleation::RedstoneGraph::edge_count() const {
    auto result = nucleation::capi::RedstoneGraph_edge_count(this->AsFFI());
    return result;
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::RedstoneGraph::nodes_json() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::RedstoneGraph_nodes_json(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::RedstoneGraph::nodes_json_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::RedstoneGraph_nodes_json(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::RedstoneGraph::edges_json() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::RedstoneGraph_edges_json(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::RedstoneGraph::edges_json_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::RedstoneGraph_edges_json(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::RedstoneGraph::features_json() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::RedstoneGraph_features_json(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::RedstoneGraph::features_json_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::RedstoneGraph_features_json(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::RedstoneGraph::fingerprint(std::string_view preset) const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::RedstoneGraph_fingerprint(this->AsFFI(),
        {preset.data(), preset.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::RedstoneGraph::fingerprint_write(std::string_view preset, W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::RedstoneGraph_fingerprint(this->AsFFI(),
        {preset.data(), preset.size()},
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::RedstoneGraph::to_json() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::RedstoneGraph_to_json(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::RedstoneGraph::to_json_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::RedstoneGraph_to_json(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::RedstoneGraph>, nucleation::NucleationError> nucleation::RedstoneGraph::from_json(std::string_view json) {
    auto result = nucleation::capi::RedstoneGraph_from_json({json.data(), json.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::RedstoneGraph>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::RedstoneGraph>>(std::unique_ptr<nucleation::RedstoneGraph>(nucleation::RedstoneGraph::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::RedstoneGraph>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline const nucleation::capi::RedstoneGraph* nucleation::RedstoneGraph::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::RedstoneGraph*>(this);
}

inline nucleation::capi::RedstoneGraph* nucleation::RedstoneGraph::AsFFI() {
    return reinterpret_cast<nucleation::capi::RedstoneGraph*>(this);
}

inline const nucleation::RedstoneGraph* nucleation::RedstoneGraph::FromFFI(const nucleation::capi::RedstoneGraph* ptr) {
    return reinterpret_cast<const nucleation::RedstoneGraph*>(ptr);
}

inline nucleation::RedstoneGraph* nucleation::RedstoneGraph::FromFFI(nucleation::capi::RedstoneGraph* ptr) {
    return reinterpret_cast<nucleation::RedstoneGraph*>(ptr);
}

inline void nucleation::RedstoneGraph::operator delete(void* ptr) {
    nucleation::capi::RedstoneGraph_destroy(reinterpret_cast<nucleation::capi::RedstoneGraph*>(ptr));
}


#endif // NUCLEATION_RedstoneGraph_HPP
