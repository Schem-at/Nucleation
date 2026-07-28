export interface RunConfigInput {
  population: number;
  generations: number;
  mutation_rate: number;
  bbox: [number, number, number];
  eval_ticks: number;
  seed: number;
}

export interface Block {
  x: number;
  y: number;
  z: number;
  state: string;
}

export interface LeaderboardEntry {
  id: string;
  name?: string;
  fitness: number;
  gen: number;
  blocks: Block[];
}

export interface HistoryPoint {
  gen: number;
  best: number;
  mean: number;
}

export interface RunState {
  status: "running" | "done";
  generation: number;
  total_generations?: number;
  history: HistoryPoint[];
  evals_per_sec: number;
  leaderboard: LeaderboardEntry[];
}

export async function createRun(cfg: RunConfigInput): Promise<{ id: string }> {
  const res = await fetch("/api/runs", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(cfg),
  });
  if (!res.ok) throw new Error(`create run failed: ${res.status}`);
  return res.json();
}

export async function getRun(id: string): Promise<RunState> {
  const res = await fetch(`/api/runs/${id}`);
  if (!res.ok) throw new Error(`fetch run failed: ${res.status}`);
  return res.json();
}

export async function stopRun(id: string): Promise<void> {
  const res = await fetch(`/api/runs/${id}/stop`, { method: "POST" });
  if (!res.ok) throw new Error(`stop run failed: ${res.status}`);
}
