<?php
namespace Stencil;

final class SdfAxis {
    const X = 0;
    const Y = 1;
    const Z = 2;

    public static function name(int $value): string {
        return match ($value) {
            self::X => 'X',
            self::Y => 'Y',
            self::Z => 'Z',
            default => "Unknown({$value})",
        };
    }
}
