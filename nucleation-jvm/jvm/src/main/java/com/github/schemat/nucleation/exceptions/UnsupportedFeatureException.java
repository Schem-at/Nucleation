package com.github.schemat.nucleation.exceptions;

/**
 * Thrown when a feature-gated API (e.g. simulation) is invoked on a cdylib
 * that was not built with the corresponding feature.
 */
public class UnsupportedFeatureException extends NucleationException {
    public UnsupportedFeatureException(String message) { super(message); }
}
