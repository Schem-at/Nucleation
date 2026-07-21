#!/usr/bin/env python3
"""Compose code and renderer frames into a polished, code-synchronised GIF.

Reads transparent ``f####.png`` frames and ``timing.json`` produced by a
``readme_*`` example.  The layout and tokenizer are intentionally reusable and
have no Python package dependencies; only rsvg-convert and ffmpeg are needed.

    python3 tools/readme-media/compose_code.py <dir> [out.gif]
"""
from __future__ import annotations

import base64
from dataclasses import dataclass
import html
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile

BG = "#0b0f14"
PANEL = "#111820"
PANEL_TOP = "#17212b"
BORDER = "#25313d"
MUTED = "#71808f"
FG = "#d8dee6"
STRING = "#8bd49c"
CALL = "#82c7ff"
NUMBER = "#d6a5ff"
KEYWORD = "#ff9eaa"
ACCENT = "#f0b35b"
HILITE_BG = "#2a2117"
SCENE_GLOW = "#17242a"

FONT = 16
LINE_H = 30
PAD = 24
GUTTER_W = 42
EDITOR_TOP = 46
CHAR_W = FONT * 0.61
TOKEN_RE = re.compile(
    r'("(?:\\.|[^"\\])*")'
    r'|(?<![\w.])(-?\d+(?:\.\d+)?)'
    r'|\b(for|in|if|else|from|import|as|True|False|None)\b'
    r'|(?<=\.)([A-Za-z_][A-Za-z0-9_]*)(?=\()'
)


@dataclass(frozen=True)
class Layout:
    width: int
    height: int
    panel_w: int
    scene_w: int
    scene_x: int
    scene_y: int


def syntax_spans(line: str) -> list[tuple[str, str | None]]:
    """Split one Python line into display spans and semantic token kinds."""
    spans: list[tuple[str, str | None]] = []
    cursor = 0
    for match in TOKEN_RE.finditer(line):
        if match.start() > cursor:
            spans.append((line[cursor : match.start()], None))
        kind = "string" if match.group(1) else "number" if match.group(2) else "keyword" if match.group(3) else "call"
        spans.append((match.group(0), kind))
        cursor = match.end()
    if cursor < len(line):
        spans.append((line[cursor:], None))
    return spans


def calculate_layout(code: list[str], anim_w: int, anim_h: int) -> Layout:
    """Size the editor from its content while reserving the full render canvas."""
    longest = max((len(line) for line in code), default=0)
    content_w = int(longest * CHAR_W + GUTTER_W + PAD * 2)
    panel_w = max(590, min(720, content_w))
    scene_w = anim_w
    width = panel_w + scene_w + PAD
    code_h = EDITOR_TOP + len(code) * LINE_H + PAD
    height = max(anim_h + PAD * 2, code_h, 468)
    return Layout(
        width=width,
        height=height,
        panel_w=panel_w,
        scene_w=scene_w,
        scene_x=panel_w + PAD,
        scene_y=(height - anim_h) // 2,
    )


def _frame_data_uri(frame: str) -> str:
    if frame.startswith("data:image/"):
        return frame
    with open(frame, "rb") as handle:
        return "data:image/png;base64," + base64.b64encode(handle.read()).decode()


def _escape(value: str) -> str:
    return html.escape(value, quote=True)


