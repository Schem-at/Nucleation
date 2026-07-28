<?php
namespace Stencil;

final class TickSettleMode {
    const Placement = 0;
    const Quiet = 1;
    const InWorld = 2;

    public static function name(int $value): string {
        return match ($value) {
            self::Placement => 'Placement',
            self::Quiet => 'Quiet',
            self::InWorld => 'InWorld',
            default => "Unknown({$value})",
        };
    }
}
