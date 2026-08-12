#!/usr/bin/env python
"""Build `index.html` (self-contained) and `REPORT.md` from the run results.

    python build_gallery.py

Reads `results/*.json`, converts `renders/*.png` to WebP (cwebp) so the whole
gallery fits in one file, and embeds them as data URIs -- no external CSS, JS,
fonts or images, because the page is published as an artifact under a strict
CSP.
"""

from __future__ import annotations

import base64
import html
import json
import os
import shutil
import subprocess
import sys
import time

HERE = os.path.dirname(os.path.abspath(__file__))
RESULTS = os.path.join(HERE, "results")
RENDERS = os.path.join(HERE, "renders")
WEB = os.path.join(RENDERS, "web")

WEBP_QUALITY = "76"
WEBP_WIDTH = "1000"

FAMILY = [
    ("X", "Crossings", "Two signals that cannot both go straight."),
    ("P", "Permutations", "The bits arrive somewhere other than where they left."),
    ("V", "Form changes", "Vertical stack, flat plane, and the corners between."),
    ("O", "Obstacles", "Something solid is in the way."),
    ("Z", "Vertical transport",
     "Changing level: what the solver can price, and two verified forms it "
     "cannot reach."),
    ("C", "Configurability", "One problem, four configs, side by side."),
    ("U", "Asked and refused", "Capabilities the router does not have yet."),
]


# ---------------------------------------------------------------------------
# data


def load():
    out = []
    for f in sorted(os.listdir(RESULTS)):
        if f.endswith(".json"):
            with open(os.path.join(RESULTS, f)) as fh:
                out.append(json.load(fh))
    return out


def make_webp():
    """PNG -> WebP so 19 renders fit in one HTML file. Idempotent."""
    if not os.path.isdir(RENDERS):
        return {}
    if not shutil.which("cwebp"):
        print("cwebp not found: embedding the PNGs instead (much larger)")
        return {f[:-4]: os.path.join(RENDERS, f)
                for f in os.listdir(RENDERS) if f.endswith(".png")}
    os.makedirs(WEB, exist_ok=True)
    out = {}
    for f in sorted(os.listdir(RENDERS)):
        if not f.endswith(".png"):
            continue
        src, dst = os.path.join(RENDERS, f), os.path.join(WEB, f[:-4] + ".webp")
        if (not os.path.exists(dst)
                or os.path.getmtime(dst) < os.path.getmtime(src)):
            subprocess.run(["cwebp", "-quiet", "-q", WEBP_QUALITY,
                            "-resize", WEBP_WIDTH, "0", src, "-o", dst],
                           check=True)
        out[f[:-4]] = dst
    return out


def data_uri(path):
    mime = "image/webp" if path.endswith(".webp") else "image/png"
    with open(path, "rb") as fh:
        return "data:%s;base64,%s" % (mime,
                                      base64.b64encode(fh.read()).decode())


def family_of(r):
    return r["id"][0]


def is_control(r):
    return "Control" in (r.get("title") or "")


# ---------------------------------------------------------------------------
# small helpers


def e(x):
    return html.escape(str(x), quote=True)


def num(x):
    return "&mdash;" if x is None else ("{:,}".format(x)
                                        if isinstance(x, int) else e(x))


def cfg_of(r):
    """The knobs this entry actually set, as the caller wrote them."""
    scn = r.get("scenario") or {}
    out = []
    for b in scn.get("buses", []):
        bits = []
        if b.get("style"):
            bits.append(("style", b["style"]))
        if b.get("rule"):
            bits.append(("rule (NetClassRule)", b["rule"]))
        if b.get("adapted") is not None:
            bits.append(("width adaptation", b["adapted"]))
        if bits:
            out.append((b["name"], bits))
    return out


def verdict(r):
    return "SOLVED" if r.get("solved") else "UNSOLVED"


def surprise(r):
    """Did the outcome contradict what the scenario predicted?"""
    want = (r.get("expect") or "solved").lower()
    got = verdict(r).lower()
    return None if want == got else (want, got)


# ---------------------------------------------------------------------------
# HTML


