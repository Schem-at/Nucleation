<?php
namespace Stencil;

final class FieldProgramUnaryOp {
    const Neg = 0;
    const Abs = 1;
    const Sqrt = 2;
    const Log = 3;
    const Sin = 4;
    const Cos = 5;
    const Acos = 6;
    const VecX = 7;
    const VecY = 8;
    const VecZ = 9;
    const Length = 10;
    const Normalize = 11;

    public static function name(int $value): string {
        return match ($value) {
            self::Neg => 'Neg',
            self::Abs => 'Abs',
            self::Sqrt => 'Sqrt',
            self::Log => 'Log',
            self::Sin => 'Sin',
            self::Cos => 'Cos',
            self::Acos => 'Acos',
            self::VecX => 'VecX',
            self::VecY => 'VecY',
            self::VecZ => 'VecZ',
            self::Length => 'Length',
            self::Normalize => 'Normalize',
            default => "Unknown({$value})",
        };
    }
}
