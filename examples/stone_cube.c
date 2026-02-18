#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// Nucleation FFI types
typedef struct {
    unsigned char *data;
    size_t len;
} ByteArray;

typedef void SchematicWrapper;

// Nucleation FFI functions
extern SchematicWrapper *schematic_new(void);
extern void schematic_free(SchematicWrapper *schematic);
extern int schematic_set_block(SchematicWrapper *schematic, int x, int y, int z, const char *block_name);
extern int schematic_set_name(SchematicWrapper *schematic, const char *name);
extern ByteArray schematic_to_schematic(const SchematicWrapper *schematic);
extern void free_byte_array(ByteArray array);
extern char *schematic_last_error(void);
extern void free_string(char *string);

int main(void) {
    int size = 10;

    // Create a new schematic
    SchematicWrapper *schem = schematic_new();
    if (!schem) {
        fprintf(stderr, "Failed to create schematic\n");
        return 1;
    }

    // Set the schematic name
    schematic_set_name(schem, "Stone Cube");

    // Fill a 10x10x10 cube with stone
    for (int x = 0; x < size; x++) {
        for (int y = 0; y < size; y++) {
            for (int z = 0; z < size; z++) {
                int rc = schematic_set_block(schem, x, y, z, "minecraft:stone");
                if (rc != 0) {
                    fprintf(stderr, "Failed to set block at (%d, %d, %d)\n", x, y, z);
                    schematic_free(schem);
                    return 1;
                }
            }
        }
    }

    printf("Set %d blocks\n", size * size * size);

    // Export to .schematic format
    ByteArray data = schematic_to_schematic(schem);
    if (!data.data || data.len == 0) {
        char *err = schematic_last_error();
        fprintf(stderr, "Failed to export schematic: %s\n", err ? err : "unknown error");
        if (err) free_string(err);
        schematic_free(schem);
        return 1;
    }

    // Write to file
    const char *filename = "stone_cube.schematic";
    FILE *f = fopen(filename, "wb");
    if (!f) {
        perror("fopen");
        free_byte_array(data);
        schematic_free(schem);
        return 1;
    }

    size_t written = fwrite(data.data, 1, data.len, f);
    fclose(f);

    if (written != data.len) {
        fprintf(stderr, "Failed to write all data\n");
        free_byte_array(data);
        schematic_free(schem);
        return 1;
    }

    printf("Saved %zu bytes to %s\n", data.len, filename);

    // Cleanup
    free_byte_array(data);
    schematic_free(schem);

    return 0;
}
