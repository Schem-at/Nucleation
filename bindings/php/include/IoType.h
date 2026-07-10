#ifndef IoType_H
#define IoType_H

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include "diplomat_runtime.h"


#include "IoType.d.h"






IoType* IoType_unsigned_int(uint32_t bits);

IoType* IoType_signed_int(uint32_t bits);

IoType* IoType_float32(void);

IoType* IoType_boolean(void);

IoType* IoType_ascii(uint32_t chars);

void IoType_destroy(IoType* self);





#endif // IoType_H
