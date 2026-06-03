import { createSignal, For, Show, type JSX } from "solid-js";
import type { Bill, BillEditScope } from "../api/types";
import { bills } from "../api";
import { centsToApi, inputToApi, parseCents } from "../lib/money";
import { useI18n } from "../lib/i18n";
import { pushToast } from "./Toast";
import { ApiError } from "../api/client";

interface Props {
  bill: Bill;
  month: string;
  accounts: Array<{ id: string; name: string }>;
  categories: Array<{ id: string; name: string }>;
  onSaved: () => void;
  onCancel: () => void;
}

export function BillEditModal(props: Props): JSX.Element {
  const { t } = useI18n();
  const [description, setDescription] = createSignal(props.bill.description);
  const [valueInput, setValueInput] = createSignal(
    centsToApi(parseCents(props.bill.value)).replace(".", ","),
  );
  const [dueDay, setDueDay] = createSignal<number>(props.bill.due_day);
  const [accountId, setAccountId] = createSignal<string>(props.bill.account_id ?? "");
  const [categoryId, setCategoryId] = createSignal<string>(props.bill.category_id ?? "");
  const [scope, setScope] = createSignal<BillEditScope>("this_month");
  const [busy, setBusy] = createSignal(false);
  const [err, setErr] = createSignal<string | null>(null);

  async function save(e: Event): Promise<void> {
    e.preventDefault();
    setErr(null);
    const valueStr = inputToApi(valueInput());
    const day = Math.min(31, Math.max(1, Math.floor(dueDay())));
    setBusy(true);
    try {
      await bills.update(props.bill.id, {
        description: description().trim() || undefined,
        value: valueStr,
        due_day: day,
        account_id: accountId() || null,
        category_id: categoryId() || null,
        scope: scope(),
        month: props.month,
      });
      pushToast(t("bills.paidToast", { name: description() }), "ok");
      props.onSaved();
    } catch (e2) {
      setErr(e2 instanceof ApiError ? `${e2.code}: ${e2.message}` : "error");
    } finally {
      setBusy(false);
    }
  }

  return (
    <>
      <button
        type="button"
        class="fixed inset-0 z-40 bg-black/40 backdrop-blur-[2px] enter-fade"
        aria-label={t("common.cancel")}
        onClick={props.onCancel}
      />
      <div
        role="dialog"
        aria-modal="true"
        class="fixed left-1/2 top-1/2 z-50 w-[min(92vw,520px)] max-h-[88vh] -translate-x-1/2 -translate-y-1/2 overflow-y-auto card p-5 shadow-[var(--shadow-sheet)] enter"
      >
        <h3 class="h-title">{t("bills.editTitle")}</h3>

        <form class="mt-4 flex flex-col gap-4" onSubmit={save} novalidate>
          <label>
            <span class="label">{t("bills.description")}</span>
            <input
              class="field"
              value={description()}
              onInput={(e) => setDescription(e.currentTarget.value)}
            />
          </label>

          <div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
            <label>
              <span class="label">{t("bills.value")}</span>
              <input
                class="field tabular"
                type="text"
                inputmode="decimal"
                value={valueInput()}
                onInput={(e) => setValueInput(e.currentTarget.value)}
              />
            </label>
            <label>
              <span class="label">{t("bills.dueDay")}</span>
              <input
                class="field tabular"
                type="number"
                min="1"
                max="31"
                inputmode="numeric"
                value={dueDay()}
                onInput={(e) => setDueDay(parseInt(e.currentTarget.value || "1", 10))}
              />
            </label>
          </div>

          <Show when={props.accounts.length > 0}>
            <label>
              <span class="label">{t("bills.account")}</span>
              <select
                class="field"
                value={accountId()}
                onChange={(e) => setAccountId(e.currentTarget.value)}
              >
                <For each={props.accounts}>{(a) => <option value={a.id}>{a.name}</option>}</For>
              </select>
            </label>
          </Show>

          <Show when={props.categories.length > 0}>
            <label>
              <span class="label">{t("bills.category")}</span>
              <select
                class="field"
                value={categoryId()}
                onChange={(e) => setCategoryId(e.currentTarget.value)}
              >
                <For each={props.categories}>{(c) => <option value={c.id}>{c.name}</option>}</For>
              </select>
            </label>
          </Show>

          <fieldset class="flex flex-col gap-2">
            <legend class="label">{t("bills.scope")}</legend>
            <ScopeOption value="this_month" current={scope()} onSelect={setScope} label={t("bills.scopeThisMonth")} />
            <ScopeOption value="this_and_next" current={scope()} onSelect={setScope} label={t("bills.scopeThisAndNext")} />
            <ScopeOption value="all" current={scope()} onSelect={setScope} label={t("bills.scopeAll")} />
            <Show when={scope() === "all"}>
              <p class="meta rounded-md px-3 py-2 text-[color:var(--color-warning-fg)]" style={{ background: "var(--color-warning-bg)" }}>
                ⚠ {t("bills.scopeAllWarn")}
              </p>
            </Show>
          </fieldset>

          <Show when={err()}>{(m) => <p class="meta text-[color:var(--color-danger-fg)]">{m()}</p>}</Show>

          <div class="flex gap-3 pt-2">
            <button type="button" class="btn btn-ghost flex-1" onClick={props.onCancel}>
              {t("common.cancel")}
            </button>
            <button type="submit" class="btn btn-primary flex-[2]" disabled={busy()}>
              {busy() ? t("addTransaction.saving") : t("bills.saveEdit")}
            </button>
          </div>
        </form>
      </div>
    </>
  );
}

function ScopeOption(props: {
  value: BillEditScope;
  current: BillEditScope;
  onSelect: (v: BillEditScope) => void;
  label: string;
}): JSX.Element {
  const active = (): boolean => props.value === props.current;
  return (
    <button
      type="button"
      class="flex items-center gap-3 rounded-lg border px-3 py-2.5 text-left transition"
      classList={{
        "border-[color:var(--color-primary-600)] bg-[color:var(--color-primary-50)]": active(),
        "border-[color:var(--color-surface-sunken)] hover:bg-[color:var(--color-surface-muted)]": !active(),
      }}
      onClick={() => props.onSelect(props.value)}
      aria-pressed={active()}
    >
      <span
        class="inline-grid h-5 w-5 place-items-center rounded-full border-2"
        classList={{
          "border-[color:var(--color-primary-600)]": active(),
          "border-[color:var(--color-surface-sunken)]": !active(),
        }}
      >
        <Show when={active()}>
          <span class="h-2.5 w-2.5 rounded-full bg-[color:var(--color-primary-600)]" />
        </Show>
      </span>
      <span class="body-strong">{props.label}</span>
    </button>
  );
}
