#ifndef IoLayout_H
#define IoLayout_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"


#include "IoLayout.d.h"






void IoLayout_input_names_json(const IoLayout* self, DiplomatWrite* write);

void IoLayout_output_names_json(const IoLayout* self, DiplomatWrite* write);

void IoLayout_destroy(IoLayout* self);





#endif // IoLayout_H