def build_svg(spec: dict, frame_png: str, active: int) -> str:
    """Create one complete compositor frame as SVG."""
    code = spec["code"]
    aw, ah = int(spec["anim_w"]), int(spec["anim_h"])
    layout = calculate_layout(code, aw, ah)
    data_uri = _frame_data_uri(frame_png)
    filename = _escape(spec.get("filename", "example.py"))
    title = _escape(spec.get("title", "Live build"))

    rows: list[str] = []
    baseline = PAD + EDITOR_TOP + FONT
    token_colours = {
        None: FG,
        "string": STRING,
        "call": CALL,
        "number": NUMBER,
        "keyword": KEYWORD,
    }
    for index, line in enumerate(code):
        y = baseline + index * LINE_H
        if index == active:
            rows.append(
                f'<rect data-role="active-line" x="12" y="{y-FONT-7}" '
                f'width="{layout.panel_w-24}" height="{LINE_H}" rx="6" fill="{HILITE_BG}"/>'
            )
            rows.append(
                f'<rect x="12" y="{y-FONT-7}" width="3" height="{LINE_H}" rx="1.5" fill="{ACCENT}"/>'
            )
        number_fill = ACCENT if index == active else MUTED
        rows.append(
            f'<text data-role="line-number" x="{PAD+GUTTER_W-12}" y="{y}" '
            f'text-anchor="end" font-family="ui-monospace,SFMono-Regular,Menlo,monospace" '
            f'font-size="13" fill="{number_fill}">{index+1}</text>'
        )
        x = PAD + GUTTER_W
        for text, kind in syntax_spans(line):
            fill = token_colours[kind]
            opacity = "1" if index == active else "0.76"
            rows.append(
                f'<text x="{x:.1f}" y="{y}" font-family="ui-monospace,SFMono-Regular,Menlo,monospace" '
                f'font-size="{FONT}" font-weight="500" fill="{fill}" opacity="{opacity}" '
                f'xml:space="preserve">{_escape(text)}</text>'
            )
            x += len(text) * CHAR_W

    return f'''<svg xmlns="http://www.w3.org/2000/svg" width="{layout.width}" height="{layout.height}">
<defs>
  <radialGradient id="sceneGlow" cx="50%" cy="53%" r="55%">
    <stop offset="0" stop-color="{SCENE_GLOW}"/><stop offset="1" stop-color="{BG}"/>
  </radialGradient>
</defs>
<rect width="{layout.width}" height="{layout.height}" fill="{BG}"/>
<rect x="0.5" y="0.5" width="{layout.panel_w-1}" height="{layout.height-1}" rx="12" fill="{PANEL}" stroke="{BORDER}"/>
<path d="M12 0.5 H{layout.panel_w-12} Q{layout.panel_w-0.5} 0.5 {layout.panel_w-0.5} 12 V{EDITOR_TOP} H0.5 V12 Q0.5 0.5 12 0.5" fill="{PANEL_TOP}"/>
<circle data-role="window-dot" cx="20" cy="23" r="4" fill="#ef6a67"/>
<circle data-role="window-dot" cx="34" cy="23" r="4" fill="#e6b450"/>
<circle data-role="window-dot" cx="48" cy="23" r="4" fill="#57ab5a"/>
<text x="{layout.panel_w//2}" y="28" text-anchor="middle" font-family="ui-monospace,SFMono-Regular,Menlo,monospace" font-size="13" fill="{MUTED}">{filename}</text>
<line x1="0" y1="{EDITOR_TOP}" x2="{layout.panel_w}" y2="{EDITOR_TOP}" stroke="{BORDER}"/>
{''.join(rows)}
<line data-role="scene-divider" x1="{layout.panel_w+PAD//2}" y1="{PAD}" x2="{layout.panel_w+PAD//2}" y2="{layout.height-PAD}" stroke="{BORDER}"/>
<rect x="{layout.scene_x}" y="{layout.scene_y}" width="{aw}" height="{ah}" rx="12" fill="url(#sceneGlow)"/>
<text x="{layout.scene_x+aw-16}" y="{layout.scene_y+23}" text-anchor="end" font-family="ui-sans-serif,-apple-system,sans-serif" font-size="12" fill="{ACCENT}">{title}</text>
<image x="{layout.scene_x}" y="{layout.scene_y}" width="{aw}" height="{ah}" href="{data_uri}"/>
</svg>'''


def _require(command: str) -> None:
    if shutil.which(command) is None:
        raise SystemExit(f"missing required command: {command}")


def main() -> None:
    if len(sys.argv) < 2:
        print(__doc__)
        raise SystemExit(2)
    directory = sys.argv[1]
    out = sys.argv[2] if len(sys.argv) > 2 else os.path.join(directory, "composed.gif")
    spec = json.load(open(os.path.join(directory, "timing.json"), encoding="utf-8"))
    active = spec["active"]
    frames = sorted(f for f in os.listdir(directory) if re.fullmatch(r"f\d+\.png", f))
    if len(frames) != len(active):
        raise ValueError(f"{len(frames)} frames vs {len(active)} timing entries")
    _require("rsvg-convert")
    _require("ffmpeg")

    with tempfile.TemporaryDirectory(prefix="compose-") as tmp:
        for index, frame in enumerate(frames):
            svg = build_svg(spec, os.path.join(directory, frame), active[index])
            svg_path = os.path.join(tmp, f"c{index:04d}.svg")
            with open(svg_path, "w", encoding="utf-8") as handle:
                handle.write(svg)
            subprocess.run(
                ["rsvg-convert", svg_path, "-o", os.path.join(tmp, f"c{index:04d}.png")],
                check=True,
            )

        subprocess.run(
            [
                "ffmpeg", "-y", "-v", "error", "-framerate", "20",
                "-i", os.path.join(tmp, "c%04d.png"),
                "-vf", "split[a][b];[a]palettegen=max_colors=160[p];[b][p]paletteuse",
                "-loop", "0", out,
            ],
            check=True,
        )
    print(f"wrote {out} ({len(frames)} frames, {os.path.getsize(out) // 1024} KB)")


if __name__ == "__main__":
    main()
