<?php
namespace Stencil;

final class FieldSample {
    public function __construct(public float $distance, public int $material_id, public int $blend_material_id, public float $blend_weight) {}

    public static function fromFFI($s): self {
        return new self($s->distance, $s->material_id, $s->blend_material_id, $s->blend_weight);
    }

    public function toFFI() {
        $s = Lib::ffi()->new('FieldSample');
        $s->distance = $this->distance;
        $s->material_id = $this->material_id;
        $s->blend_material_id = $this->blend_material_id;
        $s->blend_weight = $this->blend_weight;
        return $s;
    }
}
