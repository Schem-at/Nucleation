package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface StoreLib: Library {
    fun Store_destroy(handle: Pointer)
    fun Store_open(url: Slice): ResultPointerInt
    fun Store_get_b64(handle: Pointer, key: Slice, write: Pointer): ResultUnitInt
    fun Store_put(handle: Pointer, key: Slice, data: Slice): ResultUnitInt
    fun Store_exists(handle: Pointer, key: Slice): ResultByteInt
    fun Store_delete(handle: Pointer, key: Slice): ResultUnitInt
    fun Store_list(handle: Pointer, prefix: Slice, write: Pointer): ResultUnitInt
    fun Store_put_if_absent(handle: Pointer, key: Slice, data: Slice): ResultByteInt
    fun Store_list_paginated(handle: Pointer, prefix: Slice, after: Slice, limit: FFIUint32, write: Pointer): ResultUnitInt
    fun Store_health(handle: Pointer): ResultUnitInt
    fun Store_open_schematic(handle: Pointer, key: Slice): ResultPointerInt
    fun Store_save_schematic(handle: Pointer, schematic: Pointer, key: Slice, version: Slice): ResultUnitInt
}
/** A key/value store opened from a URL (e.g. `mem://`, `file:///path`,
*`s3://bucket/prefix`, `redis://…`, `postgres://…`).
*/
class Store internal constructor (
    internal val handle: Pointer,
    // These ensure that anything that is borrowed is kept alive and not cleaned
    // up by the garbage collector.
    internal val selfEdges: List<Any>,
    internal var owned: Boolean,
)  {

    init {
        if (this.owned) {
            this.registerCleaner()
        }
    }

    private class StoreCleaner(val handle: Pointer, val lib: StoreLib) : Runnable {
        override fun run() {
            lib.Store_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, Store.StoreCleaner(handle, Store.lib));
    }

    companion object {
        internal val libClass: Class<StoreLib> = StoreLib::class.java
        internal val lib: StoreLib = Native.load("nucleation", libClass)
        @JvmStatic
        
        /** Open a store from a URL. Errors with `Store` on an unknown scheme or
        *connection failure.
        */
        fun open(url: String): Result<Store> {
            val urlSliceMemory = PrimitiveArrayTools.borrowUtf8(url)
            
            val returnVal = lib.Store_open(urlSliceMemory.slice);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal 
                    val returnOpaque = Store(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                urlSliceMemory.close()
            }
        }
    }
    
    /** Fetch `key`, writing the value as base64 (PORTING rule 6). Errors with
    *`NotFound` when the key is absent.
    */
    fun getB64(key: String): Result<String> {
        val keySliceMemory = PrimitiveArrayTools.borrowUtf8(key)
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Store_get_b64(handle, keySliceMemory.slice, write);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                
                val returnString = DW.writeToString(write)
                return returnString.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            keySliceMemory.close()
        }
    }
    
    /** Store `data` at `key`.
    */
    fun put(key: String, data: UByteArray): Result<Unit> {
        val keySliceMemory = PrimitiveArrayTools.borrowUtf8(key)
        val dataSliceMemory = PrimitiveArrayTools.borrow(data)
        
        val returnVal = lib.Store_put(handle, keySliceMemory.slice, dataSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            keySliceMemory.close()
            dataSliceMemory.close()
        }
    }
    
    /** Whether `key` exists.
    */
    fun exists(key: String): Result<Boolean> {
        val keySliceMemory = PrimitiveArrayTools.borrowUtf8(key)
        
        val returnVal = lib.Store_exists(handle, keySliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return (nativeOkVal > 0).ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            keySliceMemory.close()
        }
    }
    
    /** Delete `key` (idempotent).
    */
    fun delete(key: String): Result<Unit> {
        val keySliceMemory = PrimitiveArrayTools.borrowUtf8(key)
        
        val returnVal = lib.Store_delete(handle, keySliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            keySliceMemory.close()
        }
    }
    
    /** List keys under `prefix`, written as a JSON array string.
    */
    fun list(prefix: String): Result<String> {
        val prefixSliceMemory = PrimitiveArrayTools.borrowUtf8(prefix)
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Store_list(handle, prefixSliceMemory.slice, write);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                
                val returnString = DW.writeToString(write)
                return returnString.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            prefixSliceMemory.close()
        }
    }
    
    /** Atomically write `data` at `key` only if it does not already exist.
    *Returns `true` if written, `false` if the key existed.
    */
    fun putIfAbsent(key: String, data: UByteArray): Result<Boolean> {
        val keySliceMemory = PrimitiveArrayTools.borrowUtf8(key)
        val dataSliceMemory = PrimitiveArrayTools.borrow(data)
        
        val returnVal = lib.Store_put_if_absent(handle, keySliceMemory.slice, dataSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return (nativeOkVal > 0).ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            keySliceMemory.close()
            dataSliceMemory.close()
        }
    }
    
    /** A keyset page of keys under `prefix`. `after` is the exclusive cursor
    *(empty string for the first page); at most `limit` keys are returned.
    *Writes a JSON object string `{"keys":[...],"next":"…"|null}`.
    */
    fun listPaginated(prefix: String, after: String, limit: UInt): Result<String> {
        val prefixSliceMemory = PrimitiveArrayTools.borrowUtf8(prefix)
        val afterSliceMemory = PrimitiveArrayTools.borrowUtf8(after)
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Store_list_paginated(handle, prefixSliceMemory.slice, afterSliceMemory.slice, FFIUint32(limit), write);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                
                val returnString = DW.writeToString(write)
                return returnString.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            prefixSliceMemory.close()
            afterSliceMemory.close()
        }
    }
    
    /** Health check: `Ok` when the store is usable.
    */
    fun health(): Result<Unit> {
        
        val returnVal = lib.Store_health(handle);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    /** Open a schematic stored at `key` in this store. Works for every
    *backend, including `redis://`/`postgres://`/`mem://` that the
    *single-string URI form (`StoreIo::open`) rejects.
    */
    fun openSchematic(key: String): Result<Schematic> {
        val keySliceMemory = PrimitiveArrayTools.borrowUtf8(key)
        
        val returnVal = lib.Store_open_schematic(handle, keySliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal 
                val returnOpaque = Schematic(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            keySliceMemory.close()
        }
    }
    
    /** Save a schematic at `key` in this store. `version` selects the format
    *version (empty string = format default). Works for every backend,
    *including `redis://`/`postgres://`/`mem://` that the single-string URI
    *form (`StoreIo::save`) rejects.
    */
    fun saveSchematic(schematic: Schematic, key: String, version: String): Result<Unit> {
        val keySliceMemory = PrimitiveArrayTools.borrowUtf8(key)
        val versionSliceMemory = PrimitiveArrayTools.borrowUtf8(version)
        
        val returnVal = lib.Store_save_schematic(handle, schematic.handle, keySliceMemory.slice, versionSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            keySliceMemory.close()
            versionSliceMemory.close()
        }
    }

}