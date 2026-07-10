<?php
namespace Stencil;

final class NucleationError {
    const NullArgument = 0;
    const InvalidArgument = 1;
    const Parse = 2;
    const Serialize = 3;
    const Io = 4;
    const Lock = 5;
    const Store = 6;
    const Mesh = 7;
    const Render = 8;
    const Simulation = 9;
    const AlreadyConsumed = 10;
    const NotFound = 11;

    public static function name(int $value): string {
        return match ($value) {
            self::NullArgument => 'NullArgument',
            self::InvalidArgument => 'InvalidArgument',
            self::Parse => 'Parse',
            self::Serialize => 'Serialize',
            self::Io => 'Io',
            self::Lock => 'Lock',
            self::Store => 'Store',
            self::Mesh => 'Mesh',
            self::Render => 'Render',
            self::Simulation => 'Simulation',
            self::AlreadyConsumed => 'AlreadyConsumed',
            self::NotFound => 'NotFound',
            default => "Unknown({$value})",
        };
    }
}