CSS = """
:root{
  /* light: cool paper, blue-biased neutrals -- the renders are navy */
  --ground:#eef0f5; --panel:#fbfbfd; --panel-2:#f2f4f8;
  --ink:#171b24; --ink-2:#454c5c; --ink-3:#6d7688;
  --rule:#d3d8e3; --rule-2:#e3e7ef;
  --redstone:#b8392a;              /* the subject's own signal colour */
  /* the renders' own backdrop: the same in both themes ON PURPOSE, so the
     plate reads as continuous with the image rather than framing it */
  --plate:#0e1119;
  --pass:#0f7a52; --pass-bg:#e2f1ea; --pass-line:#0f7a52;
  --open:#96591a; --open-bg:#f7eddc; --open-line:#b06d21;
  --shadow:0 1px 2px rgba(23,27,36,.06), 0 8px 24px -12px rgba(23,27,36,.18);
  --mono:ui-monospace,SFMono-Regular,"SF Mono","Cascadia Mono","Roboto Mono",
         Menlo,Consolas,monospace;
  --serif:"Iowan Old Style","Charter","Palatino Linotype",Palatino,
          "Source Serif 4",Georgia,serif;
}
@media (prefers-color-scheme:dark){
  :root:not([data-theme="light"]){
    --ground:#0e1119; --panel:#151a25; --panel-2:#1b2130;
    --ink:#e6e9f0; --ink-2:#a9b1c2; --ink-3:#7c8598;
    --rule:#28303f; --rule-2:#1f2634;
    --redstone:#e2604b;
    --pass:#4fc294; --pass-bg:#102a20; --pass-line:#2e9a72;
    --open:#e0a95a; --open-bg:#2a2013; --open-line:#b58335;
    --shadow:0 1px 2px rgba(0,0,0,.5), 0 10px 30px -14px rgba(0,0,0,.7);
  }
}
:root[data-theme="dark"]{
  --ground:#0e1119; --panel:#151a25; --panel-2:#1b2130;
  --ink:#e6e9f0; --ink-2:#a9b1c2; --ink-3:#7c8598;
  --rule:#28303f; --rule-2:#1f2634;
  --redstone:#e2604b;
  --pass:#4fc294; --pass-bg:#102a20; --pass-line:#2e9a72;
  --open:#e0a95a; --open-bg:#2a2013; --open-line:#b58335;
  --shadow:0 1px 2px rgba(0,0,0,.5), 0 10px 30px -14px rgba(0,0,0,.7);
}

*{box-sizing:border-box}
body{
  margin:0; background:var(--ground); color:var(--ink);
  font-family:var(--serif); font-size:17px; line-height:1.65;
  -webkit-font-smoothing:antialiased;
}
.wrap{max-width:1180px; margin:0 auto; padding:0 24px 96px}
a{color:var(--redstone); text-decoration-thickness:1px;
  text-underline-offset:2px}
a:focus-visible,summary:focus-visible{outline:2px solid var(--redstone);
  outline-offset:3px; border-radius:2px}
code,kbd{font-family:var(--mono)}

/* ---- masthead ---- */
.mast{padding:56px 0 28px; border-bottom:1px solid var(--rule)}
.eyebrow{font-family:var(--mono); font-size:11.5px; letter-spacing:.16em;
  text-transform:uppercase; color:var(--ink-3); margin:0 0 14px}
h1{font-family:var(--mono); font-size:clamp(26px,4.2vw,40px); font-weight:600;
  letter-spacing:-.015em; line-height:1.15; margin:0 0 14px;
  text-wrap:balance}
.lede{max-width:66ch; color:var(--ink-2); margin:0 0 8px}
.mast .meta{font-family:var(--mono); font-size:12px; color:var(--ink-3);
  margin-top:18px; display:flex; flex-wrap:wrap; gap:6px 18px}

/* ---- stat row ---- */
.stats{display:grid; gap:1px; background:var(--rule);
  grid-template-columns:repeat(auto-fit,minmax(150px,1fr));
  border:1px solid var(--rule); border-radius:3px; overflow:hidden;
  margin:32px 0 0}
.stat{background:var(--panel); padding:16px 18px}
.stat b{display:block; font-family:var(--mono); font-size:27px;
  font-weight:600; letter-spacing:-.02em; font-variant-numeric:tabular-nums;
  line-height:1.1}
.stat span{font-family:var(--mono); font-size:10.5px; letter-spacing:.13em;
  text-transform:uppercase; color:var(--ink-3)}
.stat.ok b{color:var(--pass)} .stat.open b{color:var(--open)}

h2{font-family:var(--mono); font-size:13px; font-weight:600;
  letter-spacing:.15em; text-transform:uppercase; color:var(--ink-3);
  margin:64px 0 6px; padding-bottom:10px; border-bottom:1px solid var(--rule)}
h2 + .sub{color:var(--ink-2); font-size:15.5px; margin:10px 0 22px;
  max-width:66ch}
h3{font-family:var(--mono); font-size:16.5px; font-weight:600; margin:0;
  letter-spacing:-.01em}

/* ---- ledger ---- */
.scroll{overflow-x:auto; border:1px solid var(--rule); border-radius:3px;
  background:var(--panel)}
table{border-collapse:collapse; width:100%; font-family:var(--mono);
  font-size:12.5px; font-variant-numeric:tabular-nums}
th,td{text-align:left; padding:8px 12px; border-bottom:1px solid var(--rule-2);
  white-space:nowrap}
th{font-size:10.5px; letter-spacing:.1em; text-transform:uppercase;
  color:var(--ink-3); background:var(--panel-2); position:sticky; top:0}
tbody tr:last-child td{border-bottom:0}
td.n{text-align:right}
.dim{color:var(--ink-3)}

/* ---- pills ---- */
.pill{display:inline-block; font-family:var(--mono); font-size:10.5px;
  font-weight:600; letter-spacing:.1em; text-transform:uppercase;
  padding:3px 8px; border-radius:2px; border:1px solid}
.pill.ok{color:var(--pass); background:var(--pass-bg);
  border-color:var(--pass-line)}
.pill.open{color:var(--open); background:var(--open-bg);
  border-color:var(--open-line)}
.pill.plain{color:var(--ink-3); background:var(--panel-2);
  border-color:var(--rule)}

/* ---- cards ---- */
.card{background:var(--panel); border:1px solid var(--rule); border-radius:3px;
  box-shadow:var(--shadow); margin:26px 0 0; overflow:hidden;
  border-left:3px solid var(--rule)}
.card.ok{border-left-color:var(--pass-line)}
.card.open{border-left-color:var(--open-line)}
.card > header{padding:20px 22px 16px; border-bottom:1px solid var(--rule-2);
  display:flex; flex-wrap:wrap; gap:10px 16px; align-items:baseline}
.card > header .id{font-family:var(--mono); font-size:11.5px;
  letter-spacing:.1em; color:var(--ink-3)}
.card > header .grow{flex:1 1 320px; min-width:0}
.q{margin:8px 22px 0; padding:0; color:var(--ink-2); font-size:15.5px;
  max-width:70ch}
.q b{font-family:var(--mono); font-size:10.5px; letter-spacing:.12em;
  text-transform:uppercase; color:var(--ink-3); display:block;
  margin-bottom:2px; font-weight:600}
figure{margin:18px 0 0; background:var(--plate);
  border-top:1px solid var(--rule-2);
  border-bottom:1px solid var(--rule-2)}
figure img{display:block; width:100%; height:auto; max-width:100%}
figcaption{font-family:var(--mono); font-size:11px; color:var(--ink-3);
  padding:8px 22px; background:var(--panel-2)}
.noart{padding:26px 22px; font-family:var(--mono); font-size:12.5px;
  color:var(--ink-3); background:var(--panel-2);
  border-top:1px solid var(--rule-2); border-bottom:1px solid var(--rule-2)}
.body{padding:20px 22px 22px; display:grid; gap:22px;
  grid-template-columns:minmax(0,1fr) minmax(0,1fr)}
@media (max-width:860px){ .body{grid-template-columns:minmax(0,1fr)} }
.block{min-width:0}
.block > h4{font-family:var(--mono); font-size:10.5px; font-weight:600;
  letter-spacing:.13em; text-transform:uppercase; color:var(--ink-3);
  margin:0 0 8px}
pre{margin:0; padding:12px 14px; background:var(--panel-2); overflow-x:auto;
  border:1px solid var(--rule-2); border-radius:3px;
  font-family:var(--mono); font-size:12px; line-height:1.6;
  color:var(--ink); white-space:pre}
.kv{font-family:var(--mono); font-size:12.5px;
  font-variant-numeric:tabular-nums}
.kv div{display:flex; justify-content:space-between; gap:16px;
  padding:5px 0; border-bottom:1px solid var(--rule-2)}
.kv div:last-child{border-bottom:0}
.kv span:first-child{color:var(--ink-3)}
.verdictbox{margin:0 22px; padding:12px 14px; border-radius:3px;
  font-size:15px; border:1px solid}
.verdictbox.open{background:var(--open-bg); border-color:var(--open-line)}
.verdictbox.ok{background:var(--pass-bg); border-color:var(--pass-line)}
.verdictbox b{font-family:var(--mono); font-size:10.5px; letter-spacing:.12em;
  text-transform:uppercase; display:block; margin-bottom:3px}
.note{margin:16px 22px 0; color:var(--ink-2); font-size:15px; max-width:74ch}
.note b{font-family:var(--mono); font-size:10.5px; letter-spacing:.12em;
  text-transform:uppercase; color:var(--ink-3); display:block;
  margin-bottom:2px; font-weight:600}
details{margin:16px 22px 0; border:1px solid var(--rule-2); border-radius:3px;
  background:var(--panel-2)}
details > summary{cursor:pointer; padding:9px 14px; font-family:var(--mono);
  font-size:11.5px; letter-spacing:.06em; color:var(--ink-2)}
details[open] > summary{border-bottom:1px solid var(--rule-2)}
details .scroll{border:0; border-radius:0; background:transparent}
.foot{margin-top:72px; padding-top:22px; border-top:1px solid var(--rule);
  font-family:var(--mono); font-size:12px; color:var(--ink-3)}
.foot p{max-width:80ch}
ul.plain{margin:0; padding-left:20px; color:var(--ink-2); font-size:15.5px}
ul.plain li{margin:9px 0; max-width:72ch}
@media (prefers-reduced-motion:reduce){*{transition:none!important;
  animation:none!important}}
"""


