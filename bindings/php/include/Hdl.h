#ifndef Hdl_H
#define Hdl_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "NucleationError.d.h"
#include "Schematic.d.h"

#include "Hdl.d.h"






typedef struct Hdl_compile_blif_result {union {Schematic* ok; NucleationError err;}; bool is_ok;} Hdl_compile_blif_result;
Hdl_compile_blif_result Hdl_compile_blif(DiplomatStringView blif, DiplomatStringView name, bool bake);

typedef struct Hdl_compile_blif_report_result {union { NucleationError err;}; bool is_ok;} Hdl_compile_blif_report_result;
Hdl_compile_blif_report_result Hdl_compile_blif_report(DiplomatStringView blif, DiplomatStringView name, DiplomatWrite* write);

typedef struct Hdl_compile_blif_contract_result {union { NucleationError err;}; bool is_ok;} Hdl_compile_blif_contract_result;
Hdl_compile_blif_contract_result Hdl_compile_blif_contract(DiplomatStringView blif, DiplomatStringView name, DiplomatWrite* write);

void Hdl_destroy(Hdl* self);





#endif // Hdl_H
