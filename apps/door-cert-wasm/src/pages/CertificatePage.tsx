import { useMemo } from "react";
import { Link, useParams } from "react-router-dom";
import { loadRecord, loadXray } from "../lib/store";
import { Seal } from "../components/Seal";
import { StatTile } from "../components/StatTile";
import { ActivityChart } from "../components/ActivityChart";
import { ClassificationBlock } from "../components/ClassificationBlock";
import { Heatmap } from "../components/Heatmap";
import { MaterialsGrid, stackMath } from "../components/MaterialsGrid";
import { MeshReplay } from "../components/MeshReplay";
import type {
  Badges,
  BlockEntityAudit,
  Census,
  Engineering,
  ResetTime,
  Symmetry,
} from "../lib/types";

/** Minecraft runs at 20 ticks a second, so ticks convert to in-game seconds.
 *  The simulation itself finishes in a fraction of that. */
const secs = (ticks: number) => (ticks / 20).toFixed(2);
const pct = (v: number) => (v < 1 ? v.toFixed(1) : String(Math.round(v)));
const tk = (n: number) => `${n} tick${n === 1 ? "" : "s"}`;
const fmt = (n: number) => Math.round(n).toLocaleString("en-US");
const plural = (n: number, one: string, many = one + "s") =>
  `${fmt(n)} ${n === 1 ? one : many}`;
/** Block ids read as words on the sheet; the namespace is noise here. */
const blockWord = (id: string) => id.replace(/^minecraft:/, "").replace(/_/g, " ");

/** "4 comparators and 4 furnaces" — the absent block entities named and
 *  counted, densest first, in the order the engine reported them. Naming them
 *  is the whole point of the audit: "9 block entities missing" tells a builder
 *  nothing they can act on, "4 comparators" tells them exactly which part of
 *  their door this run got wrong. */
const missingList = (missing: BlockEntityAudit["missing"]) => {
  const parts = missing.map((m) => plural(m.count, blockWord(m.name)));
  if (parts.length <= 1) return parts[0] ?? "";
  return `${parts.slice(0, -1).join(", ")} and ${parts[parts.length - 1]}`;
};

/** Redstone can move on the very tick the lever is thrown, and that reads
 *  better as a word than as a zero. */
const responds = (ticks: number | null) =>
  ticks === null
    ? "nothing moved"
    : ticks === 0
      ? "responds instantly"
      : `responds in ${tk(ticks)}`;

/** The doorway time next to the settle time it sits inside. The gap is the
 *  interesting part: a door can be walkable long before its tape stops. */
function strokeSub(doorway: number | null, settle: number | null, latency: number | null) {
  if (doorway === null)
    return settle === null ? "not measured" : `never cleared · settles at ${tk(settle)}`;
  const tail =
    settle === null
      ? ""
      : settle > doorway
        ? ` · settles at ${settle}`
        : " · quiet the same tick";
  return `${secs(doorway)} s · ${responds(latency)}${tail}`;
}

function resetSub(r: ResetTime | null, what: string) {
  if (!r) return "not measured";
  if (r.ticks === null) return r.note ?? `none found within ${r.searched} ticks`;
  if (r.stroke_ticks === null) return `before the lever may ${what} again`;
  return r.negative
    ? `${r.ticks} < ${r.stroke_ticks}-tick stroke — re-triggerable mid-stroke`
    : `${r.ticks} vs the ${r.stroke_ticks}-tick stroke`;
}

/** The parts that make it move, most structural first. Zero counts are left
 *  out — a door without observers should not have to say so. */
