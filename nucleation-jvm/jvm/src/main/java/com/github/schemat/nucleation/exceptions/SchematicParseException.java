package com.github.schemat.nucleation.exceptions;

/** Thrown when a schematic blob cannot be parsed in the requested format. */
public class SchematicParseException extends NucleationException {
    public SchematicParseException(String message) { super(message); }
}