def stat_block(rs):
    solved = [r for r in rs if r.get("solved")]
    cases = sum((r.get("verification") or {}).get("total", 0) or 0 for r in rs)
    passed = sum((r.get("verification") or {}).get("passed", 0) or 0
                 for r in rs)
    solve = sum(r.get("solve_seconds") or 0 for r in rs)
    cells = sum(sum(m.get("cells", 0) for m in (r.get("per_bus") or {}).values())
                for r in rs)
    return """<div class="stats">
 <div class="stat ok"><b>%d</b><span>solved</span></div>
 <div class="stat open"><b>%d</b><span>unsolved</span></div>
 <div class="stat"><b>%d/%d</b><span>sim cases passed</span></div>
 <div class="stat"><b>%s</b><span>routed cells</span></div>
 <div class="stat"><b>%.2fs</b><span>total solver time</span></div>
</div>""" % (len(solved), len(rs) - len(solved), passed, cases,
             "{:,}".format(cells), solve)


def ledger(rs):
    rows = []
    for r in rs:
        v = r.get("verification") or {}
        cv = r.get("cost_vector") or {}
        cells = sum(m.get("cells", 0)
                    for m in (r.get("per_bus") or {}).values())
        ok = r.get("solved")
        rows.append(
            "<tr>"
            '<td><a href="#%s">%s</a></td>'
            '<td><span class="pill %s">%s</span></td>'
            "<td>%s</td>"
            '<td class="n">%s</td><td class="n">%s</td><td class="n">%s</td>'
            '<td class="n">%s</td><td class="n">%s</td><td class="n">%s</td>'
            "</tr>" % (
                e(r["id"]), e(r["id"]), "ok" if ok else "open",
                verdict(r), e(r["title"]),
                ("%d/%d" % (v.get("passed", 0), v.get("total", 0))
                 if v.get("total") else '<span class="dim">n/a</span>'),
                num(cells or None), num(cv.get("length")),
                num(cv.get("delay_rt")), num(cv.get("skew_rt")),
                "%.0f ms" % ((r.get("solve_seconds") or 0) * 1000)))
    return ('<div class="scroll"><table><thead><tr>'
            "<th>entry</th><th>verdict</th><th>scenario</th><th>sim</th>"
            "<th>cells</th><th>wire</th><th>delay rt</th><th>skew rt</th>"
            "<th>solve</th></tr></thead><tbody>%s</tbody></table></div>"
            % "".join(rows))