function censusRows(c: Census): { term: string; detail: string }[] {
  const rows: { term: string; detail: string }[] = [];
  const add = (n: number, one: string, many: string, detail = "") => {
    if (n > 0) rows.push({ term: `${n} ${n === 1 ? one : many}`, detail });
  };
  add(c.sticky_piston, "sticky piston", "sticky pistons");
  add(c.piston, "piston", "pistons");
  add(c.observer, "observer", "observers");
  add(
    c.repeater,
    "repeater",
    "repeaters",
    c.repeater_delays.length
      ? `delay ${c.repeater_delays.join(", ")}`
      : "",
  );
  add(c.comparator, "comparator", "comparators");
  add(c.redstone_block, "redstone block", "redstone blocks");
  add(c.redstone_torch, "redstone torch", "redstone torches");
  add(c.redstone_wire, "dust", "dust");
  add(c.slime_block, "slime block", "slime blocks");
  add(c.honey_block, "honey block", "honey blocks");
  return rows;
}

/** Symmetry as two sentences rather than a grid of ticks and crosses. It is
 *  one bit per axis; a table would be four times the furniture for the same
 *  four bits, and the sentence carries the caveat with it. */
function symmetryLines(s: Symmetry): { pattern: string; machine: string } {
  const p = s.pattern;
  const pattern = !p
    ? "No walkable passage was extracted, so there is no pattern to mirror."
    : p.horizontal && p.vertical
      ? "The pattern mirrors both ways — left to right and top to bottom."
      : p.horizontal
        ? "The pattern mirrors left to right, but not top to bottom."
        : p.vertical
          ? "The pattern mirrors top to bottom, but not left to right."
          : "The pattern mirrors on neither axis.";
  const exact = s.machine.filter((m) => m.mirror).map((m) => m.axis);
  // Nothing but the pattern is ever exactly symmetric — the control sits on one
  // side — so the best axis reports how close it gets rather than failing.
  const best = [...s.machine].sort((a, b) => b.share - a.share)[0];
  const machine = exact.length
    ? `The whole build mirrors ${exact.join(" and ")} exactly.`
    : best && best.share > 0
      ? `The build itself is closest to symmetric ${best.axis}: ${pct(
          best.share * 100,
        )}% of its blocks have a mirrored twin.`
      : "The build itself is not symmetric on any axis.";
  return { pattern, machine };
}

/** The four qualifiers, each as a name and the evidence for or against it.
 *  A badge that only appears when earned tells you nothing when it is absent —
 *  you cannot tell "not observerless" from "not measured" — so all four are
 *  always shown and the ones not earned carry their own count. */
function badgeRows(b: Badges, c: Census): { name: string; on: boolean; detail: string }[] {
  const slime = c.slime_block + c.honey_block;
  return [
    {
      name: "observerless",
      on: b.observerless,
      detail: b.observerless ? "no observers" : plural(c.observer, "observer"),
    },
    {
      name: "dustless",
      on: b.dustless,
      detail: b.dustless ? "no dust" : `${fmt(c.redstone_wire)} dust`,
    },
    {
      name: "slimeless",
      on: b.slimeless,
      detail: b.slimeless
        ? "no slime, no honey"
        : `${fmt(c.slime_block)} slime + ${fmt(c.honey_block)} honey = ${fmt(slime)}`,
    },
    {
      name: b.cycleless ? "cycle-less" : `tape of period ${b.tape?.period ?? "?"}`,
      on: b.cycleless,
      detail: b.cycleless
        ? b.pistons > 0
          ? `${plural(b.pistons, "piston")} fire · busiest ${fmt(b.busiest)}×, never on a steady period`
          : "no piston fired"
        : `${plural(b.tape!.pistons, "piston")} run it · busiest fires ${fmt(
            b.tape!.fires,
          )}×`,
    },
  ];
}

/** The engineering readings: what the door costs, what of it is idle, what the
 *  update order shows, and how symmetric it is.
 *
 *  Grouped rather than folded into the measurement tiles because they answer a
 *  different question. The tiles time the door; these describe the machine —
 *  and one of them (server cost) needs its caveat printed beside it, which a
 *  tile has no room for. */
