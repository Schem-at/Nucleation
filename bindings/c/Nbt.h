#ifndef Nbt_H
#define Nbt_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"

#include "NucleationError.d.h"

#include "Nbt.d.h"






typedef struct Nbt_text_build_result {union { NucleationError err;}; bool is_ok;} Nbt_text_build_result;
Nbt_text_build_result Nbt_text_build(DiplomatStringView s, DiplomatStringView color, int32_t bold, int32_t italic, DiplomatWrite* write);

typedef struct Nbt_chest_build_result {union { NucleationError err;}; bool is_ok;} Nbt_chest_build_result;
Nbt_chest_build_result Nbt_chest_build(DiplomatStringView items_json, DiplomatStringView name, DiplomatWrite* write);

typedef struct Nbt_sign_build_result {union { NucleationError err;}; bool is_ok;} Nbt_sign_build_result;
Nbt_sign_build_result Nbt_sign_build(DiplomatStringView front_json, DiplomatStringView back_json, DiplomatStringView color, bool glowing, bool waxed, DiplomatWrite* write);

void Nbt_destroy(Nbt* self);





#endif // Nbt_H
