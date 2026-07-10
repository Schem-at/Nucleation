<?php
namespace Stencil;

/// Thrown for every fallible generated method. $errorType/$errorName tell you which
/// generated enum this came from (e.g. "SchematicError", "OutOfBounds"); $errorValue is
/// the raw int discriminant, for callers that want to switch on it without string matching.
final class DiplomatError extends \Exception {
    public function __construct(
        public readonly string $errorType,
        public readonly int $errorValue,
        public readonly string $errorName,
    ) {
        parent::__construct("{$errorType}::{$errorName}");
    }
}
