package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface AnimationEffectLib: Library {
    fun AnimationEffect_destroy(handle: Pointer)
    fun AnimationEffect_create(durationMs: Float): Pointer
    fun AnimationEffect_instant(): Pointer
    fun AnimationEffect_pop_in(durationMs: Float): Pointer
    fun AnimationEffect_drop_in(durationMs: Float, height: Float): Pointer
    fun AnimationEffect_drop_and_pop(durationMs: Float, height: Float): Pointer
    fun AnimationEffect_spin_in(durationMs: Float, turns: Float): Pointer
    fun AnimationEffect_turntable(durationMs: Float): Pointer
    fun AnimationEffect_add_tween(handle: Pointer, propertyName: Slice, from: Float, to: Float, easingName: Slice): ResultUnitInt
    fun AnimationEffect_add_keyframe(handle: Pointer, propertyName: Slice, at: Float, value: Float, easingName: Slice): ResultUnitInt
    fun AnimationEffect_set_repeat_forever(handle: Pointer): Unit
}
/** A reusable set of property tracks, modelled after Anime.js object animations.
*/
class AnimationEffect internal constructor (
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

    private class AnimationEffectCleaner(val handle: Pointer, val lib: AnimationEffectLib) : Runnable {
        override fun run() {
            lib.AnimationEffect_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, AnimationEffect.AnimationEffectCleaner(handle, AnimationEffect.lib));
    }

    companion object {
        internal val libClass: Class<AnimationEffectLib> = AnimationEffectLib::class.java
        internal val lib: AnimationEffectLib = Native.load("nucleation", libClass)
        @JvmStatic
        
        fun create(durationMs: Float): AnimationEffect {
            
            val returnVal = lib.AnimationEffect_create(durationMs);
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = AnimationEffect(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic
        
        fun instant(): AnimationEffect {
            
            val returnVal = lib.AnimationEffect_instant();
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = AnimationEffect(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic
        
        fun popIn(durationMs: Float): AnimationEffect {
            
            val returnVal = lib.AnimationEffect_pop_in(durationMs);
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = AnimationEffect(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic
        
        fun dropIn(durationMs: Float, height: Float): AnimationEffect {
            
            val returnVal = lib.AnimationEffect_drop_in(durationMs, height);
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = AnimationEffect(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic
        
        fun dropAndPop(durationMs: Float, height: Float): AnimationEffect {
            
            val returnVal = lib.AnimationEffect_drop_and_pop(durationMs, height);
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = AnimationEffect(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic
        
        fun spinIn(durationMs: Float, turns: Float): AnimationEffect {
            
            val returnVal = lib.AnimationEffect_spin_in(durationMs, turns);
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = AnimationEffect(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic
        
        fun turntable(durationMs: Float): AnimationEffect {
            
            val returnVal = lib.AnimationEffect_turntable(durationMs);
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = AnimationEffect(handle, selfEdges, true)
            return returnOpaque
        }
    }
    
    /** Add a two-key property tween. Property names follow Anime.js/Three.js:
    *`x`, `y`, `z`, `rotateX`, `rotateY`, `rotateZ`, `scale`, `opacity`,
    *`tintR/G/B/A`, and `emissiveR/G/B`.
    */
    fun addTween(propertyName: String, from: Float, to: Float, easingName: String): Result<Unit> {
        val propertyNameSliceMemory = PrimitiveArrayTools.borrowUtf8(propertyName)
        val easingNameSliceMemory = PrimitiveArrayTools.borrowUtf8(easingName)
        
        val returnVal = lib.AnimationEffect_add_tween(handle, propertyNameSliceMemory.slice, from, to, easingNameSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            propertyNameSliceMemory.close()
            easingNameSliceMemory.close()
        }
    }
    
    /** Add a normalised keyframe (`at` in `0..=1`) to a property track.
    */
    fun addKeyframe(propertyName: String, at: Float, value: Float, easingName: String): Result<Unit> {
        val propertyNameSliceMemory = PrimitiveArrayTools.borrowUtf8(propertyName)
        val easingNameSliceMemory = PrimitiveArrayTools.borrowUtf8(easingName)
        
        val returnVal = lib.AnimationEffect_add_keyframe(handle, propertyNameSliceMemory.slice, at, value, easingNameSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            propertyNameSliceMemory.close()
            easingNameSliceMemory.close()
        }
    }
    
    fun setRepeatForever(): Unit {
        
        val returnVal = lib.AnimationEffect_set_repeat_forever(handle);
        
    }

}