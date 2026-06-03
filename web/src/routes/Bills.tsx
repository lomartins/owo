import { createMemo, createResource, createSignal, For, Show, type JSX } from "solid-js";
import { accounts, bills, categories } from "../api";
import { useI18n } from "../lib/i18n";
import { useMonth } from "../lib/useMonth";
import { centsToApi, formatMoney, inputToApi, parseCents } from "../lib/money";
import { EmptyState } from "../components/EmptyState";
import { pushToast } from "../components/Toast";
import { BillEditModal } from "../components/BillEditModal";
import { ApiError } from "../api/client";
import type { Bill } from "../api/types";
import { localizeCategoryName, isSeedName } from "../lib/categories";

export default function BillsView(): JSX.Element {
  const { t, lang } = useI18n();
  const { month, currency, locale, bumpRefresh } = useMonth();
  const localize = (name: string): string =>
    isSeedName(name) ? localizeCategoryName(name, lang()) : name;

  const [list, { refetch }] = createResource(
    () => month(),
    (m) => bills.list(m),
  );
  const [accs] = createResource(() => accounts.list());
  const [cats] = createResource(() => categories.list());
  const [showForm, setShowForm] = createSignal(false);
  const [editingBill, setEditingBill] = createSignal<Bill | null>(null);

  const userAccounts = () => (accs() ?? []).filter((a) => a.type === "asset" && !a.archived);
  const expenseCats = () => (cats() ?? []).filter((c) => c.kind !== "INCOME" && !c.archived);

  const totalEstimated = createMemo((): bigint =>
    (list() ?? []).reduce((sum, b) => sum + parseCents(b.value), 0n),
  );
  const totalPaid = createMemo((): bigint =>
    (list() ?? []).filter((b) => b.paid).reduce((sum, b) => sum + parseCents(b.value), 0n),
  );

  void locale;

  return (
    <div class="mx-auto w-full max-w-3xl space-y-4 px-4 py-5">
      <header class="flex flex-wrap items-end justify-between gap-3">
        <div class="min-w-0">
          <h2 class="h-title">{t("bills.title")}</h2>
          <p class="meta mt-1 tabular">
            {formatMoney(totalPaid(), currency(), locale())} / {formatMoney(totalEstimated(), currency(), locale())}
          </p>
        </div>
        <button class="btn btn-primary" onClick={() => setShowForm((s) => !s)}>
          <span class="material-symbols-rounded" style={{ "font-size": "20px" }}>add</span>
          {t("bills.addCta")}
        </button>
      </header>

      <Show when={showForm()}>
        <BillForm
          defaultCurrency={currency()}
          accounts={userAccounts()}
          categories={expenseCats().map((c) => ({ id: c.id, name: localize(c.name) }))}
          onCreated={() => {
            setShowForm(false);
            refetch();
            bumpRefresh();
          }}
        />
      </Show>

      <Show
        when={!list.loading}
        fallback={<div class="card h-32 pulse-soft bg-[color:var(--color-surface-muted)]" />}
      >
        <Show
          when={(list() ?? []).length > 0}
          fallback={
            <EmptyState
              icon="event_repeat"
              title={t("bills.empty")}
              hint={t("bills.emptyHint")}
              action={<button class="btn btn-primary" onClick={() => setShowForm(true)}>{t("bills.addCta")}</button>}
            />
          }
        >
          <ul class="card divide-y divide-[color:var(--color-surface-sunken)] stagger-enter">
            <For each={list() ?? []}>
              {(b) => (
                <BillRow
                  bill={b}
                  month={month()}
                  accounts={userAccounts()}
                  currency={currency()}
                  locale={locale()}
                  onChanged={() => {
                    refetch();
                    bumpRefresh();
                  }}
                  onEdit={() => setEditingBill(b)}
                />
              )}
            </For>
          </ul>
        </Show>
      </Show>

      <Show when={editingBill()}>
        {(bill) => (
          <BillEditModal
            bill={bill()}
            month={month()}
            accounts={userAccounts().map((a) => ({ id: a.id, name: a.name }))}
            categories={expenseCats().map((c) => ({ id: c.id, name: localize(c.name) }))}
            onSaved={() => {
              setEditingBill(null);
              refetch();
              bumpRefresh();
            }}
            onCancel={() => setEditingBill(null)}
          />
        )}
      </Show>
    </div>
  );
}

