<?php
namespace Stencil;

final class GeneratedChunkOverlayMode {
    const Replace = 0;
    const KeepExisting = 1;

    public static function name(int $value): string {
        return match ($value) {
            self::Replace => 'Replace',
            self::KeepExisting => 'KeepExisting',
            default => "Unknown({$value})",
        };
    }
}
