#ifndef Fingerprint_H
#define Fingerprint_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "NucleationError.d.h"
#include "Schematic.d.h"

#include "Fingerprint.d.h"






typedef struct Fingerprint_compute_result {union { NucleationError err;}; bool is_ok;} Fingerprint_compute_result;
Fingerprint_compute_result Fingerprint_compute(const Schematic* schematic, DiplomatStringView preset, DiplomatWrite* write);

typedef struct Fingerprint_signature_json_result {union { NucleationError err;}; bool is_ok;} Fingerprint_signature_json_result;
Fingerprint_signature_json_result Fingerprint_signature_json(const Schematic* schematic, DiplomatStringView preset, DiplomatWrite* write);

typedef struct Fingerprint_footprint_distance_result {union {float ok; NucleationError err;}; bool is_ok;} Fingerprint_footprint_distance_result;
Fingerprint_footprint_distance_result Fingerprint_footprint_distance(const Schematic* a, const Schematic* b, DiplomatStringView preset);

typedef struct Fingerprint_footprint_json_result {union { NucleationError err;}; bool is_ok;} Fingerprint_footprint_json_result;
Fingerprint_footprint_json_result Fingerprint_footprint_json(const Schematic* schematic, DiplomatStringView preset, DiplomatWrite* write);

typedef struct Fingerprint_is_duplicate_result {union {bool ok; NucleationError err;}; bool is_ok;} Fingerprint_is_duplicate_result;
Fingerprint_is_duplicate_result Fingerprint_is_duplicate(const Schematic* a, const Schematic* b, DiplomatStringView preset);

void Fingerprint_destroy(Fingerprint* self);





#endif // Fingerprint_H
