<?php
namespace Stencil;

final class FieldProgramDistanceKind {
    const Exact = 0;
    const LowerBound = 1;
    const Estimate = 2;
    const Implicit = 3;

    public static function name(int $value): string {
        return match ($value) {
            self::Exact => 'Exact',
            self::LowerBound => 'LowerBound',
            self::Estimate => 'Estimate',
            self::Implicit => 'Implicit',
            default => "Unknown({$value})",
        };
    }
}