def card(r, imgs):
    ok = bool(r.get("solved"))
    cls = "ok" if ok else "open"
    v = r.get("verification") or {}
    cv = r.get("cost_vector") or {}
    per = r.get("per_bus") or {}
    scn = r.get("scenario") or {}
    out = ['<article class="card %s" id="%s">' % (cls, e(r["id"]))]

    sp = surprise(r)
    tags = ""
    if r.get("solver_produced") is False:
        tags += ('<span class="pill %s">form verified &middot; solver cannot '
                 "select</span>" % ("ok" if r.get("form_verified") else "open"))
    if sp:
        tags += ('<span class="pill plain">prediction was %s</span>' % e(sp[0]))
    out.append('<header><div class="grow"><div class="id">%s</div>'
               "<h3>%s</h3></div>"
               '<span class="pill %s">%s</span>%s</header>'
               % (e(r["id"]), e(r["title"]), cls, verdict(r), tags))

    if r.get("question"):
        out.append('<p class="q"><b>the question</b>%s</p>' % e(r["question"]))

    img = imgs.get(r["id"])
    if img:
        blocks = r.get("artifact_blocks")
        out.append('<figure><img alt="Isometric render of %s" src="%s">'
                   "<figcaption>direct render of %s &mdash; %s blocks, "
                   "textured via pack.zip</figcaption></figure>"
                   % (e(r["id"]), img, e(r.get("artifact") or ""),
                      num(blocks)))
    else:
        out.append('<div class="noart">No render: the router produced no '
                   "geometry for this scenario, so there is nothing to "
                   "show.</div>")

    out.append('<p class="verdictbox %s"><b>%s</b>%s</p>'
               % (cls, "verified in simulation" if ok else "blocked by",
                  e(r.get("blocked_by")
                    or ("%d/%d cases through mc-tick, DRC clean"
                        % (v.get("passed", 0), v.get("total", 0))))))

    out.append('<div class="body">')

    # -- solver invocation
    calls = []
    for name, b in (r.get("buses") or {}).items():
        calls.append("# %s -> %s" % (name, b.get("state")))
        calls.append(b.get("call") or "?")
        if b.get("error"):
            calls.append("#   %s" % b["error"])
    out.append('<div class="block"><h4>solver invocation</h4><pre>%s</pre>'
               "</div>" % e("\n".join(calls)))

    # -- config
    conf = cfg_of(r)
    if conf:
        lines = []
        for busname, bits in conf:
            for label, val in bits:
                lines.append("%s.%s = %s"
                             % (busname, label.split(" ")[0],
                                json.dumps(val, sort_keys=True)))
        body = e("\n".join(lines))
    else:
        body = "(router defaults: no style, no net-class rule)"
    ce = r.get("config_effect")
    if ce:
        body += e("\n\n# effect vs %s: %s" % (ce["baseline"], ce["verdict"]))
    out.append('<div class="block"><h4>configuration asked for</h4><pre>%s'
               "</pre></div>" % body)

    # -- numbers
    kv = [
        ("length (wire cells)", num(cv.get("length"))),
        ("delay", ("%s rt" % cv["delay_rt"]) if cv.get("delay_rt") is not None
         else "&mdash;"),
        ("skew", ("%s rt" % cv["skew_rt"]) if cv.get("skew_rt") is not None
         else "&mdash;"),
        ("coherence", '<span class="dim">not exposed by the bridge</span>'),
        ("footprint (bbox cells)", num(cv.get("footprint"))),
        ("bus cells emitted",
         num(sum(m.get("cells", 0) for m in per.values()) or None)),
        ("devices / glass",
         "%d / %d" % (sum(m.get("devices", 0) for m in per.values()),
                      sum(m.get("glass", 0) for m in per.values()))),
        ("artifact blocks", num(r.get("artifact_blocks"))),
        ("solver wall clock", "%.1f ms" % ((r.get("solve_seconds") or 0)
                                          * 1000)),
        ("entry wall clock", "%.2f s" % (r.get("wall_seconds") or 0)),
    ]
    out.append('<div class="block"><h4>cost vector &amp; size</h4>'
               '<div class="kv">%s</div></div>'
               % "".join("<div><span>%s</span><span>%s</span></div>" % k
                         for k in kv))

    # -- verification
    ver = []
    for label, s in (v.get("sections") or {}).items():
        ver.append("<div><span>%s</span><span>%d/%d</span></div>"
                   % (e(label), s["passed"], s["total"]))
    chk = r.get("check") or {}
    ver.append("<div><span>DRC / LVS (check)</span><span>%s</span></div>"
               % e(chk.get("repr") or "not reached"))
    for line in (chk.get("rules") or [])[:6]:
        ver.append('<div><span class="dim">rule</span><span>%s</span></div>'
                   % e(line))
    if not (v.get("sections") or {}):
        ver.insert(0, '<div><span>simulation</span><span class="dim">'
                      "not reached</span></div>")
    out.append('<div class="block"><h4>verification (mc-tick)</h4>'
               '<div class="kv">%s</div></div>' % "".join(ver))

    out.append("</div>")   # .body

    # -- fixtures
    fx = r.get("fixtures") or {}
    if fx:
        rows = "".join(
            "<div><span>%s (%s)</span><span>%s</span></div>"
            % (e(k), e(x.get("kind")), e(x.get("provenance")))
            for k, x in fx.items())
        out.append('<details><summary>Fixtures placed in this scene (NOT '
                   "solver output)</summary>"
                   '<div style="padding:12px 14px"><div class="kv">%s</div>'
                   "</div></details>" % rows)

    # -- case table
    cases = v.get("cases") or []
    if cases:
        keys = []
        for c in cases:
            for k in c:
                if k not in keys and k != "section":
                    keys.append(k)
        head = "".join("<th>%s</th>" % e(k) for k in ["section"] + keys)
        body_rows = []
        for c in cases:
            tds = ['<td>%s</td>' % e(c.get("section", ""))]
            for k in keys:
                val = c.get(k)
                if k == "ok":
                    tds.append('<td><span class="pill %s">%s</span></td>'
                               % ("ok" if val else "open",
                                  "pass" if val else "fail"))
                elif isinstance(val, int):
                    tds.append('<td class="n">%d</td>' % val)
                else:
                    tds.append("<td>%s</td>"
                               % ("&mdash;" if val is None else e(val)))
            body_rows.append("<tr>%s</tr>" % "".join(tds))
        out.append("<details><summary>Every simulation case (%d)</summary>"
                   '<div class="scroll"><table><thead><tr>%s</tr></thead>'
                   "<tbody>%s</tbody></table></div></details>"
                   % (len(cases), head, "".join(body_rows)))

    if r.get("notes"):
        out.append('<p class="note"><b>what this entry establishes</b>%s</p>'
                   % e(r["notes"]).replace("\n\n", "<br><br>"))

    out.append('<p class="note"><b>reproduce</b>'
               '<code>python run_corpus.py %s</code> &nbsp; '
               "scenario: <code>%s</code></p>"
               % (e(r["id"]), e(scn.get("_file") or "")))
    out.append("</article>")
    return "".join(out)


