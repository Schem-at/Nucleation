package at.schem.nucleation

/**
 * Java-friendly, exception-based façade for the generated [Sdf] API.
 *
 * Diplomat maps checked Rust results to Kotlin `Result`, which is natural in
 * Kotlin but has an unusable erased/mangled surface from Java. This façade
 * keeps validation and native errors while returning composable values
 * directly. [raw] exposes the generated object when lower-level access is
 * needed.
 */
class SdfExpr private constructor(val raw: Sdf) {
    companion object {
        @JvmStatic fun sphere(radius: Float) = SdfExpr(Sdf.sphere(radius).getOrThrow())
        @JvmStatic fun boxShape(halfX: Float, halfY: Float, halfZ: Float, rounding: Float) =
            SdfExpr(Sdf.boxShape(halfX, halfY, halfZ, rounding).getOrThrow())
        @JvmStatic fun ellipsoid(radiusX: Float, radiusY: Float, radiusZ: Float) =
            SdfExpr(Sdf.ellipsoid(radiusX, radiusY, radiusZ).getOrThrow())
        @JvmStatic fun torus(majorRadius: Float, minorRadius: Float) =
            SdfExpr(Sdf.torus(majorRadius, minorRadius).getOrThrow())
        @JvmStatic fun capsule(
            ax: Float, ay: Float, az: Float,
            bx: Float, by: Float, bz: Float,
            radius: Float,
        ) = SdfExpr(Sdf.capsule(ax, ay, az, bx, by, bz, radius).getOrThrow())
        @JvmStatic fun cappedCylinder(radius: Float, halfHeight: Float) =
            SdfExpr(Sdf.cappedCylinder(radius, halfHeight).getOrThrow())
        @JvmStatic fun cappedCone(halfHeight: Float, bottomRadius: Float, topRadius: Float) =
            SdfExpr(Sdf.cappedCone(halfHeight, bottomRadius, topRadius).getOrThrow())
        @JvmStatic fun plane(normalX: Float, normalY: Float, normalZ: Float, offset: Float) =
            SdfExpr(Sdf.plane(normalX, normalY, normalZ, offset).getOrThrow())
        @JvmStatic fun octahedron(size: Float) =
            SdfExpr(Sdf.octahedron(size).getOrThrow())
        @JvmStatic fun hexPrism(radius: Float, halfHeight: Float) =
            SdfExpr(Sdf.hexPrism(radius, halfHeight).getOrThrow())
        @JvmStatic fun superPrism(halfX: Float, halfY: Float, halfZ: Float, exponent: Float) =
            SdfExpr(Sdf.superPrism(halfX, halfY, halfZ, exponent).getOrThrow())
        @JvmStatic fun cells(
            frequency: Float,
            seed: Int,
            jitter: Float,
            mode: SdfCellMode,
            threshold: Float,
        ) = SdfExpr(Sdf.cells(frequency, seed, jitter, mode, threshold).getOrThrow())
        @JvmStatic fun fromJson(json: String) =
            SdfExpr(Sdf.fromJsonString(json).getOrThrow())
    }

    fun unionWith(other: SdfExpr) = SdfExpr(raw.unionWith(other.raw))
    fun intersectionWith(other: SdfExpr) = SdfExpr(raw.intersectionWith(other.raw))
    fun subtract(other: SdfExpr) = SdfExpr(raw.subtract(other.raw))
    fun smoothUnion(other: SdfExpr, radius: Float) =
        SdfExpr(raw.smoothUnion(other.raw, radius).getOrThrow())
    fun smoothSubtract(other: SdfExpr, radius: Float) =
        SdfExpr(raw.smoothSubtract(other.raw, radius).getOrThrow())
    fun smoothIntersection(other: SdfExpr, radius: Float) =
        SdfExpr(raw.smoothIntersection(other.raw, radius).getOrThrow())
    fun rounded(radius: Float) = SdfExpr(raw.rounded(radius).getOrThrow())
    fun shell(thickness: Float) = SdfExpr(raw.shell(thickness).getOrThrow())

    fun translate(x: Float, y: Float, z: Float) =
        SdfExpr(raw.translate(x, y, z).getOrThrow())
    fun rotate(xDegrees: Float, yDegrees: Float, zDegrees: Float) =
        SdfExpr(raw.rotate(xDegrees, yDegrees, zDegrees).getOrThrow())
    fun scale(factor: Float) = SdfExpr(raw.scale(factor).getOrThrow())
    fun mirror(axis: SdfAxis) = SdfExpr(raw.mirror(axis))
    fun repeatInfinite(spacingX: Float, spacingY: Float, spacingZ: Float) =
        SdfExpr(raw.repeatInfinite(spacingX, spacingY, spacingZ).getOrThrow())
    fun repeatCounted(
        spacingX: Float,
        spacingY: Float,
        spacingZ: Float,
        countX: Int,
        countY: Int,
        countZ: Int,
    ): SdfExpr {
        require(countX >= 0 && countY >= 0 && countZ >= 0) {
            "repeat counts must be non-negative"
        }
        return SdfExpr(raw.repeatCounted(
            spacingX, spacingY, spacingZ,
            countX.toUInt(), countY.toUInt(), countZ.toUInt(),
        ).getOrThrow())
    }
    @JvmOverloads
    fun displace(amplitude: Float, frequency: Float, seed: Int, octaves: Int = 3): SdfExpr {
        require(octaves in 1..8) { "octaves must be between 1 and 8" }
        return SdfExpr(raw.displace(amplitude, frequency, seed, octaves.toUInt()).getOrThrow())
    }
    fun warp(amplitude: Float, frequency: Float, seed: Int) =
        SdfExpr(raw.warp(amplitude, frequency, seed).getOrThrow())

    fun evalAt(x: Float, y: Float, z: Float): Float = raw.evalAt(x, y, z)
    @JvmOverloads
    fun normal(x: Float, y: Float, z: Float, epsilon: Float = 0.01f): SdfNormal =
        raw.normal(x, y, z, epsilon).getOrThrow()
    fun bounds(): SdfBounds = raw.bounds().getOrThrow()
    fun toShape(): Shape = raw.toShape().getOrThrow()
    fun toShapeBounded(
        minX: Int, minY: Int, minZ: Int,
        maxX: Int, maxY: Int, maxZ: Int,
    ): Shape = raw.toShapeBounded(minX, minY, minZ, maxX, maxY, maxZ).getOrThrow()
    fun toJson(): String = raw.toJson().getOrThrow()
}
