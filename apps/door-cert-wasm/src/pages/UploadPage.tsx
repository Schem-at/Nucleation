import { useCallback, useEffect, useRef, useState } from "react";
import { useNavigate } from "react-router-dom";
import { newId, saveRecord, saveXray } from "../lib/store";
import type { WorkerJob, WorkerMessage } from "../lib/types";

const ACCEPT = [".litematic", ".schem", ".schematic", ".snbt"];
const STEPS = [
  { key: "engine", label: "Waking the wasm engine" },
  { key: "parsing", label: "Parsing schematic" },
  { key: "simulating", label: "Simulating redstone" },
  { key: "input", label: "Finding what drives it" },
  { key: "measuring", label: "Measuring open / close" },
  { key: "classifying", label: "Classifying the pattern" },
  { key: "reset", label: "Measuring reset time" },
  { key: "x-ray", label: "Recording the update stream" },
];

/** What the upload screen shows when a run stops: a sentence for the person,
 *  and the engine's own words kept as small print so a screenshot is still
 *  enough for us to triage from. */
type UploadError = { message: string; code?: string | null; detail?: string | null };

export function UploadPage() {
  const navigate = useNavigate();
  const [drag, setDrag] = useState(false);
  const [error, setError] = useState<UploadError | null>(null);
  const [job, setJob] = useState<{ file: string; step: string } | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const workerRef = useRef<Worker | null>(null);

  useEffect(() => () => workerRef.current?.terminate(), []);

  const submit = useCallback(
    async (file: File) => {
      setError(null);
      const ext = "." + (file.name.split(".").pop() ?? "").toLowerCase();
      if (!ACCEPT.includes(ext)) {
        setError({
          message: `"${file.name}" is not a format this reads. Upload ${ACCEPT.join(", ")}.`,
        });
        return;
      }
      const buffer = await file.arrayBuffer();
      const name = file.name.replace(/\.[^.]+$/, "");
      setJob({ file: file.name, step: "engine" });

      workerRef.current?.terminate();
      const worker = new Worker(new URL("../worker/certify.worker.ts", import.meta.url), {
        type: "module",
      });
      workerRef.current = worker;
      worker.onmessage = ({ data }: MessageEvent<WorkerMessage>) => {
        if (data.type === "progress") {
          setJob((j) => (j ? { ...j, step: data.step } : j));
        } else if (data.type === "done") {
          const id = newId();
          saveRecord(id, data.record);
          if (data.xray) saveXray(id, data.xray);
          worker.terminate();
          navigate(`/door/${id}`);
        } else {
          setError({
            message:
              data.error ||
              "The run stopped without saying why. Reload the page and try the file again.",
            code: data.code,
            detail: data.detail,
          });
          setJob(null);
          worker.terminate();
        }
      };
      worker.onerror = (e) => {
        setError({
          message:
            "The wasm engine crashed partway through. Reload the page before trying another door — " +
            "a crashed engine stays crashed for the rest of the session.",
          code: e.message || "worker error",
        });
        setJob(null);
        worker.terminate();
      };
      const payload: WorkerJob = { name, ext, buffer };
      worker.postMessage(payload, [buffer]);
    },
    [navigate],
  );

  const activeIdx = job ? STEPS.findIndex((s) => s.key === job.step) : -1;

  return (
    <div className="upload-wrap">
      <div className="upload-hero">
        <p className="eyebrow">Piston door validator</p>

      </div>

      {!job && (
        <>
          <div
            className={"dropzone" + (drag ? " drag" : "")}
            role="button"
            tabIndex={0}
            aria-label="Choose a schematic file"
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
            <div className="upload-error" role="alert" data-testid="upload-error">
              <p className="upload-error-msg">{error.message}</p>
              {(error.code || error.detail) && (
                <p className="upload-error-code" data-testid="upload-error-code">
                  {[error.code, error.detail].filter(Boolean).join(" · ")}
                </p>
              )}
            </div>
          )}
        </>
      )}

      {job && (
        <div className="steps" aria-live="polite">
          <div className="steps-head">Certifying {job.file}</div>
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
