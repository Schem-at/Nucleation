<?php
namespace Stencil;

final class ItemScale {
    public function __construct(public float $x, public float $y, public float $z) {}

    public static function fromFFI($s): self {
        return new self($s->x, $s->y, $s->z);
    }

    public function toFFI() {
        $s = Lib::ffi()->new('ItemScale');
        $s->x = $this->x;
        $s->y = $this->y;
        $s->z = $this->z;
        return $s;
    }
}
