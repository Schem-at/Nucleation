#ifndef Design_H
#define Design_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "NucleationError.d.h"
#include "Schematic.d.h"

#include "Design.d.h"






typedef struct Design_create_result {union {Design* ok; NucleationError err;}; bool is_ok;} Design_create_result;
Design_create_result Design_create(DiplomatStringView name);

typedef struct Design_for_schematic_result {union {Design* ok; NucleationError err;}; bool is_ok;} Design_for_schematic_result;
Design_for_schematic_result Design_for_schematic(DiplomatStringView name, const Schematic* base);

typedef struct Design_add_cell_result {union { NucleationError err;}; bool is_ok;} Design_add_cell_result;
Design_add_cell_result Design_add_cell(Design* self, DiplomatStringView name, const Schematic* cell, DiplomatWrite* write);

typedef struct Design_place_result {union { NucleationError err;}; bool is_ok;} Design_place_result;
Design_place_result Design_place(Design* self, DiplomatStringView name, DiplomatStringView cell, int32_t x, int32_t y, int32_t z, int32_t rot_y);

typedef struct Design_declare_input_result {union { NucleationError err;}; bool is_ok;} Design_declare_input_result;
Design_declare_input_result Design_declare_input(Design* self, DiplomatStringView name, int32_t ax, int32_t ay, int32_t az, int32_t sx, int32_t sy, int32_t sz, uint8_t width, DiplomatStringView ty);

typedef struct Design_declare_output_result {union { NucleationError err;}; bool is_ok;} Design_declare_output_result;
Design_declare_output_result Design_declare_output(Design* self, DiplomatStringView name, int32_t ax, int32_t ay, int32_t az, int32_t sx, int32_t sy, int32_t sz, uint8_t width, DiplomatStringView ty);

typedef struct Design_route_bus_result {union { NucleationError err;}; bool is_ok;} Design_route_bus_result;
Design_route_bus_result Design_route_bus(Design* self, DiplomatStringView name, DiplomatStringView driver, DiplomatStringView sinks_json, DiplomatStringView gates_json, DiplomatStringView style_json, DiplomatWrite* write);

typedef struct Design_route_bus_or_result {union { NucleationError err;}; bool is_ok;} Design_route_bus_or_result;
Design_route_bus_or_result Design_route_bus_or(Design* self, DiplomatStringView name, DiplomatStringView drivers_json, DiplomatStringView sinks_json, DiplomatStringView gates_json, DiplomatStringView style_json, DiplomatWrite* write);

typedef struct Design_set_block_result {union { NucleationError err;}; bool is_ok;} Design_set_block_result;
Design_set_block_result Design_set_block(Design* self, int32_t x, int32_t y, int32_t z, DiplomatStringView block);

typedef struct Design_move_instance_result {union { NucleationError err;}; bool is_ok;} Design_move_instance_result;
Design_move_instance_result Design_move_instance(Design* self, DiplomatStringView name, int32_t x, int32_t y, int32_t z, int32_t rot_y, DiplomatWrite* write);

typedef struct Design_add_gate_result {union { NucleationError err;}; bool is_ok;} Design_add_gate_result;
Design_add_gate_result Design_add_gate(Design* self, DiplomatStringView bus, DiplomatStringView gate, int32_t x, int32_t y, int32_t z, int32_t sx, int32_t sy, int32_t sz, DiplomatWrite* write);

typedef struct Design_move_gate_result {union { NucleationError err;}; bool is_ok;} Design_move_gate_result;
Design_move_gate_result Design_move_gate(Design* self, DiplomatStringView bus, DiplomatStringView gate, int32_t x, int32_t y, int32_t z, DiplomatWrite* write);

typedef struct Design_set_bus_rule_result {union { NucleationError err;}; bool is_ok;} Design_set_bus_rule_result;
Design_set_bus_rule_result Design_set_bus_rule(Design* self, DiplomatStringView bus, DiplomatStringView rule_json);

typedef struct Design_bus_skew_result {union { NucleationError err;}; bool is_ok;} Design_bus_skew_result;
Design_bus_skew_result Design_bus_skew(const Design* self, DiplomatStringView name, DiplomatWrite* write);

typedef struct Design_bus_state_result {union { NucleationError err;}; bool is_ok;} Design_bus_state_result;
Design_bus_state_result Design_bus_state(const Design* self, DiplomatStringView name, DiplomatWrite* write);

typedef struct Design_rip_result {union { NucleationError err;}; bool is_ok;} Design_rip_result;
Design_rip_result Design_rip(Design* self, DiplomatStringView name);

typedef struct Design_flatten_result {union {Schematic* ok; NucleationError err;}; bool is_ok;} Design_flatten_result;
Design_flatten_result Design_flatten(const Design* self);

typedef struct Design_check_result {union { NucleationError err;}; bool is_ok;} Design_check_result;
Design_check_result Design_check(const Design* self, DiplomatWrite* write);

typedef struct Design_bake_result {union {Schematic* ok; NucleationError err;}; bool is_ok;} Design_bake_result;
Design_bake_result Design_bake(const Design* self, uint32_t budget);

void Design_destroy(Design* self);





#endif // Design_H