function BillForm(props: {
  defaultCurrency: string;
  accounts: Array<{ id: string; name: string }>;
  categories: Array<{ id: string; name: string }>;
  onCreated: () => void;
}): JSX.Element {
  const { t } = useI18n();
  const [description, setDescription] = createSignal("");
  const [value, setValue] = createSignal("");
  const [dueDay, setDueDay] = createSignal<number>(5);
  const [accountId, setAccountId] = createSignal<string>(props.accounts[0]?.id ?? "");
  const [categoryId, setCategoryId] = createSignal<string>(props.categories[0]?.id ?? "");
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  async function submit(e: Event): Promise<void> {
    e.preventDefault();
    setError(null);
    if (!description().trim() || !value().trim()) {
      setError(t("addTransaction.amountRequired"));
      return;
    }
    const day = Math.min(31, Math.max(1, Math.floor(dueDay())));
    setBusy(true);
    try {
      await bills.create({
        description: description().trim(),
        value: inputToApi(value()),
        currency: props.defaultCurrency,
        due_day: day,
        account_id: accountId() || null,
        category_id: categoryId() || null,
      });
      pushToast(`${description()} ✓`, "ok");
      props.onCreated();
    } catch (err) {
      setError(err instanceof ApiError ? `${err.code}: ${err.message}` : "Error");
    } finally {
      setBusy(false);
    }
  }

  return (
    <form class="card flex flex-col gap-4 p-5 enter" onSubmit={submit} novalidate>
      <label>
        <span class="label">{t("bills.description")}</span>
        <input
          class="field"
          value={description()}
          onInput={(e) => setDescription(e.currentTarget.value)}
          placeholder={t("bills.descriptionPlaceholder")}
        />
      </label>

      <div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
        <label>
          <span class="label">{t("bills.value")}</span>
          <input
            class="field tabular"
            type="text"
            inputmode="decimal"
            placeholder="0,00"
            value={value()}
            onInput={(e) => setValue(e.currentTarget.value)}
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
          <p class="caption mt-1">{t("bills.dueDayHint")}</p>
        </label>
      </div>

      <div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
        <label>
          <span class="label">{t("bills.account")}</span>
          <select class="field" value={accountId()} onChange={(e) => setAccountId(e.currentTarget.value)}>
            <For each={props.accounts}>{(a) => <option value={a.id}>{a.name}</option>}</For>
          </select>
        </label>
        <label>
          <span class="label">{t("bills.category")}</span>
          <select class="field" value={categoryId()} onChange={(e) => setCategoryId(e.currentTarget.value)}>
            <For each={props.categories}>{(c) => <option value={c.id}>{c.name}</option>}</For>
          </select>
        </label>
      </div>

      <Show when={error()}>{(m) => <p class="meta text-[color:var(--color-danger-fg)]">{m()}</p>}</Show>

      <button class="btn btn-primary self-start" type="submit" disabled={busy()}>
        {busy() ? t("addTransaction.saving") : t("bills.submitCreate")}
      </button>
    </form>
  );
}

