<?php
namespace Stencil;

final class FieldChannel {
    const WorldX = 0;
    const WorldY = 1;
    const WorldZ = 2;
    const Distance = 3;
    const DepthBelowSurface = 4;
    const Slope = 5;
    const NormalY = 6;
    const Curvature = 7;

    public static function name(int $value): string {
        return match ($value) {
            self::WorldX => 'WorldX',
            self::WorldY => 'WorldY',
            self::WorldZ => 'WorldZ',
            self::Distance => 'Distance',
            self::DepthBelowSurface => 'DepthBelowSurface',
            self::Slope => 'Slope',
            self::NormalY => 'NormalY',
            self::Curvature => 'Curvature',
            default => "Unknown({$value})",
        };
    }
}
