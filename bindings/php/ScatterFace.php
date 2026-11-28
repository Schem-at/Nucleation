<?php
namespace Stencil;

final class ScatterFace {
    const Top = 0;
    const Bottom = 1;
    const North = 2;
    const South = 3;
    const East = 4;
    const West = 5;

    public static function name(int $value): string {
        return match ($value) {
            self::Top => 'Top',
            self::Bottom => 'Bottom',
            self::North => 'North',
            self::South => 'South',
            self::East => 'East',
            self::West => 'West',
            default => "Unknown({$value})",
        };
    }
}
