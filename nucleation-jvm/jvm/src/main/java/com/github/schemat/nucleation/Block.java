package com.github.schemat.nucleation;

import java.util.Map;

/**
 * Immutable record describing a single block in a {@link Schematic}.
 *
 * <p>The {@code properties} map is always non-null; for blocks with no
 * properties it is an empty map. Decoded once from the bulk
 * {@code getAllBlocks} call so per-block iteration costs no JNI calls.
 */
public record Block(int x, int y, int z, String name, Map<String, String> properties) {
}