function EngineeringSection({
  eng,
  census,
  peakChanges,
  peakTick,
}: {
  eng: Engineering;
  census: Census;
  /** The mechanical peak, moved here from the measurement tiles so it sits
   *  beside the computational one it is the counterpart to. */
  peakChanges: number;
  peakTick: number;
}) {
  const cost = eng.cost;
  const dead = eng.dead;
  const first = eng.first;
  const sym = symmetryLines(eng.symmetry);
  const badges = badgeRows(eng.badges, census);
  const idleShare = dead && dead.total > 0 ? (dead.idle / dead.total) * 100 : null;

  return (
    <div className="sheet-section" data-testid="engineering">
      <p className="eyebrow">Engineering</p>
      <h3 className="eng-title">What it costs and what earns its place</h3>

      <ul className="eng-badges" data-testid="badges">
        {badges.map((b) => (
          <li key={b.name} className={"eng-badge" + (b.on ? " on" : "")}>
            <b>{b.name}</b>
            <span>{b.detail}</span>
          </li>
        ))}
      </ul>

      {cost ? (
        <>
          <div className="eng-lead" data-testid="server-cost">
            <p className="eng-figure">
              <b>{fmt(cost.updates)}</b>
              <span>update dispatches per cycle</span>
            </p>
            <dl className="eng-facts">
              <div>
                <dt>block events</dt>
                <dd>
                  {fmt(cost.block_events)}
                  <i>
                    {cost.updates > 0
                      ? ` · ${pct((cost.block_events / cost.updates) * 100)}% of the total`
                      : ""}
                  </i>
                </dd>
              </div>
              <div>
                <dt>busiest tick</dt>
                <dd>
                  {fmt(cost.peak)}
                  <i> · on tick {cost.peak_tick}</i>
                </dd>
              </div>
              {cost.per_passage_cell !== null && (
                <div>
                  <dt>per doorway cell</dt>
                  <dd>
                    {fmt(cost.per_passage_cell)}
                    <i> · the size-fair number</i>
                  </dd>
                </div>
              )}
              {cost.per_second !== null && (
                <div>
                  <dt>held on a loop</dt>
                  <dd>
                    {fmt(cost.per_second)}
                    <i> · dispatches a second</i>
                  </dd>
                </div>
              )}
            </dl>
          </div>
          <p className="chart-note eng-note">
            A dispatch is one update delivered to one cell. It is a <b>count, not
            milliseconds</b>, and it does not predict TPS — a dust update and a
            block-entity tick cost a real server very different amounts. What it is good
            for is comparing doors: the same engine measured every one of them, over the
            same definition of a cycle, from the same seed. The per-doorway-cell figure is
            the one to rank on, because it does not reward a small door for being small.
          </p>
          <div className="tiles tiles-pair">
            <StatTile
              label="Peak in flight"
              value={peakChanges}
              unit=" cells"
              sub={`the mechanical peak · moving on tick ${peakTick}`}
            />
            <StatTile
              label="Peak dispatches"
              value={fmt(cost.peak)}
              unit=" updates"
              sub={`the computational peak · on tick ${cost.peak_tick}`}
            />
          </div>
        </>
      ) : (
        <p className="chart-note" data-testid="server-cost-missing">
          The update recorder did not run on this door, so there is no cost reading. Every
          other number on this sheet is unaffected.
        </p>
      )}

      <div className="eng-grid">
        <div className="eng-item" data-testid="dead-weight">
          <p className="eng-label">Dead weight</p>
          {dead && dead.idle === 0 ? (
            /* Nothing idle is a finding, not an empty state, and it deserves the
               sentence rather than a "0 of 345 — 0.0%" that reads as a bug. */
            <>
              <p className="eng-claim">
                <b>Every one</b> of its {fmt(dead.total)} blocks did something this cycle
              </p>
              <p className="eng-body">
                Each block either moved or took at least one update — nothing in this
                build sits out the stroke. That is as compact as this measurement gets.
              </p>
            </>
          ) : dead ? (
            <>
              <p className="eng-claim">
                <b>
                  {fmt(dead.idle)} of {fmt(dead.total)}
                </b>{" "}
                blocks did nothing this cycle
                {idleShare !== null ? ` — ${pct(idleShare)}%` : ""}
              </p>
              {dead.by_id.length > 0 && (
                <p className="eng-detail">
                  {dead.by_id
                    .slice(0, 4)
                    .map((r) => `${blockWord(r.id)} ×${fmt(r.count)}`)
                    .join(" · ")}
                  {dead.by_id.length > 4 ? ` · +${dead.by_id.length - 4} more kinds` : ""}
                </p>
              )}
              <p className="eng-body">
                Neither moved nor received a single update — the set difference of two
                complete logs. <b>That is not a removal list.</b> A block that only holds
                another one up is load-bearing precisely because it never has to do
                anything, and it lands here too. Read it as an upper bound on decoration
                and redundancy, then check the cells: <b>Dead weight</b> on the replay
                above marks every one of them.
              </p>
            </>
          ) : (
            <p className="eng-body">
              Needs the update log, which did not record for this door.
            </p>
          )}
        </div>

        <div className="eng-item" data-testid="first-movement">
          <p className="eng-label">First movement</p>
          {first ? (
            <>
              <p className="eng-chain">
                {first.chain.length > 0
                  ? first.chain.map((c, i) => (
                      <span key={c.id + i}>
                        {i > 0 && <i aria-hidden> → </i>}
                        {c.id}
                        {c.cells > 1 ? ` ×${fmt(c.cells)}` : ""}
                      </span>
                    ))
                  : "no component took an update first"}
              </p>
              <p className="eng-detail">
                {plural(first.hops, "update")}{" "}
                {first.ticks === 0
                  ? "on the click tick itself"
                  : `across ${tk(first.ticks)}`}{" "}
                before {blockWord(first.block)} at ({first.pos.join(", ")}) moved
              </p>
              <p className="eng-body">
                The components the engine delivered updates to between the click and the
                first block that moved, in <b>delivery order</b>. The engine records the
                order — that is what makes it worth stating at all — but not parentage:
                nothing in the log says which update scheduled which. So this is a
                sequence, not a proven critical path, and each count is distinct cells of
                that component, not hops through them.
              </p>
            </>
          ) : (
            <p className="eng-body">
              Needs the update log, which did not record for this door.
            </p>
          )}
        </div>

        <div className="eng-item" data-testid="symmetry">
          <p className="eng-label">Symmetry</p>
          <p className="eng-claim">{sym.pattern}</p>
          <p className="eng-detail">{sym.machine}</p>
          <p className="eng-body">
            Base block names only. A mirrored piston faces the other way, so demanding the
            block state match as well would report every symmetric door as asymmetric; the
            question asked is whether the same part sits in the mirrored cell.
          </p>
        </div>
      </div>
    </div>
  );
}

