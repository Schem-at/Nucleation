<?php
namespace Stencil;

final class FieldNoiseBasis {
    const ValueNoise = 0;
    const Perlin = 1;

    public static function name(int $value): string {
        return match ($value) {
            self::ValueNoise => 'ValueNoise',
            self::Perlin => 'Perlin',
            default => "Unknown({$value})",
        };
    }
}
