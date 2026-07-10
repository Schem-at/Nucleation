#ifndef Diff_H
#define Diff_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "NucleationError.d.h"
#include "Schematic.d.h"

#include "Diff.d.h"






typedef struct Diff_compute_result {union {Diff* ok; NucleationError err;}; bool is_ok;} Diff_compute_result;
Diff_compute_result Diff_compute(const Schematic* a, const Schematic* b, DiplomatStringView preset);

typedef struct Diff_compute_with_opts_result {union {Diff* ok; NucleationError err;}; bool is_ok;} Diff_compute_with_opts_result;
Diff_compute_with_opts_result Diff_compute_with_opts(const Schematic* a, const Schematic* b, DiplomatStringView preset, int32_t cost_add, int32_t cost_delete, int32_t cost_change, int32_t cost_swap, DiplomatStringView symmetry);

typedef struct Diff_from_json_result {union {Diff* ok; NucleationError err;}; bool is_ok;} Diff_from_json_result;
Diff_from_json_result Diff_from_json(DiplomatStringView json);

uint64_t Diff_distance(const Diff* self);

float Diff_support(const Diff* self);

void Diff_to_json(const Diff* self, DiplomatWrite* write);

void Diff_summary_json(const Diff* self, DiplomatWrite* write);

Schematic* Diff_added(const Diff* self);

Schematic* Diff_removed(const Diff* self);

Schematic* Diff_changed(const Diff* self);

Schematic* Diff_swapped(const Diff* self);

Schematic* Diff_markers(const Diff* self);

typedef struct Diff_to_overlay_glb_b64_result {union { NucleationError err;}; bool is_ok;} Diff_to_overlay_glb_b64_result;
Diff_to_overlay_glb_b64_result Diff_to_overlay_glb_b64(const Diff* self, DiplomatU8View after_glb, DiplomatWrite* write);

void Diff_destroy(Diff* self);





#endif // Diff_H
