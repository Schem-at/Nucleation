#ifndef RedstoneGraph_H
#define RedstoneGraph_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "NucleationError.d.h"

#include "RedstoneGraph.d.h"






uint32_t RedstoneGraph_node_count(const RedstoneGraph* self);

uint32_t RedstoneGraph_edge_count(const RedstoneGraph* self);

typedef struct RedstoneGraph_nodes_json_result {union { NucleationError err;}; bool is_ok;} RedstoneGraph_nodes_json_result;
RedstoneGraph_nodes_json_result RedstoneGraph_nodes_json(const RedstoneGraph* self, DiplomatWrite* write);

typedef struct RedstoneGraph_edges_json_result {union { NucleationError err;}; bool is_ok;} RedstoneGraph_edges_json_result;
RedstoneGraph_edges_json_result RedstoneGraph_edges_json(const RedstoneGraph* self, DiplomatWrite* write);

typedef struct RedstoneGraph_features_json_result {union { NucleationError err;}; bool is_ok;} RedstoneGraph_features_json_result;
RedstoneGraph_features_json_result RedstoneGraph_features_json(const RedstoneGraph* self, DiplomatWrite* write);

typedef struct RedstoneGraph_fingerprint_result {union { NucleationError err;}; bool is_ok;} RedstoneGraph_fingerprint_result;
RedstoneGraph_fingerprint_result RedstoneGraph_fingerprint(const RedstoneGraph* self, DiplomatStringView preset, DiplomatWrite* write);

typedef struct RedstoneGraph_to_json_result {union { NucleationError err;}; bool is_ok;} RedstoneGraph_to_json_result;
RedstoneGraph_to_json_result RedstoneGraph_to_json(const RedstoneGraph* self, DiplomatWrite* write);

typedef struct RedstoneGraph_from_json_result {union {RedstoneGraph* ok; NucleationError err;}; bool is_ok;} RedstoneGraph_from_json_result;
RedstoneGraph_from_json_result RedstoneGraph_from_json(DiplomatStringView json);

void RedstoneGraph_destroy(RedstoneGraph* self);





#endif // RedstoneGraph_H
