#ifndef RedstoneGraph_HPP
#define RedstoneGraph_HPP

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


namespace diplomat {
namespace capi {
    extern "C" {

    uint32_t RedstoneGraph_node_count(const diplomat::capi::RedstoneGraph* self);

    uint32_t RedstoneGraph_edge_count(const diplomat::capi::RedstoneGraph* self);

    typedef struct RedstoneGraph_nodes_json_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} RedstoneGraph_nodes_json_result;
    RedstoneGraph_nodes_json_result RedstoneGraph_nodes_json(const diplomat::capi::RedstoneGraph* self, diplomat::capi::DiplomatWrite* write);

    typedef struct RedstoneGraph_edges_json_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} RedstoneGraph_edges_json_result;
    RedstoneGraph_edges_json_result RedstoneGraph_edges_json(const diplomat::capi::RedstoneGraph* self, diplomat::capi::DiplomatWrite* write);

    typedef struct RedstoneGraph_features_json_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} RedstoneGraph_features_json_result;
    RedstoneGraph_features_json_result RedstoneGraph_features_json(const diplomat::capi::RedstoneGraph* self, diplomat::capi::DiplomatWrite* write);

    typedef struct RedstoneGraph_fingerprint_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} RedstoneGraph_fingerprint_result;
    RedstoneGraph_fingerprint_result RedstoneGraph_fingerprint(const diplomat::capi::RedstoneGraph* self, diplomat::capi::DiplomatStringView preset, diplomat::capi::DiplomatWrite* write);

    typedef struct RedstoneGraph_to_json_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} RedstoneGraph_to_json_result;
    RedstoneGraph_to_json_result RedstoneGraph_to_json(const diplomat::capi::RedstoneGraph* self, diplomat::capi::DiplomatWrite* write);

    typedef struct RedstoneGraph_from_json_result {union {diplomat::capi::RedstoneGraph* ok; diplomat::capi::NucleationError err;}; bool is_ok;} RedstoneGraph_from_json_result;
    RedstoneGraph_from_json_result RedstoneGraph_from_json(diplomat::capi::DiplomatStringView json);

    void RedstoneGraph_destroy(RedstoneGraph* self);

    } // extern "C"
} // namespace capi
} // namespace

inline uint32_t RedstoneGraph::node_count() const {
    auto result = diplomat::capi::RedstoneGraph_node_count(this->AsFFI());
    return result;
}

inline uint32_t RedstoneGraph::edge_count() const {
    auto result = diplomat::capi::RedstoneGraph_edge_count(this->AsFFI());
    return result;
}

inline diplomat::result<std::string, NucleationError> RedstoneGraph::nodes_json() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::RedstoneGraph_nodes_json(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> RedstoneGraph::nodes_json_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::RedstoneGraph_nodes_json(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> RedstoneGraph::edges_json() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::RedstoneGraph_edges_json(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> RedstoneGraph::edges_json_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::RedstoneGraph_edges_json(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> RedstoneGraph::features_json() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::RedstoneGraph_features_json(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> RedstoneGraph::features_json_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::RedstoneGraph_features_json(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> RedstoneGraph::fingerprint(std::string_view preset) const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::RedstoneGraph_fingerprint(this->AsFFI(),
        {preset.data(), preset.size()},
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> RedstoneGraph::fingerprint_write(std::string_view preset, W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::RedstoneGraph_fingerprint(this->AsFFI(),
        {preset.data(), preset.size()},
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> RedstoneGraph::to_json() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::RedstoneGraph_to_json(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> RedstoneGraph::to_json_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::RedstoneGraph_to_json(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<RedstoneGraph>, NucleationError> RedstoneGraph::from_json(std::string_view json) {
    auto result = diplomat::capi::RedstoneGraph_from_json({json.data(), json.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<RedstoneGraph>, NucleationError>(diplomat::Ok<std::unique_ptr<RedstoneGraph>>(std::unique_ptr<RedstoneGraph>(RedstoneGraph::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<RedstoneGraph>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline const diplomat::capi::RedstoneGraph* RedstoneGraph::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::RedstoneGraph*>(this);
}

inline diplomat::capi::RedstoneGraph* RedstoneGraph::AsFFI() {
    return reinterpret_cast<diplomat::capi::RedstoneGraph*>(this);
}

inline const RedstoneGraph* RedstoneGraph::FromFFI(const diplomat::capi::RedstoneGraph* ptr) {
    return reinterpret_cast<const RedstoneGraph*>(ptr);
}

inline RedstoneGraph* RedstoneGraph::FromFFI(diplomat::capi::RedstoneGraph* ptr) {
    return reinterpret_cast<RedstoneGraph*>(ptr);
}

inline void RedstoneGraph::operator delete(void* ptr) {
    diplomat::capi::RedstoneGraph_destroy(reinterpret_cast<diplomat::capi::RedstoneGraph*>(ptr));
}


#endif // RedstoneGraph_HPP
