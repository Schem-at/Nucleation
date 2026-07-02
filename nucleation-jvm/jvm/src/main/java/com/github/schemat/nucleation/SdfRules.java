package com.github.schemat.nucleation;

import java.util.ArrayList;
import java.util.List;
import java.util.Locale;
import java.util.Objects;

/**
 * Fluent builder for the material-rules JSON consumed by
 * {@link Schematic#fromSdf(String, String)}.
 *
 * <p>Fill rules are evaluated in insertion order — first match wins — so
 * list specific rules first and end with {@link #fill(String)} as the
 * default. Surface rules scatter decoration blocks on air above surfaces.
 *
 * <pre>{@code
 * String rules = new SdfRules()
 *     .fillDepth(0, 0, "minecraft:grass_block")
 *     .fillDepth(1, 3, "minecraft:dirt")
 *     .fillBelowY(40, "minecraft:deepslate")
 *     .fill("minecraft:stone")
 *     .surface(0.15, 31, "minecraft:grass_block", "minecraft:short_grass", "minecraft:fern")
 *     .toJson();
 * }</pre>
 */
public final class SdfRules {

    private final List<String> fill = new ArrayList<>();
    private final List<String> surface = new ArrayList<>();

    private static String esc(String s) {
        StringBuilder b = new StringBuilder(s.length() + 2);
        for (int i = 0; i < s.length(); i++) {
            char c = s.charAt(i);
            switch (c) {
                case '"' -> b.append("\\\"");
                case '\\' -> b.append("\\\\");
                default -> {
                    if (c < 0x20) b.append(String.format(Locale.ROOT, "\\u%04x", (int) c));
                    else b.append(c);
                }
            }
        }
        return b.toString();
    }

    /** Rule matching a depth-below-surface band (0 = the surface block itself). */
    public SdfRules fillDepth(int minDepth, int maxDepth, String block) {
        fill.add("{\"when\":{\"depthBelowSurface\":{\"min\":" + minDepth + ",\"max\":" + maxDepth
                + "}},\"block\":\"" + esc(block) + "\"}");
        return this;
    }

    /** Rule matching absolute world Y at or below the given value. */
    public SdfRules fillBelowY(int maxY, String block) {
        fill.add("{\"when\":{\"yRange\":{\"max\":" + maxY + "}},\"block\":\"" + esc(block) + "\"}");
        return this;
    }

    /** Rule matching absolute world Y at or above the given value. */
    public SdfRules fillAboveY(int minY, String block) {
        fill.add("{\"when\":{\"yRange\":{\"min\":" + minY + "}},\"block\":\"" + esc(block) + "\"}");
        return this;
    }

    /** Rule gated by 2D FBM noise over (x, z): matches where noise > threshold. */
    public SdfRules fillNoise(double threshold, double frequency, int seed, String block) {
        fill.add("{\"when\":{\"noise\":{\"threshold\":" + (float) threshold
                + ",\"frequency\":" + (float) frequency + ",\"seed\":" + seed
                + "}},\"block\":\"" + esc(block) + "\"}");
        return this;
    }

    /** Unconditional rule — the default material. Add it last. */
    public SdfRules fill(String block) {
        fill.add("{\"block\":\"" + esc(block) + "\"}");
        return this;
    }

    /**
     * Scatter decorations on air above surface blocks whose fill block starts
     * with {@code onBlock} (pass {@code null} to decorate any surface).
     */
    public SdfRules surface(double density, int seed, String onBlock, String... blocks) {
        Objects.requireNonNull(blocks, "blocks");
        StringBuilder b = new StringBuilder("{\"density\":").append((float) density)
                .append(",\"seed\":").append(seed);
        if (onBlock != null) b.append(",\"on\":\"").append(esc(onBlock)).append('"');
        b.append(",\"blocks\":[");
        for (int i = 0; i < blocks.length; i++) {
            if (i > 0) b.append(',');
            b.append('"').append(esc(blocks[i])).append('"');
        }
        b.append("]}");
        surface.add(b.toString());
        return this;
    }

    public String toJson() {
        return "{\"fill\":[" + String.join(",", fill) + "],\"surface\":["
                + String.join(",", surface) + "]}";
    }

    @Override public String toString() { return toJson(); }
}