function BillRow(props: {
  bill: Bill;
  month: string;
  accounts: Array<{ id: string; name: string }>;
  currency: string;
  locale: string;
  onChanged: () => void;
  onEdit: () => void;
}): JSX.Element {
  const { t } = useI18n();
  const [busy, setBusy] = createSignal(false);
  const [editing, setEditing] = createSignal(false);
  const [draftValue, setDraftValue] = createSignal<string>("");

  function startEdit(): void {
    setDraftValue(centsToApi(parseCents(props.bill.value)).replace(".", ","));
    setEditing(true);
  }

  async function saveEdit(): Promise<void> {
    const next = inputToApi(draftValue());
    if (next === props.bill.value) {
      setEditing(false);
      return;
    }
    setBusy(true);
    try {
      await bills.update(props.bill.id, { value: next });
      setEditing(false);
      props.onChanged();
    } catch (err) {
      pushToast(err instanceof ApiError ? err.code : "error", "error");
    } finally {
      setBusy(false);
    }
  }

  async function pay(): Promise<void> {
    const acct = props.bill.account_id ?? props.accounts[0]?.id;
    if (!acct) {
      pushToast(t("addTransaction.accountRequired"), "error");
      return;
    }
    setBusy(true);
    try {
      await bills.pay(props.bill.id, props.month, { source_account_id: acct });
      pushToast(t("bills.paidToast", { name: props.bill.description }), "ok");
      props.onChanged();
    } catch (err) {
      if (err instanceof ApiError && err.code === "CONFLICT") {
        pushToast(t("bills.alreadyPaid"), "info");
      } else {
        pushToast(err instanceof ApiError ? err.code : t("bills.payFailed"), "error");
      }
    } finally {
      setBusy(false);
    }
  }

  async function reset(): Promise<void> {
    setBusy(true);
    try {
      await bills.reset(props.bill.id, props.month);
      props.onChanged();
    } finally {
      setBusy(false);
    }
  }

  async function remove(): Promise<void> {
    setBusy(true);
    try {
      await bills.remove(props.bill.id);
      props.onChanged();
    } finally {
      setBusy(false);
    }
  }

  const cents = (): bigint => parseCents(props.bill.value);

  return (
    <li class="grid grid-cols-[32px_1fr_auto] items-center gap-x-3 gap-y-2 px-4 py-3 sm:grid-cols-[32px_1fr_auto_auto]">
      <span class="tx-icon" aria-hidden>
        {props.bill.description.trim()[0]?.toUpperCase() ?? "?"}
      </span>

      <div class="min-w-0">
        <div class="body-strong truncate">{props.bill.description}</div>
        <div class="meta mt-0.5 flex flex-wrap items-center gap-x-1.5 gap-y-1">
          <Show
            when={props.bill.paid}
            fallback={<span>{t("bills.dueOnDay", { day: props.bill.due_day })}</span>}
          >
            <span class="pill pill-success">{t("bills.paid")}</span>
            <span aria-hidden="true">·</span>
            <span>{t("bills.dueOnDay", { day: props.bill.due_day })}</span>
          </Show>
        </div>
      </div>

      <Show
        when={editing()}
        fallback={
          <button
            type="button"
            class="money tabular col-start-2 row-start-2 self-start text-left hover:text-[color:var(--color-primary-600)] sm:col-start-3 sm:row-start-1 sm:self-center sm:text-right"
            onClick={startEdit}
            aria-label={t("bills.edit")}
          >
            {formatMoney(cents(), props.currency, props.locale)}
          </button>
        }
      >
        <div class="col-start-2 row-start-2 flex items-center gap-2 sm:col-start-3 sm:row-start-1">
          <input
            class="field tabular w-28 text-right"
            type="text"
            inputmode="decimal"
            value={draftValue()}
            onInput={(e) => setDraftValue(e.currentTarget.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter") void saveEdit();
              if (e.key === "Escape") setEditing(false);
            }}
            autofocus
          />
          <button
            type="button"
            class="icon-btn"
            disabled={busy()}
            aria-label={t("common.save")}
            onClick={() => void saveEdit()}
          >
            <span class="material-symbols-rounded" style={{ "font-size": "18px" }}>check</span>
          </button>
          <button
            type="button"
            class="icon-btn"
            aria-label={t("common.cancel")}
            onClick={() => setEditing(false)}
          >
            <span class="material-symbols-rounded" style={{ "font-size": "18px" }}>close</span>
          </button>
        </div>
      </Show>

      <div class="col-start-3 row-start-1 row-span-2 flex items-center gap-1 self-center sm:col-start-4">
        <Show
          when={!props.bill.paid}
          fallback={
            <button class="btn btn-ghost" disabled={busy()} onClick={() => void reset()}>
              {t("bills.reset")}
            </button>
          }
        >
          <button class="btn btn-primary" disabled={busy()} onClick={() => void pay()}>
            {busy() ? t("bills.paying") : t("bills.pay")}
          </button>
        </Show>
        <button class="icon-btn" disabled={busy()} onClick={props.onEdit} aria-label={t("bills.edit")}>
          <span class="material-symbols-rounded" style={{ "font-size": "18px" }}>edit</span>
        </button>
        <button class="icon-btn" disabled={busy()} onClick={() => void remove()} aria-label={t("bills.delete")}>
          <span class="material-symbols-rounded" style={{ "font-size": "18px" }}>delete</span>
        </button>
      </div>
    </li>
  );
}
