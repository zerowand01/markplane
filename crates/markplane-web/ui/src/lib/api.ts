import { DATA_VERSION_MISMATCH, setDataVersionMismatch } from "@/lib/data-version";

const API_BASE = typeof window !== "undefined" ? "" : "http://localhost:4200";

/** Error thrown for any non-2xx API response, carrying the server's error code. */
export class ApiError extends Error {
  readonly status: number;
  readonly code: string | null;

  constructor(message: string, status: number, code: string | null) {
    super(message);
    this.name = "ApiError";
    this.status = status;
    this.code = code;
  }

  get isDataVersionMismatch(): boolean {
    return this.code === DATA_VERSION_MISMATCH;
  }
}

export function isDataVersionMismatch(error: unknown): boolean {
  return error instanceof ApiError && error.isDataVersionMismatch;
}

/** Success envelope: `data` always, `meta` on list endpoints. */
interface Envelope {
  data: unknown;
  meta: { total: number };
}

/** Pull `{ message, code }` out of an error body, tolerating non-object JSON. */
function parseErrorBody(body: unknown): { message?: string; code?: string } {
  if (typeof body !== "object" || body === null) return {};
  const error = (body as { error?: unknown }).error;
  if (typeof error !== "object" || error === null) return {};
  const { message, code } = error as { message?: unknown; code?: unknown };
  return {
    message: typeof message === "string" ? message : undefined,
    code: typeof code === "string" ? code : undefined,
  };
}

/**
 * Perform a request against the API, unwrap the `{ data, meta? }` envelope on
 * success, and throw an `ApiError` on failure. Every API call funnels through
 * here so the data version banner tracks the server's state without each hook
 * having to know about it.
 */
async function request(path: string, init?: RequestInit): Promise<Envelope> {
  const res = await fetch(`${API_BASE}${path}`, init);
  if (!res.ok) {
    const body = parseErrorBody(await res.json().catch(() => null));
    const error = new ApiError(
      body.message || res.statusText || `API error: ${res.status}`,
      res.status,
      body.code ?? null
    );
    if (error.isDataVersionMismatch) {
      setDataVersionMismatch(error.message);
    }
    throw error;
  }
  // Any successful response proves the mismatch is resolved (for example the
  // user restarted `markplane serve` with an upgraded binary). The banner
  // component refetches everything when this flips back to null.
  setDataVersionMismatch(null);
  return res.json();
}

export async function fetcher<T>(path: string): Promise<T> {
  const json = await request(path);
  return json.data as T;
}

export async function fetchList<T>(path: string): Promise<{ data: T[]; total: number }> {
  const json = await request(path);
  return { data: json.data as T[], total: json.meta.total };
}

export async function postAction<T>(path: string): Promise<T> {
  const json = await request(path, { method: "POST" });
  return json.data as T;
}

export async function postJson<T>(path: string, body: unknown): Promise<T> {
  const json = await request(path, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
  return json.data as T;
}

export async function patchJson<T>(path: string, body: unknown): Promise<T> {
  const json = await request(path, {
    method: "PATCH",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
  return json.data as T;
}

export async function deleteAction<T>(path: string): Promise<T> {
  const json = await request(path, { method: "DELETE" });
  return json.data as T;
}
