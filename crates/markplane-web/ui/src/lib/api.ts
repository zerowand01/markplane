const API_BASE = typeof window !== "undefined" ? "" : "http://localhost:4200";

export async function fetcher<T>(path: string): Promise<T> {
  const res = await fetch(`${API_BASE}${path}`);
  if (!res.ok) {
    const error = await res.json().catch(() => ({ error: { message: res.statusText } }));
    throw new Error(error.error?.message || `API error: ${res.status}`);
  }
  const json = await res.json();
  return json.data;
}

export async function fetchList<T>(path: string): Promise<{ data: T[]; total: number }> {
  const res = await fetch(`${API_BASE}${path}`);
  if (!res.ok) {
    const error = await res.json().catch(() => ({ error: { message: res.statusText } }));
    throw new Error(error.error?.message || `API error: ${res.status}`);
  }
  const json = await res.json();
  return { data: json.data, total: json.meta.total };
}

export async function postAction<T>(path: string): Promise<T> {
  const res = await fetch(`${API_BASE}${path}`, { method: "POST" });
  if (!res.ok) {
    const error = await res.json().catch(() => ({ error: { message: res.statusText } }));
    throw new Error(error.error?.message || `API error: ${res.status}`);
  }
  const json = await res.json();
  return json.data;
}

export async function postJson<T>(path: string, body: unknown): Promise<T> {
  const res = await fetch(`${API_BASE}${path}`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
  if (!res.ok) {
    const error = await res.json().catch(() => ({ error: { message: res.statusText } }));
    throw new Error(error.error?.message || `API error: ${res.status}`);
  }
  const json = await res.json();
  return json.data;
}

export async function patchJson<T>(path: string, body: unknown): Promise<T> {
  const res = await fetch(`${API_BASE}${path}`, {
    method: "PATCH",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
  if (!res.ok) {
    const error = await res.json().catch(() => ({ error: { message: res.statusText } }));
    throw new Error(error.error?.message || `API error: ${res.status}`);
  }
  const json = await res.json();
  return json.data;
}

export async function deleteAction<T>(path: string): Promise<T> {
  const res = await fetch(`${API_BASE}${path}`, { method: "DELETE" });
  if (!res.ok) {
    const error = await res.json().catch(() => ({ error: { message: res.statusText } }));
    throw new Error(error.error?.message || `API error: ${res.status}`);
  }
  const json = await res.json();
  return json.data;
}
