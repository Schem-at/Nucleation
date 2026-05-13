package com.github.schemat.nucleation;

/** Width / height / length of a {@link Schematic}'s bounding box. */
public record Dimensions(int width, int height, int length) {
    public int volume() { return width * height * length; }
}
