<?php
namespace Stencil;

final class FieldRange {
    public function __construct(public float $min, public float $max) {}

    public static function fromFFI($s): self {
        return new self($s->min, $s->max);
    }

    public function toFFI() {
        $s = Lib::ffi()->new('FieldRange');
        $s->min = $this->min;
        $s->max = $this->max;
        return $s;
    }
}
