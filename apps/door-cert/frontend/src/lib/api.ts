import type { Certificate, DoorStatus } from "./types";

async function ok<T>(res: Response): Promise<T> {
  if (!res.ok) throw new Error(`${res.status} ${res.statusText}`);
  return res.json() as Promise<T>;
}

export async function uploadDoor(file: File): Promise<{ id: string }> {
  const form = new FormData();
  form.append("file", file);
  return ok(await fetch("/api/doors", { method: "POST", body: form }));
}

export async function getStatus(id: string): Promise<DoorStatus> {
  return ok(await fetch(`/api/doors/${id}`));
}

export async function getCertificate(id: string): Promise<Certificate> {
  return ok(await fetch(`/api/doors/${id}/certificate`));
}
