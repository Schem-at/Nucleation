#ifndef Store_H
#define Store_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "NucleationError.d.h"
#include "Schematic.d.h"

#include "Store.d.h"






typedef struct Store_open_result {union {Store* ok; NucleationError err;}; bool is_ok;} Store_open_result;
Store_open_result Store_open(DiplomatStringView url);

typedef struct Store_get_b64_result {union { NucleationError err;}; bool is_ok;} Store_get_b64_result;
Store_get_b64_result Store_get_b64(const Store* self, DiplomatStringView key, DiplomatWrite* write);

typedef struct Store_put_result {union { NucleationError err;}; bool is_ok;} Store_put_result;
Store_put_result Store_put(const Store* self, DiplomatStringView key, DiplomatU8View data);

typedef struct Store_exists_result {union {bool ok; NucleationError err;}; bool is_ok;} Store_exists_result;
Store_exists_result Store_exists(const Store* self, DiplomatStringView key);

typedef struct Store_delete_result {union { NucleationError err;}; bool is_ok;} Store_delete_result;
Store_delete_result Store_delete(const Store* self, DiplomatStringView key);

typedef struct Store_list_result {union { NucleationError err;}; bool is_ok;} Store_list_result;
Store_list_result Store_list(const Store* self, DiplomatStringView prefix, DiplomatWrite* write);

typedef struct Store_put_if_absent_result {union {bool ok; NucleationError err;}; bool is_ok;} Store_put_if_absent_result;
Store_put_if_absent_result Store_put_if_absent(const Store* self, DiplomatStringView key, DiplomatU8View data);

typedef struct Store_list_paginated_result {union { NucleationError err;}; bool is_ok;} Store_list_paginated_result;
Store_list_paginated_result Store_list_paginated(const Store* self, DiplomatStringView prefix, DiplomatStringView after, uint32_t limit, DiplomatWrite* write);

typedef struct Store_health_result {union { NucleationError err;}; bool is_ok;} Store_health_result;
Store_health_result Store_health(const Store* self);

typedef struct Store_open_schematic_result {union {Schematic* ok; NucleationError err;}; bool is_ok;} Store_open_schematic_result;
Store_open_schematic_result Store_open_schematic(const Store* self, DiplomatStringView key);

typedef struct Store_save_schematic_result {union { NucleationError err;}; bool is_ok;} Store_save_schematic_result;
Store_save_schematic_result Store_save_schematic(const Store* self, const Schematic* schematic, DiplomatStringView key, DiplomatStringView version);

void Store_destroy(Store* self);





#endif // Store_H
