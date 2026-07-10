#ifndef Scripting_H
#define Scripting_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "NucleationError.d.h"
#include "Schematic.d.h"

#include "Scripting.d.h"






typedef struct Scripting_run_lua_script_result {union {Schematic* ok; NucleationError err;}; bool is_ok;} Scripting_run_lua_script_result;
Scripting_run_lua_script_result Scripting_run_lua_script(DiplomatStringView path);

typedef struct Scripting_run_js_script_result {union {Schematic* ok; NucleationError err;}; bool is_ok;} Scripting_run_js_script_result;
Scripting_run_js_script_result Scripting_run_js_script(DiplomatStringView path);

typedef struct Scripting_run_script_result {union {Schematic* ok; NucleationError err;}; bool is_ok;} Scripting_run_script_result;
Scripting_run_script_result Scripting_run_script(DiplomatStringView path);

void Scripting_destroy(Scripting* self);





#endif // Scripting_H
