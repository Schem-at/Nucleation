#ifndef Routing_H
#define Routing_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "NucleationError.d.h"
#include "Schematic.d.h"

#include "Routing.d.h"






typedef struct Routing_route_net_result {union { NucleationError err;}; bool is_ok;} Routing_route_net_result;
Routing_route_net_result Routing_route_net(Schematic* schematic, int32_t sx, int32_t sy, int32_t sz, int32_t dx, int32_t dy, int32_t dz, DiplomatStringView label, DiplomatWrite* write);

typedef struct Routing_route_all_result {union { NucleationError err;}; bool is_ok;} Routing_route_all_result;
Routing_route_all_result Routing_route_all(Schematic* schematic, DiplomatStringView nets_json, DiplomatWrite* write);

typedef struct Routing_lvs_result {union { NucleationError err;}; bool is_ok;} Routing_lvs_result;
Routing_lvs_result Routing_lvs(const Schematic* schematic, DiplomatStringView intent_json, DiplomatWrite* write);

typedef struct Routing_drc_result {union { NucleationError err;}; bool is_ok;} Routing_drc_result;
Routing_drc_result Routing_drc(const Schematic* schematic, bool check_decay, DiplomatWrite* write);

typedef struct Routing_sta_result {union { NucleationError err;}; bool is_ok;} Routing_sta_result;
Routing_sta_result Routing_sta(const Schematic* schematic, DiplomatStringView netlist_json, DiplomatWrite* write);

void Routing_destroy(Routing* self);





#endif // Routing_H
