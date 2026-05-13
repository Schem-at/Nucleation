package com.github.schemat.nucleation.exceptions;

/** Thrown when a block name or property combination is malformed. */
public class InvalidBlockStateException extends NucleationException {
    public InvalidBlockStateException(String message) { super(message); }
}
