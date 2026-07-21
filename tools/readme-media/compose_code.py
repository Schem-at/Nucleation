#!/usr/bin/env python3
"""Composite a code panel beside an animation, highlighting the live line.

Reads a directory produced by a `readme_*` example (transparent `f####.png`
frames plus `timing.json`) and writes a side-by-side GIF: syntax-lit code on
the left, the build on the right, the active line lifting as its block appears.

    python3 tools/readme-media/compose_code.py <dir> [out.gif]

Needs `rsvg-convert` (SVG raster) and `ffmpeg` (GIF assembly) on PATH. No
Python dependencies.
"""
import base64
import html
import json
import os
import re
import subprocess
import sys
import tempfile

# A dark palette that reads on GitHub light or dark.
BG = "#0d1117"
PANEL = "#161b22"
DIM = "#8b949e"
FG = "#c9d1d9"
STRING = "#7ee787"
CALL = "#d2a8ff"
HILITE_BG = "#1f6feb33"
HILITE_BAR = "#1f6feb"

LINE_H = 32
FONT = 17
PAD = 24


def syntax_spans(line):
    """Crude tokeniser: colour strings and the method after a dot. Returns a
    list of (text, colour) with everything else in the foreground colour."""
    spans, i = [], 0
    for m in re.finditer(r'"[^"]*"', line):
        if m.start() > i:
            spans.append((line[i:m.start()], None))
        spans.append((m.group(), STRING))
        i = m.end()
    if i < len(line):
        spans.append((line[i:], None))
    # Recolour a `.method(` call in the non-string spans.
    out = []
    for text, col in spans:
        if col is None:
            for mm in re.finditer(r'(\.)([a-z_]+)(?=\()', text):
                pass
        out.append((text, col))
    return out


def esc(s):
    return html.escape(s, quote=True)


def build_svg(spec, frame_png, active):
    code = spec["code"]
    aw, ah = spec["anim_w"], spec["anim_h"]
    panel_w = 480
    width = panel_w + aw + PAD
    height = max(ah + 2 * PAD, len(code) * LINE_H + 3 * PAD)
    b64 = base64.b64encode(open(frame_png, "rb").read()).decode()

    rows = []
    y0 = PAD + 28
    for i, line in enumerate(code):
        y = y0 + i * LINE_H
        if i == active:
            rows.append(
                f'<rect x="{PAD-6}" y="{y-FONT-4}" width="{panel_w-2*PAD+12}" '
                f'height="{LINE_H-4}" rx="5" fill="{HILITE_BG}"/>'
            )
            rows.append(
                f'<rect x="{PAD-6}" y="{y-FONT-4}" width="3" height="{LINE_H-4}" fill="{HILITE_BAR}"/>'
            )
        # tokens
        x = PAD + 6
        for text, col in syntax_spans(line):
            fill = col or (FG if i == active else DIM)
            weight = "600" if i == active else "400"
            # monospace advance ~ FONT*0.6 per char
            rows.append(
                f'<text x="{x}" y="{y}" font-family="ui-monospace,Menlo,monospace" '
                f'font-size="{FONT}" font-weight="{weight}" fill="{fill}" '
                f'xml:space="preserve">{esc(text)}</text>'
            )
            x += len(text) * FONT * 0.6
    title = esc(spec.get("title", ""))
    return f'''<svg xmlns="http://www.w3.org/2000/svg" width="{int(width)}" height="{int(height)}">
<rect width="{int(width)}" height="{int(height)}" fill="{BG}"/>
<rect x="{PAD-14}" y="{PAD-8}" width="{panel_w-PAD}" height="{height-2*PAD+16}" rx="10" fill="{PANEL}"/>
<text x="{PAD}" y="{PAD}" font-family="ui-sans-serif,system-ui,sans-serif" font-size="13" fill="{DIM}">{title}</text>
{''.join(rows)}
<image x="{panel_w}" y="{(height-ah)//2}" width="{aw}" height="{ah}" href="data:image/png;base64,{b64}"/>
</svg>'''


def main():
    if len(sys.argv) < 2:
        print(__doc__)
        sys.exit(2)
    d = sys.argv[1]
    out = sys.argv[2] if len(sys.argv) > 2 else os.path.join(d, "composed.gif")
    spec = json.load(open(os.path.join(d, "timing.json")))
    active = spec["active"]
    frames = sorted(f for f in os.listdir(d) if re.match(r"f\d+\.png$", f))
    assert len(frames) == len(active), f"{len(frames)} frames vs {len(active)} timing entries"

    tmp = tempfile.mkdtemp(prefix="compose-")
    for i, fr in enumerate(frames):
        svg = build_svg(spec, os.path.join(d, fr), active[i])
        sp = os.path.join(tmp, f"c{i:04d}.svg")
        open(sp, "w").write(svg)
        subprocess.run(["rsvg-convert", sp, "-o", os.path.join(tmp, f"c{i:04d}.png")], check=True)

    fps = 20
    subprocess.run([
        "ffmpeg", "-y", "-v", "error", "-framerate", str(fps),
        "-i", os.path.join(tmp, "c%04d.png"),
        "-vf", "split[a][b];[a]palettegen=max_colors=128[p];[b][p]paletteuse",
        "-loop", "0", out,
    ], check=True)
    kb = os.path.getsize(out) // 1024
    print(f"wrote {out} ({len(frames)} frames, {kb} KB)")


if __name__ == "__main__":
    main()
