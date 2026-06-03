// Bearer-token HTTP client. Same-origin /api/v1.
//
// Token trade-off: we keep the bearer token in localStorage (key `owo.token`).
// This is exposed to XSS — accepted for the self-hosted single-user MVP and
// documented in specs/phases/D-web-spa.md. The alternative — httpOnly cookies
// set by the login endpoint — needs a backend change. App state is NEVER
// cached in localStorage; only the token + cached user id/display name.

import type { ApiErrorBody } from "./types";

const TOKEN_KEY = "owo.token";
const USER_KEY = "owo.user";

export interface CachedUser {
  id: string;
  email: string;
  display_name: string;
  default_currency: string;
  locale: string;
  partner_name?: string | null;
  photo_url?: string | null;
}

export function getToken(): string | null {
  try { return localStorage.getItem(TOKEN_KEY); } catch { return null; }
}
export function setToken(t: string | null): void {
  try {
    if (t) localStorage.setItem(TOKEN_KEY, t);
    else localStorage.removeItem(TOKEN_KEY);
  } catch { /* ignore */ }
}
export function getCachedUser(): CachedUser | null {
  try {
    const raw = localStorage.getItem(USER_KEY);
    return raw ? JSON.parse(raw) as CachedUser : null;
  } catch { return null; }
}
export function setCachedUser(u: CachedUser | null): void {
  try {
    if (u) localStorage.setItem(USER_KEY, JSON.stringify(u));
    else localStorage.removeItem(USER_KEY);
  } catch { /* ignore */ }
}

export class ApiError extends Error {
  constructor(
    public status: number,
    public code: string,
    message: string,
    public details?: unknown,
  ) {
    super(message);
  }
}

export interface RequestOptions {
  method?: string;
  query?: Record<string, string | number | undefined | null>;
  body?: unknown;
  headers?: Record<string, string>;
  signal?: AbortSignal;
}

function buildUrl(path: string, query?: RequestOptions["query"]): string {
  const base = path.startsWith("/api/") ? path : `/api/v1${path}`;
  if (!query) return base;
  const params = new URLSearchParams();
  for (const [k, v] of Object.entries(query)) {
    if (v === undefined || v === null || v === "") continue;
    params.append(k, String(v));
  }
  const qs = params.toString();
  return qs ? `${base}?${qs}` : base;
}

export async function request<T>(path: string, opts: RequestOptions = {}): Promise<T> {
  const url = buildUrl(path, opts.query);
  const headers: Record<string, string> = {
    "Accept": "application/json",
    ...opts.headers,
  };
  const token = getToken();
  if (token) headers["Authorization"] = `Bearer ${token}`;
  if (opts.body !== undefined) headers["Content-Type"] = "application/json";

  const res = await fetch(url, {
    method: opts.method ?? (opts.body !== undefined ? "POST" : "GET"),
    headers,
    body: opts.body !== undefined ? JSON.stringify(opts.body) : undefined,
    signal: opts.signal,
  });

  if (res.status === 204) return undefined as T;

  const text = await res.text();
  const data: unknown = text ? safeJson(text) : undefined;

  if (!res.ok) {
    const errBody = data as ApiErrorBody | undefined;
    const code = errBody?.error?.code ?? `HTTP_${res.status}`;
    const msg = errBody?.error?.message ?? res.statusText ?? "Request failed";
    if (res.status === 401) {
      setToken(null);
      setCachedUser(null);
    }
    throw new ApiError(res.status, code, msg, errBody?.error?.details);
  }
  return data as T;
}

function safeJson(s: string): unknown {
  try { return JSON.parse(s); } catch { return s; }
}

/** Best-effort device id for the auth session row. Generated once per browser. */
export function deviceInfo(): { device_id: string; device_name: string } {
  const KEY = "owo.device_id";
  let id: string;
  try {
    id = localStorage.getItem(KEY) ?? "";
    if (!id) {
      id = crypto.randomUUID();
      localStorage.setItem(KEY, id);
    }
  } catch {
    id = crypto.randomUUID();
  }
  return { device_id: id, device_name: "web" };
}
