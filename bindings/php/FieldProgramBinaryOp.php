<?php
namespace Stencil;

final class FieldProgramBinaryOp {
    const Add = 0;
    const Sub = 1;
    const Mul = 2;
    const Div = 3;
    const Min = 4;
    const Max = 5;
    const Pow = 6;
    const Atan2 = 7;
    const Lt = 8;
    const Le = 9;
    const Gt = 10;
    const Ge = 11;
    const Eq = 12;
    const Dot = 13;
    const Cross = 14;
    const Scale = 15;

    public static function name(int $value): string {
        return match ($value) {
            self::Add => 'Add',
            self::Sub => 'Sub',
            self::Mul => 'Mul',
            self::Div => 'Div',
            self::Min => 'Min',
            self::Max => 'Max',
            self::Pow => 'Pow',
            self::Atan2 => 'Atan2',
            self::Lt => 'Lt',
            self::Le => 'Le',
            self::Gt => 'Gt',
            self::Ge => 'Ge',
            self::Eq => 'Eq',
            self::Dot => 'Dot',
            self::Cross => 'Cross',
            self::Scale => 'Scale',
            default => "Unknown({$value})",
        };
    }
}
