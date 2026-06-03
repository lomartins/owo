import { createResource, createSignal, For, Show, type JSX } from "solid-js";
import { accounts } from "../api";
import { useI18n } from "../lib/i18n";
import { useMonth } from "../lib/useMonth";
import { formatMoney, inputToApi, parseCents } from "../lib/money";
import { EmptyState } from "../components/EmptyState";
import { pushToast } from "../components/Toast";
import { ApiError } from "../api/client";

export default function AccountsView(): JSX.Element {
  const { t } = useI18n();
  const { currency, locale } = useMonth();
  const [list, { refetch }] = createResource(() => accounts.list());
  const [showForm, setShowForm] = createSignal(false);

  const userAccounts = () => (list() ?? []).filter((a) => a.type === "asset" && !a.archived);

  return (
    <div class="mx-auto w-full max-w-3xl space-y-4 px-4 py-5">
      <header class="flex items-center justify-between gap-3">
        <h2 class="h-title">{t("accounts.title")}</h2>
        <button
          type="button"
          class="btn btn-primary"
          onClick={() => setShowForm((s) => !s)}
        >
          <span class="material-symbols-rounded" style={{ "font-size": "20px" }}>add</span>
          {t("accounts.addCta")}
        </button>
      </header>

      <Show when={showForm()}>
        <CreateAccountForm
          defaultCurrency={currency()}
          onCreated={() => {
            setShowForm(false);
            refetch();
          }}
        />
      </Show>

      <Show
        when={list()}
        fallback={<div class="card h-32 animate-pulse bg-[color:var(--color-surface-muted)]" />}
      >
        <Show
          when={userAccounts().length > 0}
          fallback={
            <EmptyState
              icon="account_balance_wallet"
              title={t("accounts.empty")}
              hint={t("accounts.emptyHint")}
              action={
                <button class="btn btn-primary" onClick={() => setShowForm(true)}>
                  {t("accounts.addCta")}
                </button>
              }
            />
          }
        >
          <ul class="card divide-y divide-[color:var(--color-surface-sunken)]">
            <For each={userAccounts()}>
              {(a) => (
                <li class="tx-row px-4 py-3">
                  <span class="tx-icon" aria-hidden>
                    {a.name.trim()[0]?.toUpperCase() ?? "?"}
                  </span>
                  <div class="min-w-0 flex-1">
                    <div class="body-strong truncate">{a.name}</div>
                    <div class="meta">{a.currency}</div>
                  </div>
                  <span class="money tabular">
                    {formatMoney(parseCents(a.current_balance), a.currency, locale())}
                  </span>
                </li>
              )}
            </For>
          </ul>
        </Show>
      </Show>
    </div>
  );
}

function CreateAccountForm(props: { defaultCurrency: string; onCreated: () => void }): JSX.Element {
  const { t } = useI18n();
  const [name, setName] = createSignal("");
  const [currency, setCurrency] = createSignal(props.defaultCurrency || "BRL");
  const [initialBalance, setInitialBalance] = createSignal("");
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  async function submit(e: Event): Promise<void> {
    e.preventDefault();
    setError(null);
    if (!name().trim()) {
      setError(t("accounts.nameRequired"));
      return;
    }
    setBusy(true);
    try {
      const initial = inputToApi(initialBalance() || "0");
      await accounts.create({
        name: name().trim(),
        type: "asset",
        currency: currency().trim().toUpperCase(),
        initial_balance: initial,
      });
      pushToast(t("accounts.createdToast", { name: name() }), "ok");
      setName("");
      setInitialBalance("");
      props.onCreated();
    } catch (err) {
      setError(err instanceof ApiError ? `${err.code}: ${err.message}` : "Error");
    } finally {
      setBusy(false);
    }
  }

  return (
    <form class="card flex flex-col gap-4 p-5" onSubmit={submit} novalidate>
      <label>
        <span class="label">{t("accounts.name")}</span>
        <input
          class="field"
          value={name()}
          onInput={(e) => setName(e.currentTarget.value)}
          placeholder={t("accounts.namePlaceholder")}
          autocomplete="off"
        />
      </label>

      <div class="grid grid-cols-2 gap-3">
        <label>
          <span class="label">{t("accounts.currency")}</span>
          <input
            class="field"
            value={currency()}
            onInput={(e) => setCurrency(e.currentTarget.value)}
            maxLength={3}
          />
        </label>
        <label>
          <span class="label">{t("accounts.initialBalance")}</span>
          <input
            class="field"
            type="text"
            inputmode="decimal"
            placeholder="0,00"
            value={initialBalance()}
            onInput={(e) => setInitialBalance(e.currentTarget.value)}
          />
        </label>
      </div>

      <Show when={error()}>{(msg) => <p class="meta text-[color:var(--color-danger-fg)]">{msg()}</p>}</Show>

      <button class="btn btn-primary" type="submit" disabled={busy()}>
        {busy() ? t("accounts.creating") : t("accounts.create")}
      </button>
    </form>
  );
}