export function CertificatePage() {
  const { id = "" } = useParams();
  const rec = useMemo(() => loadRecord(id), [id]);
  const xray = useMemo(() => loadXray(id), [id]);

  const certUrl = useMemo(() => {
    if (!rec) return "";
    const blob = new Blob([JSON.stringify(rec.certificate, null, 1)], {
      type: "application/json",
    });
    return URL.createObjectURL(blob);
  }, [rec]);

  if (!rec)
    return (
      <div className="error-page">
        <p>Certificate {id} is not in this browser's records.</p>
        <p>
          <Link to="/">Submit a door</Link>
        </p>
      </div>
    );

  const cert = rec.certificate;
  const [dx, dy, dz] = cert.dims;
  const movingBlocks = cert.heatmap.values.flat().filter((v) => v > 0).length;
  const totalBlocks = cert.materials.reduce((a, m) => a + m.count, 0);
  const stacks = Math.ceil(totalBlocks / 64);
  const ap = cert.aperture;
  const conflict = cert.aperture_conflict ?? null;
  /** Null on structure SNBT, which never becomes a Schematic, and on records
   *  certified before the audit existed. */
  const audit = cert.block_entity_audit ?? null;
  const missingBE = audit && audit.missing_total > 0 ? audit : null;
  /** Any reason the run took a reading it will not stand behind. The aperture
   *  dispute is one. A file missing block-entity data is the other, and it
   *  taints the same numbers just as thoroughly: they time a machine whose
   *  comparators all read 0. Both keep every figure on the sheet — deleting a
   *  measurement is its own kind of dishonesty — and mark it where it is read. */
  const disputed = !!conflict || !!missingBE;
  /** What actually drives this door. Older records have no `input`, and a lever
   *  is what they all were. */
  const control = cert.input?.kind ?? "lever";
  const rows = censusRows(cert.census);
  /** Null on records certified before the engineering readings existed. */
  const eng = cert.engineering ?? null;
  const ro = cert.reset_open;
  const rc = cert.reset_close;
  // The doorway cycle: click to walkable, click to shut again. Null the
  // moment either half is unmeasured — half a cycle is not a cycle.
  const cycleTicks =
    cert.open_ticks !== null && cert.close_ticks !== null
      ? cert.open_ticks + cert.close_ticks
      : null;
  const settleCycle =
    cert.open_settle_ticks !== null && cert.close_settle_ticks !== null
      ? cert.open_settle_ticks + cert.close_settle_ticks
      : null;
  const resetCycle =
    ro?.ticks != null && rc?.ticks != null ? ro.ticks + rc.ticks : null;
  const negatives = [
    ro?.negative ? { r: ro, verb: "opening" } : null,
    rc?.negative ? { r: rc, verb: "closing" } : null,
  ].filter(Boolean) as { r: ResetTime; verb: string }[];

  return (
    <div className="sheet">
      <div className="sheet-band">
        <span>
          report <b>{id}</b>
        </span>
        <span>
          seed <b>{cert.seed}</b>
        </span>
        <span>
          verdict <b>{cert.verdict}</b>
        </span>
        <span>
          run <b>{new Date().toISOString().slice(0, 10)}</b>
        </span>
      </div>

      <div className="sheet-body">
        <div className="hero">
          <div className="hero-main">
            <p className="eyebrow">Piston door validation report</p>
            <h1>{cert.name}</h1>
            {conflict ? (
              /* The hero line is the most quotable thing on the sheet, so on a
                 disputed door it carries the dispute rather than one side of
                 it. Quoting this line alone still tells the truth. */
              <p className="hero-aperture hero-aperture-split" data-testid="aperture">
                <b>
                  {conflict.saved.w} × {conflict.saved.h} or {conflict.settled.w} ×{" "}
                  {conflict.settled.h}
                </b>{" "}
                disputed
                <span>
                  {conflict.saved.cells} cells as saved · {conflict.settled.cells} cells on the
                  cycle it settles into
                </span>
              </p>
            ) : ap ? (
              <p className="hero-aperture" data-testid="aperture">
                <b>
                  {ap.w} × {ap.h}
                </b>{" "}
                {ap.rectangular === false ? "envelope" : "aperture"}
                <span>
                  {ap.cells} cells clear{ap.depth > 1 ? ` · ${ap.depth} deep` : ""}
                  {ap.note ? ` · ${ap.note}` : ""}
                </span>
              </p>
            ) : (
              <p className="hero-aperture" data-testid="aperture">
                <b>No aperture</b>
                <span>nothing opened when the {control} was thrown</span>
              </p>
            )}
            <p className="hero-sub">
              One cycle measured end to end:{" "}
              {cert.rest_is_closed
                ? `${control} on, door opens, ${control} off, door shuts.`
                : `${control} on, door shuts, ${control} off, door opens.`}{" "}
              Opening and closing are timed at the <b>doorway</b> — the tick the passage
              becomes walkable, and the tick the pattern is solid again — not the tick the
              machine finally goes quiet, which is reported separately as settle.
            </p>
            <div className="hero-dims">
              <span>
                bounds{" "}
                <b>
                  {dx} × {dy} × {dz}
                </b>{" "}
                blocks
              </span>
              <span>
                {control} at <b>({cert.lever.join(", ")})</b>
              </span>
              <span>
                {/* On a disputed door the hero carries no timings at all. The
                    numbers exist and are printed below, marked; up here, beside
                    the door's name, they would be read as the door's time. */}
                {disputed ? (
                  <>
                    doorway cycle <b>not certified</b>
                  </>
                ) : cycleTicks !== null ? (
                  <>
                    doorway cycle <b>{cycleTicks} ticks</b> · {secs(cycleTicks)} s in game
                  </>
                ) : (
                  <>
                    doorway cycle <b>not measured</b>
                  </>
                )}
              </span>
              {!disputed && settleCycle !== null && (
                <span>
                  settles after <b>{settleCycle} ticks</b>
                </span>
              )}
            </div>
            {/* First of the hero notes, and ahead of the aperture dispute on
                purpose: when both fire, this one is the CAUSE. A door whose
                comparators all read 0 is exactly the kind of door that settles
                into a different cycle than the one it was saved in, and a
                reader who meets "two doors, one file" first will go hunting
                through their build for a fault that is in the export. */}
            {missingBE && (
              <p className="hero-note note-alarm" data-testid="blockentity-audit">
                <span className="badge badge-alarm">incomplete file</span>{" "}
                <b>This file is missing block-entity data.</b> Every block is present and
                correct — which is why nothing else here complains — but{" "}
                {missingList(missingBE.missing)} arrived with no block entity attached
                {missingBE.present === 0
                  ? ", and the file carries none at all"
                  : `, out of ${plural(
                      missingBE.present,
                      "block entity",
                      "block entities",
                    )} it does carry`}
                . That is where a comparator keeps its output signal and a container keeps
                its contents, so this run simulated them as a comparator reading <b>0</b>{" "}
                and a box holding nothing. A door that resets on those numbers may not
                reset in game, and one that fails on them may run perfectly — the
                measurement is not wrong, it is of a different machine, so no verdict is
                stamped. Nothing here says your build is broken: several export tools drop
                block entities on conversion and pass every block through regardless.
                Re-export from the original save, or upload the file in the format it was
                built in, and this run can be certified.
              </p>
            )}
            {conflict && (
              <p className="hero-note note-alarm" data-testid="conflict">
                <span className="badge badge-alarm">no classification</span>{" "}
                <b>Two doors, one file.</b> In the state it was saved in, this build opens a{" "}
                <b>
                  {conflict.saved.w} × {conflict.saved.h}
                </b>{" "}
                doorway of {conflict.saved.cells} cells
                {conflict.saved.name ? ` — a ${conflict.saved.name}` : ""}. Run it to the cycle
                it actually repeats and the doorway is{" "}
                <b>
                  {conflict.settled.w} × {conflict.settled.h}
                </b>
                , {conflict.settled.cells} cells: {conflict.drift} cells never come back. Both
                are real measurements of the same machine and nothing in the file says which
                one is the door, so no pattern is named and no opening time is stamped. Every
                number below describes the settled cycle. Check the build for a part that only
                works from the saved state — a comparator reading a container, an unpowered
                loader — and upload it in the state it is meant to run in.
              </p>
            )}
            {cert.input_note && (
              <p className="hero-note" data-testid="input-note">
                <b>Input.</b> {cert.input_note}
              </p>
            )}
            {cert.timing_note && (
              <p className="hero-note">
                <b>Doorway timing incomplete.</b> {cert.timing_note}. The settle times below
                are still the machine's, but they are not opening times and are not quoted
                as such.
              </p>
            )}
            {negatives.map((n) => (
              <p className="hero-note note-flag" key={n.verb}>
                <span className="badge">negative reset</span> The lever can be thrown again{" "}
                <b>{tk(n.r.ticks!)}</b> after the {n.verb} click, while the{" "}
                {n.r.stroke_ticks}-tick {n.verb} stroke is still running — and the door
                still finishes it and returns to state. Rare.
              </p>
            ))}
            {!cert.rest_is_closed && (
              <p className="hero-note">
                <b>Saved open.</b> The doorway already stands clear in the file, so the
                first lever click closes it and the second opens it. Opening and closing
                below are named for what the door does, not for which click came first.
              </p>
            )}
            {cert.needed_priming && !conflict && (
              <p className="hero-note">
                <b>Saved mid-cycle.</b> The door did not return to its saved state, so it was
                run to its steady state first; timings are measured from there. The doorway
                measures the same either way, which is what makes the rest of this sheet safe
                to read.
              </p>
            )}
          </div>
          <Seal openTicks={cert.open_ticks} verdict={cert.verdict} />
        </div>
      </div>

      {cert.classification && (
        <div className="sheet-section">
          <ClassificationBlock cls={cert.classification} />
        </div>
      )}

      <div className="sheet-section">
        <p className="eyebrow">Exhibit A — the measured cycle</p>
        <MeshReplay
          replay={rec.replay}
          lever={cert.lever}
          xray={xray}
          geometry={cert.aperture_geometry ?? null}
          idle={eng?.dead?.cells ?? null}
        />
        <p className="exhibit-caption">
          Drawn from the recorded block changes. The two lime marks on the scrubber are the{" "}
          {control} throws; it starts on the one that{" "}
          {cert.rest_is_closed ? "opens" : "closes"} the door.{" "}
          {eng?.dead && eng.dead.idle > 0
            ? `Dead weight marks the ${fmt(eng.dead.idle)} blocks that neither moved nor took an update all cycle.`
            : ""}
        </p>
      </div>

      <div className="sheet-section">
        <p className="eyebrow">
          {conflict
            ? "Measurements — of the settled cycle only"
            : missingBE
              ? "Measurements — of an incomplete file"
              : "Measurements"}
        </p>
        {!conflict && missingBE && (
          /* Same reasoning as the conflict caveat below, different cause: the
             numbers are real, and they are of the wrong machine. Said again
             here because the tiles are what gets screenshotted, and a figure
             lifted off this sheet travels without the hero note. */
          <p className="tiles-refusal" data-testid="measurements-caveat">
            <b>Not certified, and the file is why.</b> Every figure below is a true
            measurement of what this file contains — but the file is missing{" "}
            {missingList(missingBE.missing)} worth of block-entity data, so the run timed a
            door whose comparators read 0 and whose containers are empty. Quote none of
            these as this build's numbers until the file is re-exported with its block
            entities intact.
          </p>
        )}
        {conflict && (
          /* The tiles are the most liftable numbers on the page. On a disputed
             door every one of them times a cycle the run cannot attribute to
             this door, so the refusal is set at reading size and each tile is
             marked where it is read, not only in the banner up in the hero. */
          <p className="tiles-refusal" data-testid="measurements-caveat">
            <b>Not certified, and these are the reason.</b> Every figure below was measured on
            the {conflict.settled.w} × {conflict.settled.h} cycle the machine settles into —
            one of the two doors in this file. The other is the{" "}
            {conflict.saved.w} × {conflict.saved.h} doorway it was saved with, and nothing in
            the file says which one is the door. Quote none of these as this build's opening
            time or rate; they are true of a cycle, not of a door.
          </p>
        )}
        <div className="tiles">
          <StatTile
            label="Opens in"
            value={cert.open_ticks ?? "—"}
            unit={cert.open_ticks !== null ? " ticks" : undefined}
            sub={strokeSub(cert.open_ticks, cert.open_settle_ticks, cert.open_latency)}
            disputed={disputed}
          />
          <StatTile
            label="Closes in"
            value={cert.close_ticks ?? "—"}
            unit={cert.close_ticks !== null ? " ticks" : undefined}
            sub={strokeSub(cert.close_ticks, cert.close_settle_ticks, cert.close_latency)}
            disputed={disputed}
          />
          <StatTile
            label="Settles at"
            value={settleCycle ?? "—"}
            unit={settleCycle !== null ? " ticks" : undefined}
            sub={
              settleCycle === null
                ? "the machine never went quiet"
                : `${cert.open_settle_ticks ?? "—"} opening + ${
                    cert.close_settle_ticks ?? "—"
                  } closing · when the last block stops`
            }
            disputed={disputed}
          />
          <StatTile
            label="Reset after opening"
            value={ro?.ticks ?? "—"}
            unit={ro?.ticks != null ? " ticks" : undefined}
            sub={resetSub(ro, "open")}
            disputed={disputed}
          />
          <StatTile
            label="Reset after closing"
            value={rc?.ticks ?? "—"}
            unit={rc?.ticks != null ? " ticks" : undefined}
            sub={resetSub(rc, "close")}
            disputed={disputed}
          />
          <StatTile
            label="Cycle rate"
            value={Math.round(cert.cycles_per_minute)}
            unit=" /min"
            sub={
              resetCycle !== null
                ? `rate-limited by reset: ${resetCycle} ticks between opens`
                : cycleTicks !== null
                  ? `doorway cycle of ${cycleTicks} ticks — reset not measured`
                  : `from settle: ${settleCycle ?? "—"} ticks`
            }
            disputed={disputed}
          />
          <StatTile
            label="Stroke mass"
            value={cert.moved_cells}
            unit=" cells"
            sub={`travel per stroke · ${movingBlocks} columns active`}
            disputed={disputed}
          />
          <StatTile
            label="Aperture cost"
            value={ap ? `${pct((ap.cells / cert.volume) * 100)}%` : "—"}
            sub={
              ap
                ? `${ap.cells}-cell doorway in a ${cert.volume}-cell build`
                : "no doorway measured"
            }
            disputed={disputed}
          />
        </div>
      </div>

      {eng && (
        <EngineeringSection
          eng={eng}
          census={cert.census}
          peakChanges={cert.peak_changes}
          peakTick={cert.peak_tick}
        />
      )}

      <div className="sheet-section chart-block">
        <p className="eyebrow">Activity trace</p>
        <h3>Events per tick of the cycle</h3>
        <p className="chart-note">
          Stacked, so the height is everything happening on that tick. One burst per lever
          click; the marker under the axis is where the stroke actually finishes, and shaded
          columns are ticks with nothing at all.
        </p>
        <ActivityChart events={cert.events_per_tick} flips={rec.replay.flips} />
      </div>

      <div className="sheet-section chart-block">
        <p className="eyebrow">Bill of materials</p>
        <h3>
          {totalBlocks} blocks · {stacks} {stacks === 1 ? "stack" : "stacks"}
        </h3>
        <p className="chart-note">
          What to bring, counted at rest and most-used first.{" "}
          {stackMath(totalBlocks) ? `Gather ${stackMath(totalBlocks)}.` : ""}
        </p>
        <MaterialsGrid materials={cert.materials} />
      </div>

      <div className="sheet-section">
        <div className="split">
          <div className="chart-block">
            <p className="eyebrow">Change footprint</p>
            <h3>Block changes per column</h3>
            <p className="chart-note">
              The door's XY silhouette; the stronger the blue, the more a column churned.
            </p>
            <Heatmap heatmap={cert.heatmap} lever={cert.lever} />
          </div>
          {rows.length > 0 && (
            <div className="chart-block">
              <p className="eyebrow">Mechanism</p>
              <h3>What moves it</h3>
              <p className="chart-note">
                The working parts, counted from the door at rest.
              </p>
              <dl className="census" data-testid="census">
                {rows.map((r) => (
                  <div key={r.term}>
                    <dt>{r.term}</dt>
                    {r.detail ? <dd>{r.detail}</dd> : <dd aria-hidden />}
                  </div>
                ))}
              </dl>
            </div>
          )}
        </div>
      </div>

      <div className="sheet-foot">
        <span>schemat.io · door validator — simulated in this browser</span>
        <span>
          seed {cert.seed} ·{" "}
          <a href={certUrl} download={`${cert.name}.certificate.json`}>
            download certificate JSON
          </a>
        </span>
      </div>
    </div>
  );
}
