<?php
namespace Stencil;

final class FieldProgramValueType {
    const Scalar = 0;
    const Vec3 = 1;
    const Bool = 2;

    public static function name(int $value): string {
        return match ($value) {
            self::Scalar => 'Scalar',
            self::Vec3 => 'Vec3',
            self::Bool => 'Bool',
            default => "Unknown({$value})",
        };
    }
}
