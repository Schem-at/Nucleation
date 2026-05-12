/*
 * Tiny shim layer for the Haskell example.
 *
 * GHC's FFI can't return C structs by value (struct-return ABI varies
 * across platforms, and `foreign import ccall` only accepts marshallable
 * primitives + Ptr). The Nucleation FFI returns a `ByteArray` struct
 * containing { data, len }; we provide hs_-prefixed wrappers that take
 * those fields as out-pointers / value pairs that GHC can handle.
 */

#include <stddef.h>
#include <stdint.h>

typedef struct {
    uint8_t *data;
    size_t   len;
} ByteArray;

/* Symbols defined in libnucleation. */
extern ByteArray schematic_to_litematic(const void *schem);
extern void      free_byte_array(ByteArray ba);

/* Out-parameter wrapper: writes the struct fields into caller-supplied
 * pointers so the Haskell side never has to handle struct-by-value. */
void hs_schematic_to_litematic(const void *schem,
                               uint8_t **out_data,
                               size_t *out_len) {
    ByteArray ba = schematic_to_litematic(schem);
    *out_data = ba.data;
    *out_len  = ba.len;
}

/* Mirror wrapper for freeing — recombines the two values into a ByteArray
 * and forwards. */
void hs_free_byte_array(uint8_t *data, size_t len) {
    ByteArray ba = { data, len };
    free_byte_array(ba);
}
