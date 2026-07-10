<?php
namespace Stencil;

final class MeshBounds {
    public function __construct(public float $min_x, public float $min_y, public float $min_z, public float $max_x, public float $max_y, public float $max_z) {}

    public static function fromFFI($s): self {
        return new self($s->min_x, $s->min_y, $s->min_z, $s->max_x, $s->max_y, $s->max_z);
    }

    public function toFFI() {
        $s = Lib::ffi()->new('MeshBounds');
        $s->min_x = $this->min_x;
        $s->min_y = $this->min_y;
        $s->min_z = $this->min_z;
        $s->max_x = $this->max_x;
        $s->max_y = $this->max_y;
        $s->max_z = $this->max_z;
        return $s;
    }
}