def roadmap_items(rs):
    """The blocking assumptions, grouped, worst first."""
    return [
        ("No clearance policy reaches the router",
         "X02 / U03",
         "The router avoids OCCUPIED cells correctly and then runs its own "
         "dust diagonally adjacent to a live foreign wire, which couples the "
         "two signals. `NetClassRule.spacing` exists for exactly this and is "
         "never read on the routing path. Until a clearance rule reaches the "
         "router, no bus can be trusted next to a neighbour that carries a "
         "signal."),
        ("A flat-form bundle that detours does not conduct, and still "
         "reports success",
         "O03 / X03",
         "One plain wall in front of a flat 2-pitch bundle is enough: the "
         "router returns state `routed`, `check()` is clean, and in "
         "simulation lanes are dead or shorted together. Vertical-form "
         "bundles detour correctly (O01, O02), so the defect is in the flat "
         "detour. A router that reports success for a bus that does not "
         "carry bits is the most damaging failure mode in this list."),
        ("One level-change mechanism, and it is the expensive one",
         "Z01 / Z02 / Z03",
         "The router changes level only with the level-shift tile, priced at 2 "
         "cells of straight horizontal run per y, so an 8-level climb between "
         "ports 5 apart is refused outright. Two denser forms are already "
         "built and measured in `vforms.py`: the torch ladder climbs at ONE xz "
         "cell per bit per level with no reach limit (256/256 here), and the "
         "ring riser descends passively at 1 y per cell where redstone has no "
         "active descending carrier at all (verified here too). Neither can be "
         "selected through any call. This is the largest capability gap in the "
         "corpus, and unlike the defects above it is purely additive work."),
        ("Every configuration knob except materials is decoration",
         "C02 / C03 / U02 / U03",
         "`NetClassRule` is read in `Design::check` only (src/design.rs "
         "~4289): `y_band` and `max_len_rt` are reported after the fact, and "
         "`region`, `spacing` and `direction_bias` are read nowhere at all. "
         "`RoutingRegion` -- include/exclude zones with a legality predicate "
         "and Insign authoring -- has no consumer in `design.rs`. And the "
         "cost weights are hard-coded: `route_bus` sets `cost: "
         "BusCost::default()` and neither the compact nor the "
         "latency-optimised preset can be selected from any binding, because "
         "`src/bridge/design.rs` contains no reader or writer for cost at "
         "all. Four entries here set a knob, verify, and come out with "
         "geometry identical to the baseline."),
        ("Ports must be lever banks and lamp banks",
         "U01",
         "`declare_input` needs a lever beside each bit's dust and "
         "`declare_output` needs a lamp under it; a bare dust tap is refused "
         "in both directions. So the router cannot be attached to another "
         "mechanism's output -- which is most of what daily work is."),
        ("No analog carrier and no analog type",
         "U01",
         "Every bus is bits-on-lanes. `IoType` has no signal-strength "
         "variant, and the router has no form that emits the hex comb stage "
         "-- a value-preserving analog carrier that is already measured, "
         "verified 66/66, and sitting in the corpus unused."),
        ("Net order is the caller's problem",
         "P02 / P02b",
         "The same eight-net permutation routes 8/8 hardest-first and 7/8 in "
         "declaration order, because nets are placed greedily with no "
         "rip-up-and-retry. Nothing in the API mentions ordering."),
        ("Descending port steps are refused outright",
         "P01",
         "A sink declared msb-first (step `(0,-2,0)`) is refused: "
         "\"this design realizes the verified vertical 2y-pitch stack ... "
         "port `a_out` has step (0, -2, 0), which is neither\". A bundle-level "
         "reversal or shuffle has to be decomposed into 1-bit nets by hand "
         "(P02), which costs a long straight run per net."),
        ("Form adapters have no clearance query",
         "V03b",
         "The flat 90-degree corner routes with the sink 38 blocks out and "
         "fails with an internal plan conflict at 24. The error names a cell, "
         "not a clearance, so the only way to find the limit is to try."),
        ("The router's own cost vector is invisible",
         "every entry",
         "`BusCostVector` (length / delay_rt / skew_rt / coherence / "
         "footprint) is computed in src/design.rs and has a `to_json`, but no "
         "binding exposes a reader. Every number in this gallery except delay "
         "and skew is measured by the harness off the emitted cells, and "
         "`coherence` is left null rather than faked."),
    ]


