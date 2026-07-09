use super::*;

// --- World Streaming ---

/// Opaque handle wrapping a `ChunkIter` streaming iterator.
pub struct WorldStreamHandle(crate::formats::world_stream::ChunkIter);
/// Opaque handle wrapping a `WorldChunkView` (a single decoded chunk).
pub struct WorldChunkViewHandle(crate::formats::world_stream::WorldChunkView);
/// Opaque handle wrapping a `WorldSink` writer.
pub struct WorldSinkHandle(crate::formats::world_stream::WorldSink);

/// Open a streaming iterator over a world directory.
/// Returns NULL on error. Free with `worldstream_free`.
#[no_mangle]
pub extern "C" fn worldstream_open_dir(path: *const c_char) -> *mut WorldStreamHandle {
    if path.is_null() {
        return ptr::null_mut();
    }
    let path_str = match unsafe { CStr::from_ptr(path) }.to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    let source =
        match crate::formats::world_stream::WorldSource::open_dir(std::path::Path::new(path_str)) {
            Ok(s) => s,
            Err(_) => return ptr::null_mut(),
        };
    match source.chunks() {
        Ok(it) => Box::into_raw(Box::new(WorldStreamHandle(it))),
        Err(_) => ptr::null_mut(),
    }
}

/// Open a streaming iterator over a world directory, bounded to the given
/// block-coordinate box `[min_x..max_x, min_y..max_y, min_z..max_z]`.
/// Returns NULL on error. Free with `worldstream_free`.
#[no_mangle]
pub extern "C" fn worldstream_open_dir_bounded(
    path: *const c_char,
    min_x: c_int,
    min_y: c_int,
    min_z: c_int,
    max_x: c_int,
    max_y: c_int,
    max_z: c_int,
) -> *mut WorldStreamHandle {
    if path.is_null() {
        return ptr::null_mut();
    }
    let path_str = match unsafe { CStr::from_ptr(path) }.to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    let source =
        match crate::formats::world_stream::WorldSource::open_dir(std::path::Path::new(path_str)) {
            Ok(s) => s,
            Err(_) => return ptr::null_mut(),
        };
    match source.chunks_bounded((min_x, min_y, min_z), (max_x, max_y, max_z)) {
        Ok(it) => Box::into_raw(Box::new(WorldStreamHandle(it))),
        Err(_) => ptr::null_mut(),
    }
}

