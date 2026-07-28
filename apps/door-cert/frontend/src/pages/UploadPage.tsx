import { useCallback, useEffect, useRef, useState } from "react";
import { useNavigate } from "react-router-dom";
import { getStatus, uploadDoor } from "../lib/api";

const ACCEPT = [".litematic", ".schem", ".schematic", ".snbt"];
const STEPS = [
  { key: "parsing", label: "Parsing schematic" },
  { key: "simulating", label: "Simulating redstone" },
  { key: "measuring", label: "Measuring open / close" },
  { key: "rendering", label: "Rendering animation" },
];

export function UploadPage() {
  const navigate = useNavigate();
  const [drag, setDrag] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [job, setJob] = useState<{ id: string; file: string; step: string | null } | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const pollRef = useRef<number>();

  const submit = useCallback(
    async (file: File) => {
      setError(null);
      const ext = "." + (file.name.split(".").pop() ?? "").toLowerCase();
      if (!ACCEPT.includes(ext)) {
        setError(`"${file.name}" is not a supported format. Upload ${ACCEPT.join(", ")}.`);
        return;
      }
      try {
        const { id } = await uploadDoor(file);
        setJob({ id, file: file.name, step: "parsing" });
      } catch (e) {
        setError(`Upload failed (${e instanceof Error ? e.message : "network error"}). Try again.`);
      }
    },
    [],
  );

  useEffect(() => {
    if (!job) return;
    pollRef.current = window.setInterval(async () => {
      try {
        const s = await getStatus(job.id);
        if (s.status === "done") {
          window.clearInterval(pollRef.current);
          navigate(`/door/${job.id}`);
        } else if (s.status === "error") {
          window.clearInterval(pollRef.current);
          setError(s.error ?? "Simulation failed. Check that the schematic contains a lever.");
          setJob(null);
        } else {
          setJob((j) => (j ? { ...j, step: s.step } : j));
        }
      } catch {
        /* keep polling; transient */
      }
    }, 450);
    return () => window.clearInterval(pollRef.current);
  }, [job?.id, navigate]);

  const activeIdx = job ? STEPS.findIndex((s) => s.key === job.step) : -1;

  return (
    <div className="upload-wrap">
      <div className="upload-hero">
        <p className="eyebrow">Piston door performance certification</p>
        <h1>
          Your door, <em>measured</em>.
        </h1>
        <p>
          Upload a piston door with its lever. We simulate it headless, time every tick from
          lever flip to full open, and issue a shareable certificate.
        </p>
      </div>

      {!job && (
        <>
          <div
            className={"dropzone" + (drag ? " drag" : "")}
            role="button"
            tabIndex={0}
            aria-label="Upload a schematic file"
            onClick={() => inputRef.current?.click()}
            onKeyDown={(e) => e.key === "Enter" && inputRef.current?.click()}
            onDragOver={(e) => {
              e.preventDefault();
              setDrag(true);
            }}
            onDragLeave={() => setDrag(false)}
            onDrop={(e) => {
              e.preventDefault();
              setDrag(false);
              const f = e.dataTransfer.files?.[0];
              if (f) void submit(f);
            }}
          >
            <div className="dropzone-glyph" aria-hidden>
              <svg width="44" height="44" viewBox="0 0 44 44">
                {/* voxel door glyph */}
                {[
                  [12, 8], [20, 8], [28, 8],
                  [12, 16], [28, 16],
                  [12, 24], [28, 24],
                  [12, 32], [20, 32], [28, 32],
                ].map(([x, y]) => (
                  <rect key={`${x}${y}`} x={x} y={y} width={6} height={6} rx={1} fill="var(--baseline)" />
                ))}
                <rect x={20} y={16} width={6} height={14} rx={1} fill="var(--seal)" />
              </svg>
            </div>
            <p className="dropzone-title">Drop a schematic here, or click to browse</p>
            <p className="dropzone-formats">.litematic · .schem · .schematic · .snbt</p>
            <input
              ref={inputRef}
              type="file"
              accept={ACCEPT.join(",")}
              onChange={(e) => {
                const f = e.target.files?.[0];
                if (f) void submit(f);
                e.target.value = "";
              }}
            />
          </div>
          {error && (
            <div className="upload-error" role="alert">
              {error}
            </div>
          )}
          <p className="bench-note">simulated at 20 tps · deterministic seed · nucleation engine</p>
        </>
      )}

      {job && (
        <div className="steps" aria-live="polite">
          <div className="steps-head">
            Certifying {job.file} · job {job.id}
          </div>
          {STEPS.map((s, i) => {
            const state = i < activeIdx ? "done" : i === activeIdx ? "active" : "todo";
            return (
              <div key={s.key} className={`step ${state}`}>
                <i className="dot" aria-hidden />
                {s.label}
                <span className="step-ms">
                  {state === "done" ? "ok" : state === "active" ? "running" : "queued"}
                </span>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
