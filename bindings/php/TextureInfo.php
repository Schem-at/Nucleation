<?php
namespace Stencil;

final class TextureInfo {
    public function __construct(public int $width, public int $height, public bool $animated, public int $frame_count) {}

    public static function fromFFI($s): self {
        return new self($s->width, $s->height, $s->animated, $s->frame_count);
    }

    public function toFFI() {
        $s = Lib::ffi()->new('TextureInfo');
        $s->width = $this->width;
        $s->height = $this->height;
        $s->animated = $this->animated;
        $s->frame_count = $this->frame_count;
        return $s;
    }
}
