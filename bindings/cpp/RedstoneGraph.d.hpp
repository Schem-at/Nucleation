#ifndef RedstoneGraph_D_HPP
#define RedstoneGraph_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"

class NucleationError;




namespace diplomat {
namespace capi {
    struct RedstoneGraph;
} // namespace capi
} // namespace

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
  inline diplomat::result<std::string, NucleationError> nodes_json() const;
  template<typename W>
  inline diplomat::result<std::monostate, NucleationError> nodes_json_write(W& writeable_output) const;

  /**
   * The directed edges as a JSON array string.
   */
  inline diplomat::result<std::string, NucleationError> edges_json() const;
  template<typename W>
  inline diplomat::result<std::monostate, NucleationError> edges_json_write(W& writeable_output) const;

  /**
   * Computed graph features as a JSON object string.
   */
  inline diplomat::result<std::string, NucleationError> features_json() const;
  template<typename W>
  inline diplomat::result<std::monostate, NucleationError> features_json_write(W& writeable_output) const;

  /**
   * Fingerprint (hex string) for `preset` ("structural" | "functional" |
   * "exact"; empty defaults to "structural").
   */
  inline diplomat::result<std::string, NucleationError> fingerprint(std::string_view preset) const;
  template<typename W>
  inline diplomat::result<std::monostate, NucleationError> fingerprint_write(std::string_view preset, W& writeable_output) const;

  /**
   * Serialize the whole graph to JSON.
   */
  inline diplomat::result<std::string, NucleationError> to_json() const;
  template<typename W>
  inline diplomat::result<std::monostate, NucleationError> to_json_write(W& writeable_output) const;

  /**
   * Deserialize a graph from a JSON string.
   */
  inline static diplomat::result<std::unique_ptr<RedstoneGraph>, NucleationError> from_json(std::string_view json);

    inline const diplomat::capi::RedstoneGraph* AsFFI() const;
    inline diplomat::capi::RedstoneGraph* AsFFI();
    inline static const RedstoneGraph* FromFFI(const diplomat::capi::RedstoneGraph* ptr);
    inline static RedstoneGraph* FromFFI(diplomat::capi::RedstoneGraph* ptr);
    inline static void operator delete(void* ptr);
private:
    RedstoneGraph() = delete;
    RedstoneGraph(const RedstoneGraph&) = delete;
    RedstoneGraph(RedstoneGraph&&) noexcept = delete;
    RedstoneGraph operator=(const RedstoneGraph&) = delete;
    RedstoneGraph operator=(RedstoneGraph&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // RedstoneGraph_D_HPP
