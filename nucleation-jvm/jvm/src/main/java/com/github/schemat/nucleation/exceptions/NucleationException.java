package com.github.schemat.nucleation.exceptions;

/**
 * Base class for all exceptions raised by Nucleation JNI bindings.
 *
 * <p>Subclasses are thrown when the Rust side has enough context to
 * discriminate; otherwise {@code NucleationException} itself is raised.
 */
public class NucleationException extends RuntimeException {
    public NucleationException(String message) { super(message); }
    public NucleationException(String message, Throwable cause) { super(message, cause); }
}
