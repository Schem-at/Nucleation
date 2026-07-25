#ifndef GeneratedChunk_H
#define GeneratedChunk_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "GeneratedChunkCoverage.d.h"
#include "NucleationError.d.h"
#include "WorldChunkView.d.h"

#include "GeneratedChunk.d.h"






typedef struct GeneratedChunk_cx_result {union {int32_t ok; NucleationError err;}; bool is_ok;} GeneratedChunk_cx_result;
GeneratedChunk_cx_result GeneratedChunk_cx(const GeneratedChunk* self);

typedef struct GeneratedChunk_cz_result {union {int32_t ok; NucleationError err;}; bool is_ok;} GeneratedChunk_cz_result;
GeneratedChunk_cz_result GeneratedChunk_cz(const GeneratedChunk* self);

typedef struct GeneratedChunk_coverage_result {union {GeneratedChunkCoverage ok; NucleationError err;}; bool is_ok;} GeneratedChunk_coverage_result;
GeneratedChunk_coverage_result GeneratedChunk_coverage(const GeneratedChunk* self);

typedef struct GeneratedChunk_source_id_result {union { NucleationError err;}; bool is_ok;} GeneratedChunk_source_id_result;
GeneratedChunk_source_id_result GeneratedChunk_source_id(const GeneratedChunk* self, DiplomatWrite* write);

typedef struct GeneratedChunk_version_result {union { NucleationError err;}; bool is_ok;} GeneratedChunk_version_result;
GeneratedChunk_version_result GeneratedChunk_version(const GeneratedChunk* self, DiplomatWrite* write);

typedef struct GeneratedChunk_take_view_result {union {WorldChunkView* ok; NucleationError err;}; bool is_ok;} GeneratedChunk_take_view_result;
GeneratedChunk_take_view_result GeneratedChunk_take_view(GeneratedChunk* self);

void GeneratedChunk_destroy(GeneratedChunk* self);





#endif // GeneratedChunk_H
