<?php
namespace Stencil;

final class BlockPos {
    public function __construct(public int $x, public int $y, public int $z) {}

    public static function fromFFI($s): self {
        return new self($s->x, $s->y, $s->z);
    }

    public function toFFI() {
        $s = Lib::ffi()->new('BlockPos');
        $s->x = $this->x;
        $s->y = $this->y;
        $s->z = $this->z;
        return $s;
    }
}
