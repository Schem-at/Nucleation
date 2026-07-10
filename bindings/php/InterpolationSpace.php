<?php
namespace Stencil;

final class InterpolationSpace {
    const Rgb = 0;
    const Oklab = 1;

    public static function name(int $value): string {
        return match ($value) {
            self::Rgb => 'Rgb',
            self::Oklab => 'Oklab',
            default => "Unknown({$value})",
        };
    }
}