/// Open a streaming iterator from a zip archive in memory.
/// `data` must be valid for `len` bytes. Returns NULL on error.
/// Free with `worldstream_free`.
#[no_mangle]
pub extern "C" fn worldstream_from_zip(data: *const c_uchar, len: usize) -> *mut WorldStreamHandle {
    if data.is_null() {
        return ptr::null_mut();
    }
    let slice = unsafe { std::slice::from_raw_parts(data, len) };
    let source = match crate::formats::world_stream::WorldSource::from_zip_bytes(slice.to_vec()) {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    match source.chunks() {
        Ok(it) => Box::into_raw(Box::new(WorldStreamHandle(it))),
        Err(_) => ptr::null_mut(),
    }
}

/// Open a bounded streaming iterator from a zip archive in memory.
/// `data` must be valid for `len` bytes. Returns NULL on error.
/// Free with `worldstream_free`.
#[no_mangle]
pub extern "C" fn worldstream_from_zip_bounded(
    data: *const c_uchar,
    len: usize,
    min_x: c_int,
    min_y: c_int,
    min_z: c_int,
    max_x: c_int,
    max_y: c_int,
    max_z: c_int,
) -> *mut WorldStreamHandle {
    if data.is_null() {
        return ptr::null_mut();
    }
    let slice = unsafe { std::slice::from_raw_parts(data, len) };
    let source = match crate::formats::world_stream::WorldSource::from_zip_bytes(slice.to_vec()) {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    match source.chunks_bounded((min_x, min_y, min_z), (max_x, max_y, max_z)) {
        Ok(it) => Box::into_raw(Box::new(WorldStreamHandle(it))),
        Err(_) => ptr::null_mut(),
    }
}

/// Advance the iterator and return the next chunk view.
/// Returns NULL at end-of-stream. Corrupt chunks are silently skipped.
/// The returned pointer must be freed with `worldchunkview_free` (or passed
/// to `worldsink_write_chunk` / `worldsink_put_chunk`, which do not free it).
#[no_mangle]
pub extern "C" fn worldstream_next(handle: *mut WorldStreamHandle) -> *mut WorldChunkViewHandle {
    if handle.is_null() {
        return ptr::null_mut();
    }
    let it = unsafe { &mut (*handle).0 };
    loop {
        match it.next() {
            None => return ptr::null_mut(),
            Some(Ok(v)) => return Box::into_raw(Box::new(WorldChunkViewHandle(v))),
            Some(Err(_)) => continue, // documented: FFI skips corrupt chunks
        }
    }
}

/// Free a `WorldStreamHandle`. Safe to call with NULL.
#[no_mangle]
pub extern "C" fn worldstream_free(handle: *mut WorldStreamHandle) {
    if !handle.is_null() {
        unsafe {
            drop(Box::from_raw(handle));
        }
    }
}

/// Create an empty chunk view at the given chunk coordinates — the starting
/// point for generating worlds from scratch. Sections are created on demand
/// by `worldchunkview_set_block`. Serialized with `status = "minecraft:full"`
/// (Minecraft will not regenerate over it) and the default data version.
/// Never returns NULL except on allocation failure. Free with
/// `worldchunkview_free`, or pass to `worldsink_write_chunk` /
/// `worldsink_put_chunk` (which do NOT consume it).
#[no_mangle]
pub extern "C" fn worldchunkview_new(cx: c_int, cz: c_int) -> *mut WorldChunkViewHandle {
    Box::into_raw(Box::new(WorldChunkViewHandle(
        crate::formats::world_stream::WorldChunkView::new(cx, cz),
    )))
}

/// Return the chunk X coordinate (in chunk units) of a chunk view.
/// Returns -1 if `view` is NULL.
#[no_mangle]
pub extern "C" fn worldchunkview_cx(view: *const WorldChunkViewHandle) -> c_int {
    if view.is_null() {
        return -1;
    }
    unsafe { (*view).0.cx() }
}

/// Return the chunk Z coordinate (in chunk units) of a chunk view.
/// Returns -1 if `view` is NULL.
#[no_mangle]
pub extern "C" fn worldchunkview_cz(view: *const WorldChunkViewHandle) -> c_int {
    if view.is_null() {
        return -1;
    }
    unsafe { (*view).0.cz() }
}

/// Convert a chunk view to a standalone `SchematicWrapper`.
/// The returned pointer must be freed with `schematic_free`.
/// Returns NULL if `view` is NULL.
#[no_mangle]
pub extern "C" fn worldchunkview_to_schematic(
    view: *const WorldChunkViewHandle,
) -> *mut SchematicWrapper {
    if view.is_null() {
        return ptr::null_mut();
    }
    let s = unsafe { (*view).0.to_schematic() };
    // Mirror schematic_new's double-box pattern: SchematicWrapper holds a
    // *mut UniversalSchematic, then is itself boxed.
    let wrapper = SchematicWrapper(Box::into_raw(Box::new(s)));
    Box::into_raw(Box::new(wrapper))
}

/// Set a block at absolute world coordinates inside a chunk view.
/// `block_name` must be a valid Minecraft block identifier (e.g. `minecraft:stone`).
/// Returns 0 on success, -1 if any pointer is NULL, -2 if (x, z) is outside
/// this chunk's column.
#[no_mangle]
pub extern "C" fn worldchunkview_set_block(
    view: *mut WorldChunkViewHandle,
    x: c_int,
    y: c_int,
    z: c_int,
    block_name: *const c_char,
) -> c_int {
    if view.is_null() || block_name.is_null() {
        return -1;
    }
    let block_name_str = unsafe { CStr::from_ptr(block_name).to_string_lossy().into_owned() };
    let block_state = BlockState::new(block_name_str);
    let ok = unsafe { (*view).0.set_block(x, y, z, &block_state) };
    if ok {
        0
    } else {
        -2
    }
}

/// Overwrite the biome of every currently-present section of the chunk view
/// with `biome_name` (e.g. `minecraft:desert`). Sections are created lazily
/// by `worldchunkview_set_block`, so call this AFTER placing blocks.
/// Returns 0 on success, -1 if any pointer is NULL.
#[no_mangle]
pub extern "C" fn worldchunkview_set_biome(
    view: *mut WorldChunkViewHandle,
    biome_name: *const c_char,
) -> c_int {
    if view.is_null() || biome_name.is_null() {
        return -1;
    }
    let biome = unsafe { CStr::from_ptr(biome_name).to_string_lossy().into_owned() };
    unsafe { (*view).0.set_biome(&biome) };
    0
}

/// Deduped union of all sections' biome palette entries, in order of first
/// appearance; empty if no section carries biome data. Returns a StringArray
/// (NULL data / len 0 if `view` is NULL) that must be freed with
/// `free_string_array`.
#[no_mangle]
pub extern "C" fn worldchunkview_biome_palette(view: *const WorldChunkViewHandle) -> StringArray {
    if view.is_null() {
        return StringArray {
            data: ptr::null_mut(),
            len: 0,
        };
    }
    let names = unsafe { (*view).0.biome_palette() };
    let len = names.len();
    if len == 0 {
        return StringArray {
            data: ptr::null_mut(),
            len: 0,
        };
    }
    let results: Vec<*mut c_char> = names
        .into_iter()
        .map(|n| CString::new(n).unwrap().into_raw())
        .collect();
    let mut results = std::mem::ManuallyDrop::new(results);
    StringArray {
        data: results.as_mut_ptr(),
        len,
    }
}

/// Free a `WorldChunkViewHandle`. Safe to call with NULL.
#[no_mangle]
pub extern "C" fn worldchunkview_free(view: *mut WorldChunkViewHandle) {
    if !view.is_null() {
        unsafe {
            drop(Box::from_raw(view));
        }
    }
}

/// Create a new world sink that writes fresh chunk data to `dir`.
/// `options_json` may be NULL (uses defaults). Returns NULL on error.
/// Free with `worldsink_free` to abandon, or `worldsink_finish` to finalise.
#[no_mangle]
pub extern "C" fn worldsink_create(
    dir: *const c_char,
    options_json: *const c_char,
) -> *mut WorldSinkHandle {
    if dir.is_null() {
        return ptr::null_mut();
    }
    let dir_str = match unsafe { CStr::from_ptr(dir) }.to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    let options = if options_json.is_null() {
        None
    } else {
        let json_str = unsafe { CStr::from_ptr(options_json) };
        match json_str.to_str() {
            Ok(json) => {
                match serde_json::from_str::<crate::formats::world::WorldExportOptions>(json) {
                    Ok(opts) => Some(opts),
                    Err(_) => return ptr::null_mut(),
                }
            }
            Err(_) => return ptr::null_mut(),
        }
    };
    match crate::formats::world_stream::WorldSink::create(std::path::Path::new(dir_str), options) {
        Ok(sink) => Box::into_raw(Box::new(WorldSinkHandle(sink))),
        Err(_) => ptr::null_mut(),
    }
}

/// Open an existing world directory for patching via `worldsink_put_chunk`.
/// Returns NULL on error.
/// Free with `worldsink_free` to abandon, or `worldsink_finish` to finalise.
#[no_mangle]
pub extern "C" fn worldsink_open_existing(dir: *const c_char) -> *mut WorldSinkHandle {
    if dir.is_null() {
        return ptr::null_mut();
    }
    let dir_str = match unsafe { CStr::from_ptr(dir) }.to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    match crate::formats::world_stream::WorldSink::open_existing(std::path::Path::new(dir_str)) {
        Ok(sink) => Box::into_raw(Box::new(WorldSinkHandle(sink))),
        Err(_) => ptr::null_mut(),
    }
}

/// Write (append) a chunk view into the sink.
/// The `view` pointer is NOT consumed; free it separately with `worldchunkview_free`.
/// Returns 0 on success, -1 if any pointer is NULL, -2 on write error.
#[no_mangle]
pub extern "C" fn worldsink_write_chunk(
    sink: *mut WorldSinkHandle,
    view: *const WorldChunkViewHandle,
) -> c_int {
    if sink.is_null() || view.is_null() {
        return -1;
    }
    let sink_ref = unsafe { &mut (*sink).0 };
    let view_ref = unsafe { &(*view).0 };
    match sink_ref.write_chunk(view_ref) {
        Ok(()) => 0,
        Err(_) => -2,
    }
}

/// Overwrite the chunk at (`view.cx`, `view.cz`) of the sink's world with the
/// supplied view's block data. Only valid on sinks opened with
/// `worldsink_open_existing`; returns -2 on a create-mode sink.
/// The `view` pointer is NOT consumed; free it separately with `worldchunkview_free`.
/// Returns 0 on success, -1 if any pointer is NULL, -2 on error.
#[no_mangle]
pub extern "C" fn worldsink_put_chunk(
    sink: *mut WorldSinkHandle,
    view: *const WorldChunkViewHandle,
) -> c_int {
    if sink.is_null() || view.is_null() {
        return -1;
    }
    let sink_ref = unsafe { &mut (*sink).0 };
    let view_ref = unsafe { &(*view).0 };
    let cx = view_ref.cx();
    let cz = view_ref.cz();
    // Clone the view's ChunkData so the closure can take ownership.
    let data_clone = view_ref.data.clone();
    match sink_ref.patch_chunk(cx, cz, |chunk| {
        chunk.data = data_clone;
    }) {
        Ok(()) => 0,
        Err(_) => -2,
    }
}

/// Finalise and flush all pending writes for the sink.
///
/// **Ownership:** this function *consumes* the handle — it calls
/// `Box::from_raw` and then `WorldSink::finish`. After `worldsink_finish`
/// returns, the pointer is dangling and must NOT be passed to `worldsink_free`
/// or any other function.
///
/// Returns 0 on success, -1 if `sink` is NULL, -2 on I/O error.
#[no_mangle]
pub extern "C" fn worldsink_finish(sink: *mut WorldSinkHandle) -> c_int {
    if sink.is_null() {
        return -1;
    }
    // Consume the box — pointer is invalid after this point.
    let boxed = unsafe { Box::from_raw(sink) };
    match boxed.0.finish() {
        Ok(()) => 0,
        Err(_) => -2,
    }
}

/// Abandon a sink without finalising. Use this when discarding on error.
/// Do NOT call this after `worldsink_finish` — that function already frees the handle.
/// Safe to call with NULL.
#[no_mangle]
pub extern "C" fn worldsink_free(sink: *mut WorldSinkHandle) {
    if !sink.is_null() {
        unsafe {
            drop(Box::from_raw(sink));
        }
    }
}
