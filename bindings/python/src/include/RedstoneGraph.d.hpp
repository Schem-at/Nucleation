#ifndef NUCLEATION_RedstoneGraph_D_HPP
#define NUCLEATION_RedstoneGraph_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"
namespace nucleation {
namespace capi { struct RedstoneGraph; }
class RedstoneGraph;
class NucleationError;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct RedstoneGraph;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * An extracted redstone logic graph. Wraps
 * {@link crate::simulation::graph::RedstoneGraph}.
 */
class RedstoneGraph {
public:

  /**
   * Number of nodes in the graph.
   */
  inline uint32_t node_count() const;

  /**
   * Total number of directed edges in the graph.
   */
  inline uint32_t edge_count() const;

  /**
   * The nodes as a JSON array string.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nodes_json() const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nodes_json_write(W& writeable_output) const;

  /**
   * The directed edges as a JSON array string.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> edges_json() const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> edges_json_write(W& writeable_output) const;

  /**
   * Computed graph features as a JSON object string.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> features_json() const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> features_json_write(W& writeable_output) const;

  /**
   * Fingerprint (hex string) for `preset` ("structural" | "functional" |
   * "exact"; empty defaults to "structural").
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> fingerprint(std::string_view preset) const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> fingerprint_write(std::string_view preset, W& writeable_output) const;

  /**
   * Serialize the whole graph to JSON.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> to_json() const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> to_json_write(W& writeable_output) const;

  /**
   * Deserialize a graph from a JSON string.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::RedstoneGraph>, nucleation::NucleationError> from_json(std::string_view json);

    inline const nucleation::capi::RedstoneGraph* AsFFI() const;
    inline nucleation::capi::RedstoneGraph* AsFFI();
    inline static const nucleation::RedstoneGraph* FromFFI(const nucleation::capi::RedstoneGraph* ptr);
    inline static nucleation::RedstoneGraph* FromFFI(nucleation::capi::RedstoneGraph* ptr);
    inline static void operator delete(void* ptr);
private:
    RedstoneGraph() = delete;
    RedstoneGraph(const nucleation::RedstoneGraph&) = delete;
    RedstoneGraph(nucleation::RedstoneGraph&&) noexcept = delete;
    RedstoneGraph operator=(const nucleation::RedstoneGraph&) = delete;
    RedstoneGraph operator=(nucleation::RedstoneGraph&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_RedstoneGraph_D_HPP
