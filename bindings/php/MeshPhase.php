<?php
namespace Stencil;

final class MeshPhase {
    const BuildingAtlas = 0;
    const MeshingChunks = 1;
    const Complete = 2;
    const Failed = 3;

    public static function name(int $value): string {
        return match ($value) {
            self::BuildingAtlas => 'BuildingAtlas',
            self::MeshingChunks => 'MeshingChunks',
            self::Complete => 'Complete',
            self::Failed => 'Failed',
            default => "Unknown({$value})",
        };
    }
}