def build_html(rs, imgs):
    solved = [r for r in rs if r.get("solved")]
    doc = ['<title>Bus solver corpus &mdash; %d scenarios, %d solved</title>'
           % (len(rs), len(solved)),
           "<style>%s</style>" % CSS, '<div class="wrap">']

    doc.append("""<header class="mast">
<p class="eyebrow">Nucleation &middot; redstone-eda &middot; bus router</p>
<h1>What the bus solver can and cannot route</h1>
<p class="lede">%d scenarios, declared as data, handed to
<code>nucleation.Design.route_bus</code>, and then driven in the mc-tick engine
with real values. Every cell of every bus in every picture was produced by the
solver. Nothing here is hand-drawn geometry presented as router output: where
the router could not do the job, the entry says <b>UNSOLVED</b> and names the
assumption that blocked it.</p>
<p class="lede">The unsolved half is the point. %d blocking assumptions came out
of this run, two of them defects that pass DRC and one a capability that is
already built and measured but cannot be selected.</p>
<div class="meta"><span>%s</span><span>runner: run_corpus.py</span>
<span>renders: nucleation mesher + pack.zip</span>
<span>engine: mc-tick, TickSettleMode.Placement</span></div>
%s
</header>""" % (len(rs), len(roadmap_items(rs)), time.strftime("%Y-%m-%d"),
                stat_block(rs)))

    doc.append("<h2>The ledger</h2>")
    doc.append('<p class="sub">Every entry, its verdict, and what it cost. '
               "Cells and wire are measured off the blocks the router "
               "emitted; delay and skew are the router's own numbers via "
               "<code>bus_skew</code>.</p>")
    doc.append(ledger(rs))

    for key, name, blurb in FAMILY:
        fam = [r for r in rs if family_of(r) == key]
        if not fam:
            continue
        doc.append("<h2>%s</h2>" % e(name))
        doc.append('<p class="sub">%s</p>' % e(blurb))
        for r in fam:
            doc.append(card(r, imgs))

    doc.append("<h2>What the solver cannot do yet</h2>")
    doc.append('<p class="sub">The roadmap, worst first. Each item is the '
               "assumption a specific entry above ran into, not a wish "
               "list.</p><ul class=\"plain\">")
    for i, (title, where, text) in enumerate(roadmap_items(rs), 1):
        doc.append("<li><b>%s</b> <span class=\"pill plain\">%s</span><br>%s"
                   "</li>" % (e(title), e(where), e(text)))
    doc.append("</ul>")

    doc.append("""<div class="foot">
<p>Built by <code>redstone-eda/corpus_gallery/</code>:
<code>run_corpus.py</code> runs each scenario in its own process,
<code>render.py</code> renders the baked <code>.schem</code> with the same
pipeline as <code>docs/render_gallery.py</code>, and
<code>build_gallery.py</code> writes this page. Raw results are in
<code>results/*.json</code>; the same data as markdown is in
<code>REPORT.md</code>.</p>
<p>Wheel: built from this working tree with
<code>NUCLEATION_FEATURES=bridge-full,routing,hdl</code>. The corpus re-runs in
about four seconds, so it is cheap to point at a change.</p>
</div></div>""")
    return "\n".join(doc)


