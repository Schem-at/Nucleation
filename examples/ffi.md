## 0 · Building with the FFI feature

```bash
# Build a cdylib that exposes the two functions below
cargo build --release --features ffi
# → target/release/libnucleation.{so|dll|dylib}
```

The crate must be compiled with **`--features ffi`** or these symbols will not be exported.

---

## 1 · Exposed symbols

| C name                 | Signature (C)                                                | Rust origin                                                                                                                          | What it does                                                                                                                 | Memory ownership |
| ---------------------- | ------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------- | ---------------- |
| `schematic_debug_info` | `char* schematic_debug_info(const UniversalSchematic* sch);` | Returns `CString` built from a short status string such as `"Schematic has 5 regions"` or `"null schematic"` if the pointer is null. | **Caller takes ownership** of the returned `char*` and must free it with `free()` *(C)* **or** `CString::from_raw` *(Rust)*. |                  |
| `print_debug_info`     | `void print_debug_info(const UniversalSchematic* sch);`      | Convenience wrapper that just calls `schematic_debug_info()`, prints the message to `stdout`, and **immediately frees the string**.  | No heap to free on the caller side.                                                                                          |                  |

---

## 2 · Usage examples

### 2.1  From plain C

```c
#include <stdio.h>
#include <stdlib.h>

// forward-decls (usually come from a generated binding header)
extern char* schematic_debug_info(const void* sch);
extern void  print_debug_info(const void* sch);

int main(void) {
    /* imagine you obtained a UniversalSchematic* from Rust
       via some extra constructor function */
    void* my_schematic = get_schematic_from_rust();

    // Option 1: fire-and-forget print
    print_debug_info(my_schematic);

    // Option 2: get the message
    char* msg = schematic_debug_info(my_schematic);
    printf("DEBUG (C side): %s\n", msg);
    free(msg);                           // YOU must free it!
    return 0;
}
```

### 2.2  From Rust (another crate)

```rust
#[link(name = "nucleation")]           // or the dynamic lib name on your target
extern "C" {
    fn schematic_debug_info(ptr: *const UniversalSchematic) -> *mut std::os::raw::c_char;
}

let sch: *const UniversalSchematic = obtain_schematic();
unsafe {
    let c_str = schematic_debug_info(sch);
    if !c_str.is_null() {
        println!("Rust FFI -> {}", std::ffi::CStr::from_ptr(c_str).to_string_lossy());
        // Convert back into CString to reclaim the allocation:
        let _ = CString::from_raw(c_str);
    }
}
```

---

## 3 · Meshing via FFI (feature = "ffi,meshing")

Build with both features to get meshing FFI functions:

```bash
cargo build --release --features ffi,meshing
```

### 3.1  From C

```c
#include <stdio.h>
#include <stdlib.h>

// Forward declarations (from nucleation FFI)
typedef void ResourcePackWrapper;
typedef void MeshConfigWrapper;
typedef void MeshResultWrapper;
typedef void RawMeshExportWrapper;

typedef struct { unsigned char* data; size_t len; } ByteArray;
typedef struct { char** data; size_t len; } StringArray;
typedef struct { float* data; size_t len; } FloatArray;
typedef struct { unsigned int* data; size_t len; } UintArray;

// Resource pack
extern ResourcePackWrapper* resourcepack_from_bytes(const unsigned char* data, size_t len);
extern void resourcepack_free(ResourcePackWrapper* ptr);
extern size_t resourcepack_blockstate_count(const ResourcePackWrapper* ptr);
extern StringArray resourcepack_namespaces(const ResourcePackWrapper* ptr);
extern StringArray resourcepack_list_blockstates(const ResourcePackWrapper* ptr);
extern char* resourcepack_get_blockstate_json(const ResourcePackWrapper* ptr, const char* name);

// Mesh config
extern MeshConfigWrapper* meshconfig_new();
extern void meshconfig_free(MeshConfigWrapper* ptr);
extern void meshconfig_set_greedy_meshing(MeshConfigWrapper* ptr, int val);
extern void meshconfig_set_cull_occluded_blocks(MeshConfigWrapper* ptr, int val);

// Meshing
extern MeshResultWrapper* schematic_to_mesh(
    const void* schematic, const ResourcePackWrapper* pack, const MeshConfigWrapper* config);
extern void meshresult_free(MeshResultWrapper* ptr);
extern ByteArray meshresult_glb_data(const MeshResultWrapper* ptr);
extern size_t meshresult_vertex_count(const MeshResultWrapper* ptr);
extern size_t meshresult_triangle_count(const MeshResultWrapper* ptr);

// Raw mesh
extern RawMeshExportWrapper* schematic_to_raw_mesh(
    const void* schematic, const ResourcePackWrapper* pack, const MeshConfigWrapper* config);
extern void rawmeshexport_free(RawMeshExportWrapper* ptr);
extern size_t rawmeshexport_vertex_count(const RawMeshExportWrapper* ptr);
extern FloatArray rawmeshexport_positions(const RawMeshExportWrapper* ptr);
extern UintArray rawmeshexport_indices(const RawMeshExportWrapper* ptr);

// Memory cleanup
extern void free_byte_array(ByteArray arr);
extern void free_string(char* ptr);
extern void free_string_array(StringArray arr);

int main(void) {
    // Load resource pack from file bytes
    FILE* f = fopen("resourcepack.zip", "rb");
    fseek(f, 0, SEEK_END);
    long fsize = ftell(f);
    fseek(f, 0, SEEK_SET);
    unsigned char* pack_data = malloc(fsize);
    fread(pack_data, 1, fsize, f);
    fclose(f);

    ResourcePackWrapper* pack = resourcepack_from_bytes(pack_data, fsize);
    free(pack_data);

    if (!pack) {
        printf("Failed to load resource pack\n");
        return 1;
    }

    printf("Blockstates: %zu\n", resourcepack_blockstate_count(pack));

    // Configure meshing
    MeshConfigWrapper* config = meshconfig_new();
    meshconfig_set_greedy_meshing(config, 1);
    meshconfig_set_cull_occluded_blocks(config, 1);

    // Generate mesh (assuming you have a schematic pointer)
    void* schematic = /* obtain from schematic_new / schematic_from_data */;
    MeshResultWrapper* result = schematic_to_mesh(schematic, pack, config);

    if (result) {
        printf("Vertices: %zu, Triangles: %zu\n",
            meshresult_vertex_count(result),
            meshresult_triangle_count(result));

        ByteArray glb = meshresult_glb_data(result);
        // Write glb.data (glb.len bytes) to file...
        free_byte_array(glb);
        meshresult_free(result);
    }

    meshconfig_free(config);
    resourcepack_free(pack);
    return 0;
}
```

---

## 4 · Important notes & gotchas

* **Thread safety** – The two functions are thread-safe as long as you never mutate the same `UniversalSchematic` from multiple threads without proper locking.
* **Null‐checking** – Both functions guard against `NULL` and produce a safe fallback string or message.
* **No constructors/destructors exported** – Your snippet only exposes *debug* helpers. In real code you'll need additional `extern "C"` functions to create/destroy `UniversalSchematic` instances, or capture pointers produced elsewhere in Rust.
* **Meshing FFI** – All meshing wrapper pointers (`ResourcePackWrapper*`, `MeshResultWrapper*`, etc.) must be freed with their corresponding `*_free()` functions. `ByteArray` results from mesh data must be freed with `free_byte_array()`.

