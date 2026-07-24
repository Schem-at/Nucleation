<?php
namespace Stencil;

final class SdfCellMode {
    const F1 = 0;
    const F2 = 1;
    const F2MinusF1 = 2;
    const CellValue = 3;

    public static function name(int $value): string {
        return match ($value) {
            self::F1 => 'F1',
            self::F2 => 'F2',
            self::F2MinusF1 => 'F2MinusF1',
            self::CellValue => 'CellValue',
            default => "Unknown({$value})",
        };
    }
}
