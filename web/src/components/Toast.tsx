// Minimal global toast queue. No external deps.

import { createSignal, For, Show } from "solid-js";
import type { JSX } from "solid-js";

interface Toast {
  id: number;
  kind: "info" | "error" | "ok";
  message: string;
}

const [items, setItems] = createSignal<Toast[]>([]);
let nextId = 1;

export function pushToast(message: string, kind: Toast["kind"] = "info"): void {
  const id = nextId++;
  setItems((arr) => [...arr, { id, message, kind }]);
  setTimeout(() => setItems((arr) => arr.filter((t) => t.id !== id)), 4200);
}

export function ToastHost(): JSX.Element {
  return (
    <div class="pointer-events-none fixed inset-x-0 bottom-[96px] z-50 flex flex-col items-center gap-2 px-3 safe-bottom md:bottom-6">
      <For each={items()}>
        {(t) => (
          <div
            class="pointer-events-auto w-full max-w-sm rounded-xl px-4 py-3 text-sm shadow-[var(--shadow-card)] enter"
            classList={{
              "bg-[color:var(--color-neutral-800)] text-[color:var(--color-text-on-primary)]": t.kind === "info",
              "bg-[color:var(--color-danger-bg)] text-[color:var(--color-danger-fg)]": t.kind === "error",
              "bg-[color:var(--color-success-bg)] text-[color:var(--color-success-fg)]": t.kind === "ok",
            }}
            role={t.kind === "error" ? "alert" : "status"}
          >
            {t.message}
          </div>
        )}
      </For>
      <Show when={false}>{null}</Show>
    </div>
  );
}
