package com.github.schemat.nucleation;

import java.util.Objects;

/**
 * Configuration for item-model generation ({@link Schematic#toItemModel}).
 *
 * <p>Plain value object — no native resources, no need to close. Mirrors the
 * native {@code ItemModelConfig} with the same defaults.
 */
public final class ItemModelConfig {

    static final int SCALE_AUTO = 0;
    static final int SCALE_UNIFORM = 1;
    static final int SCALE_NON_UNIFORM = 2;

    private final String modelName;
    private String namespace = "nucleation";
    private boolean center = true;
    private int textureResolution = 16;
    private String item = "paper";
    private String customModelData = "1";
    private int scaleMode = SCALE_AUTO;
    private float sx = 1f, sy = 1f, sz = 1f;

    public ItemModelConfig(String modelName) {
        this.modelName = Objects.requireNonNull(modelName, "modelName");
    }

    public ItemModelConfig namespace(String namespace) {
        this.namespace = Objects.requireNonNull(namespace, "namespace");
        return this;
    }

    public ItemModelConfig center(boolean center) {
        this.center = center;
        return this;
    }

    public ItemModelConfig textureResolution(int resolution) {
        this.textureResolution = resolution;
        return this;
    }

    /** Minecraft item to bind the model to (default: {@code paper}). */
    public ItemModelConfig item(String item) {
        this.item = Objects.requireNonNull(item, "item");
        return this;
    }

    public ItemModelConfig customModelData(String customModelData) {
        this.customModelData = Objects.requireNonNull(customModelData, "customModelData");
        return this;
    }

    /** Uniform scale factor on all axes (clamped to >= 1 natively). */
    public ItemModelConfig scale(float scale) {
        this.scaleMode = SCALE_UNIFORM;
        this.sx = scale;
        return this;
    }

    public ItemModelConfig scaleXyz(float sx, float sy, float sz) {
        this.scaleMode = SCALE_NON_UNIFORM;
        this.sx = sx; this.sy = sy; this.sz = sz;
        return this;
    }

    /** Automatically fit within Minecraft's 48-unit model bounds (default). */
    public ItemModelConfig scaleAuto() {
        this.scaleMode = SCALE_AUTO;
        return this;
    }

    String modelName() { return modelName; }
    String namespaceValue() { return namespace; }
    boolean centerValue() { return center; }
    int textureResolutionValue() { return textureResolution; }
    String itemValue() { return item; }
    String customModelDataValue() { return customModelData; }
    int scaleModeValue() { return scaleMode; }
    float sxValue() { return sx; }
    float syValue() { return sy; }
    float szValue() { return sz; }
}
