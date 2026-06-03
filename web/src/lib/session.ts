// Auth session signal. Token lives in localStorage; user identity is cached
// for instant boot. App state (transactions, budgets, etc) is NEVER cached.

import { createSignal } from "solid-js";
import { getCachedUser, setCachedUser, setToken, type CachedUser } from "../api/client";
import { auth } from "../api";

const [user, setUserSignal] = createSignal<CachedUser | null>(getCachedUser());

export const currentUser = user;

export function setSession(token: string, u: CachedUser): void {
  setToken(token);
  setCachedUser(u);
  setUserSignal(u);
}

/** Update the cached user record (e.g. after editing profile or uploading a photo). */
export function setUser(u: CachedUser | null): void {
  setCachedUser(u);
  setUserSignal(u);
}

export async function logoutSession(): Promise<void> {
  try { await auth.logout(); } catch { /* token gets cleared either way */ }
  setToken(null);
  setCachedUser(null);
  setUserSignal(null);
}

export function clearSessionLocal(): void {
  setToken(null);
  setCachedUser(null);
  setUserSignal(null);
}
