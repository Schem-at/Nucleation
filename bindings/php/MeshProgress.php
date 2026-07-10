<?php
namespace Stencil;

final class MeshProgress {
    public function __construct(public int $phase, public int $current, public int $total) {}

    public static function fromFFI($s): self {
        return new self($s->phase, $s->current, $s->total);
    }

    public function toFFI() {
        $s = Lib::ffi()->new('MeshProgress');
        $s->phase = $this->phase;
        $s->current = $this->current;
        $s->total = $this->total;
        return $s;
    }
}
