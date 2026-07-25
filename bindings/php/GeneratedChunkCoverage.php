<?php
namespace Stencil;

final class GeneratedChunkCoverage {
    const Complete = 0;
    const Partial = 1;
    const Outside = 2;

    public static function name(int $value): string {
        return match ($value) {
            self::Complete => 'Complete',
            self::Partial => 'Partial',
            self::Outside => 'Outside',
            default => "Unknown({$value})",
        };
    }
}