# ---------------------------------------------------------------------------
# markdown


def build_md(rs):
    solved = [r for r in rs if r.get("solved")]
    L = ["# Bus solver corpus",
         "",
         "%d scenarios, **%d solved**, **%d unsolved**. Generated by "
         "`build_gallery.py` on %s."
         % (len(rs), len(solved), len(rs) - len(solved),
            time.strftime("%Y-%m-%d")),
         "",
         "Every bus cell in every entry was produced by "
         "`nucleation.Design.route_bus` / `route_bus_adapted` and verified by "
         "driving real values through the baked artifact in mc-tick. Where the "
         "router could not do the job the entry is UNSOLVED and names the "
         "blocking assumption; no geometry was hand-authored and presented as "
         "solver output.",
         "",
         "## Ledger",
         "",
         "| entry | verdict | scenario | sim cases | cells | wire | delay rt |"
         " skew rt | footprint | solve |",
         "|---|---|---|---|---|---|---|---|---|---|"]
    for r in rs:
        v = r.get("verification") or {}
        cv = r.get("cost_vector") or {}
        cells = sum(m.get("cells", 0)
                    for m in (r.get("per_bus") or {}).values())
        L.append("| `%s` | **%s** | %s | %s | %s | %s | %s | %s | %s | %.0f ms |"
                 % (r["id"], verdict(r), r["title"],
                    ("%d/%d" % (v.get("passed", 0), v.get("total", 0))
                     if v.get("total") else "n/a"),
                    cells or "-", cv.get("length", "-"),
                    cv.get("delay_rt", "-"), cv.get("skew_rt", "-"),
                    cv.get("footprint", "-"),
                    (r.get("solve_seconds") or 0) * 1000))

    L += ["", "`coherence` is absent from every row on purpose: "
              "`BusCostVector` is computed in `src/design.rs` but no binding "
              "exposes a reader, so the router's own number cannot be read. "
              "Length, footprint and cell counts are measured by the harness "
              "off the emitted blocks; delay and skew come from `bus_skew`.",
          ""]

    for key, name, blurb in FAMILY:
        fam = [r for r in rs if family_of(r) == key]
        if not fam:
            continue
        L += ["## %s" % name, "", blurb, ""]
        for r in fam:
            v = r.get("verification") or {}
            cv = r.get("cost_vector") or {}
            L += ["### `%s` &mdash; %s" % (r["id"], verdict(r)),
                  "",
                  "**%s**" % r["title"], "",
                  r.get("question", ""), ""]
            sp = surprise(r)
            if sp:
                L.append("> Prediction was **%s**, outcome was **%s**.\n"
                         % sp)
            L += ["```", ]
            for bname, b in (r.get("buses") or {}).items():
                L.append("# %s -> %s" % (bname, b.get("state")))
                L.append(b.get("call") or "?")
                if b.get("error"):
                    L.append("#   " + b["error"])
            L += ["```", ""]
            conf = cfg_of(r)
            if conf:
                L.append("Config: " + "; ".join(
                    "`%s.%s = %s`" % (bn, lbl.split(" ")[0],
                                      json.dumps(val, sort_keys=True))
                    for bn, bits in conf for lbl, val in bits))
                L.append("")
            L.append("| metric | value |")
            L.append("|---|---|")
            for k, val in (("length (wire cells)", cv.get("length")),
                           ("delay_rt", cv.get("delay_rt")),
                           ("skew_rt", cv.get("skew_rt")),
                           ("coherence", "not exposed by the bridge"),
                           ("footprint", cv.get("footprint")),
                           ("artifact blocks", r.get("artifact_blocks")),
                           ("solver wall clock",
                            "%.1f ms" % ((r.get("solve_seconds") or 0)
                                         * 1000)),
                           ("check",
                            (r.get("check") or {}).get("repr")
                            or "not reached"),
                           ("simulation",
                            "%d/%d cases" % (v.get("passed", 0),
                                             v.get("total", 0))
                            if v.get("total") else "not reached")):
                L.append("| %s | %s |" % (k, "-" if val is None else val))
            L.append("")
            for label, s in (v.get("sections") or {}).items():
                L.append("- %s: **%d/%d**" % (label, s["passed"], s["total"]))
            if v.get("sections"):
                L.append("")
            ce = r.get("config_effect")
            if ce:
                L += ["Config effect vs `%s`: **%s**"
                      % (ce["baseline"], ce["verdict"]), ""]
            if not r.get("solved"):
                L += ["**Blocked by:** %s" % r.get("blocked_by", "?"), ""]
            fx = r.get("fixtures") or {}
            for k, x in fx.items():
                L.append("Fixture `%s` (%s), NOT solver output: %s"
                         % (k, x.get("kind"), x.get("provenance")))
            if fx:
                L.append("")
            if r.get("notes"):
                L += [r["notes"], ""]
            if r.get("artifact"):
                L += ["Artifact: `%s` &nbsp; Render: `renders/%s.png`"
                      % (r["artifact"], r["id"]), ""]

    L += ["## What the solver cannot do yet", "",
          "The roadmap, worst first. Each item is the assumption a specific "
          "entry above ran into.", ""]
    for i, (title, where, text) in enumerate(roadmap_items(rs), 1):
        L += ["### %d. %s" % (i, title), "", "*Entries: %s*" % where, "",
              text, ""]

    L += ["## Reproducing", "",
          "```sh",
          "# a wheel built WITH routing (the released one has no Design)",
          "NUCLEATION_FEATURES=bridge-full,routing,hdl \\",
          "    <venv>/bin/pip install ./bindings/python",
          "",
          "cd redstone-eda/corpus_gallery",
          "python run_corpus.py          # ~4 s, one subprocess per scenario",
          "python render.py              # PNG per artifact (needs pack.zip)",
          "python build_gallery.py       # index.html + REPORT.md",
          "```", ""]
    return "\n".join(L)


def main():
    rs = load()
    if not rs:
        print("no results -- run run_corpus.py first")
        return 1
    paths = make_webp()
    imgs = {k: data_uri(p) for k, p in paths.items()}
    html_path = os.path.join(HERE, "index.html")
    with open(html_path, "w") as fh:
        fh.write(build_html(rs, imgs))
    md_path = os.path.join(HERE, "REPORT.md")
    with open(md_path, "w") as fh:
        fh.write(build_md(rs))
    print("index.html  %.2f MB (%d renders embedded)"
          % (os.path.getsize(html_path) / 1e6, len(imgs)))
    print("REPORT.md   %.1f KB" % (os.path.getsize(md_path) / 1e3))
    solved = len([r for r in rs if r.get("solved")])
    print("%d scenarios: %d solved, %d unsolved"
          % (len(rs), solved, len(rs) - solved))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
