import { createResource, createSignal, For, Show, type JSX } from "solid-js";
import { Portal } from "solid-js/web";
import { accounts, cards } from "../api";
import type { Account, Card, InvoicePreview } from "../api/types";
import { useI18n } from "../lib/i18n";
import { useMonth } from "../lib/useMonth";
import { formatMoney, inputToApi, parseCents, centsToApi } from "../lib/money";
import { formatDate } from "../lib/month";
import { EmptyState } from "../components/EmptyState";
import { pushToast } from "../components/Toast";
import { ApiError } from "../api/client";

const BRANDS = ["VISA", "MASTERCARD", "ELO", "AMEX", "OTHER"];

export default function AccountsView(): JSX.Element {
  const { t } = useI18n();
  const { currency, locale } = useMonth();
  const [list, { refetch }] = createResource(() => accounts.list());
  const [cardList, { refetch: refetchCards }] = createResource(() => cards.list());
  const [previews, { refetch: refetchPreviews }] = createResource(() => cards.invoicePreview());

  const [showForm, setShowForm] = createSignal(false);
  const [editing, setEditing] = createSignal<Account | null>(null);
  const [cardModal, setCardModal] = createSignal<{ paymentAccount: Account; card?: Card } | null>(null);

  const userAccounts = () => (list() ?? []).filter((a) => a.type === "asset" && !a.archived);
  const cardsFor = (accountId: string): Card[] =>
    (cardList() ?? []).filter((c) => !c.archived && c.payment_account_id === accountId);
  const previewFor = (cardId: string): InvoicePreview | undefined =>
    (previews() ?? []).find((p) => p.card_id === cardId);

  function refreshAll(): void {
    refetch();
    refetchCards();
    refetchPreviews();
  }

  return (
    <div class="mx-auto w-full max-w-3xl space-y-4 px-4 py-5">
      <header class="flex items-center justify-between gap-3">
        <h2 class="h-title">{t("accounts.title")}</h2>
        <button type="button" class="btn btn-primary" onClick={() => setShowForm((s) => !s)}>
          <span class="material-symbols-rounded" style={{ "font-size": "20px" }}>add</span>
          {t("accounts.addCta")}
        </button>
      </header>

      <Show when={showForm()}>
        <CreateAccountForm
          defaultCurrency={currency()}
          onCreated={() => {
            setShowForm(false);
            refreshAll();
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
          <ul class="space-y-3">
            <For each={userAccounts()}>
              {(a) => (
                <li class="card overflow-hidden">
                  <div class="tx-row px-4 py-3">
                    <span class="tx-icon" aria-hidden>{a.name.trim()[0]?.toUpperCase() ?? "?"}</span>
                    <div class="min-w-0 flex-1">
                      <div class="body-strong truncate">{a.name}</div>
                      <div class="meta">{a.currency}</div>
                    </div>
                    <span class="money tabular">
                      {formatMoney(parseCents(a.current_balance), a.currency, locale())}
                    </span>
                    <button
                      type="button"
                      class="btn btn-ghost ml-2 !px-2"
                      aria-label={t("accounts.edit")}
                      onClick={() => setEditing(a)}
                    >
                      <span class="material-symbols-rounded" style={{ "font-size": "20px" }}>edit</span>
                    </button>
                  </div>

                  {/* attached credit cards */}
                  <div class="border-t border-[color:var(--color-surface-sunken)] bg-[color:var(--color-surface-muted)] px-4 py-3">
                    <div class="mb-2 flex items-center justify-between">
                      <span class="eyebrow">{t("accounts.cards")}</span>
                      <button
                        type="button"
                        class="meta inline-flex items-center gap-1 text-[color:var(--color-primary-600)]"
                        onClick={() => setCardModal({ paymentAccount: a })}
                      >
                        <span class="material-symbols-rounded" style={{ "font-size": "16px" }}>add_card</span>
                        {t("accounts.attachCard")}
                      </button>
                    </div>
                    <Show
                      when={cardsFor(a.id).length > 0}
                      fallback={<p class="meta">{t("accounts.noCards")}</p>}
                    >
                      <ul class="space-y-2">
                        <For each={cardsFor(a.id)}>
                          {(c) => (
                            <CardRow
                              card={c}
                              preview={previewFor(c.id)}
                              currency={a.currency}
                              locale={locale()}
                              onEdit={() => setCardModal({ paymentAccount: a, card: c })}
                            />
                          )}
                        </For>
                      </ul>
                    </Show>
                  </div>
                </li>
              )}
            </For>
          </ul>
        </Show>
      </Show>

      <Show when={editing()}>
        {(a) => (
          <EditAccountModal
            account={a()}
            locale={locale()}
            onClose={() => setEditing(null)}
            onDone={() => {
              setEditing(null);
              refreshAll();
            }}
          />
        )}
      </Show>

      <Show when={cardModal()}>
        {(m) => (
          <CardModal
            paymentAccount={m().paymentAccount}
            card={m().card}
            onClose={() => setCardModal(null)}
            onDone={() => {
              setCardModal(null);
              refreshAll();
            }}
          />
        )}
      </Show>
    </div>
  );
}

function CardRow(props: {
  card: Card;
  preview?: InvoicePreview;
  currency: string;
  locale: string;
  onEdit: () => void;
}): JSX.Element {
  const { t } = useI18n();
  const p = () => props.preview;
  const limitPct = (): number => {
    const pv = p();
    if (!pv || !pv.limit) return 0;
    const lim = parseCents(pv.limit);
    if (lim <= 0n) return 0;
    const out = parseCents(pv.outstanding);
    return Math.min(100, Math.max(0, Number((out * 100n) / lim)));
  };
  return (
    <li class="rounded-lg bg-[color:var(--color-surface)] p-3 shadow-[var(--shadow-card)]">
      <div class="flex items-center justify-between gap-2">
        <div class="flex items-center gap-2">
          <span class="material-symbols-rounded text-[color:var(--color-primary-600)]" style={{ "font-size": "20px" }}>credit_card</span>
          <span class="body-strong">{props.card.brand} •••• {props.card.last_four_digits}</span>
        </div>
        <button type="button" class="btn btn-ghost !px-2" aria-label={t("accounts.edit")} onClick={props.onEdit}>
          <span class="material-symbols-rounded" style={{ "font-size": "18px" }}>edit</span>
        </button>
      </div>
      <Show when={p()} fallback={<p class="meta mt-1">{t("accounts.invoiceNoCycle")}</p>}>
        {(pv) => (
          <div class="mt-2 space-y-1">
            <div class="flex items-baseline justify-between">
              <span class="meta">{t("accounts.invoiceAccrued")}</span>
              <span class="money tabular body-strong">{formatMoney(parseCents(pv().accrued), props.currency, props.locale)}</span>
            </div>
            <div class="flex items-baseline justify-between">
              <span class="meta">{t("accounts.invoiceCycleTotal")}</span>
              <span class="meta tabular">{formatMoney(parseCents(pv().cycle_total), props.currency, props.locale)}</span>
            </div>
            <div class="flex items-baseline justify-between">
              <span class="meta">{t("accounts.invoiceDue")}</span>
              <span class="meta tabular">{formatDate(pv().due_date, props.locale)}</span>
            </div>
            <Show when={pv().limit}>
              <div class="flex items-baseline justify-between">
                <span class="meta">{t("accounts.invoiceOutstanding")}</span>
                <span class="meta tabular">
                  {formatMoney(parseCents(pv().outstanding), props.currency, props.locale)} / {formatMoney(parseCents(pv().limit!), props.currency, props.locale)}
                </span>
              </div>
              <div class="gauge mt-1"><span style={{ width: `${limitPct()}%` }} /></div>
            </Show>
          </div>
        )}
      </Show>
    </li>
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
      await accounts.create({
        name: name().trim(),
        type: "asset",
        currency: currency().trim().toUpperCase(),
        initial_balance: inputToApi(initialBalance() || "0"),
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
        <input class="field" value={name()} onInput={(e) => setName(e.currentTarget.value)}
          placeholder={t("accounts.namePlaceholder")} autocomplete="off" />
      </label>
      <div class="grid grid-cols-2 gap-3">
        <label>
          <span class="label">{t("accounts.currency")}</span>
          <input class="field" value={currency()} onInput={(e) => setCurrency(e.currentTarget.value)} maxLength={3} />
        </label>
        <label>
          <span class="label">{t("accounts.initialBalance")}</span>
          <input class="field" type="text" inputmode="decimal" placeholder="0,00"
            value={initialBalance()} onInput={(e) => setInitialBalance(e.currentTarget.value)} />
        </label>
      </div>
      <Show when={error()}>{(msg) => <p class="meta text-[color:var(--color-danger-fg)]">{msg()}</p>}</Show>
      <button class="btn btn-primary" type="submit" disabled={busy()}>
        {busy() ? t("accounts.creating") : t("accounts.create")}
      </button>
    </form>
  );
}

function EditAccountModal(props: {
  account: Account;
  locale: string;
  onClose: () => void;
  onDone: () => void;
}): JSX.Element {
  const { t } = useI18n();
  const a = props.account;
  const [name, setName] = createSignal(a.name);
  const [currency, setCurrency] = createSignal(a.currency);
  const [opening, setOpening] = createSignal(
    centsToApi(parseCents(a.opening_balance)).replace(".", props.locale.startsWith("pt") ? "," : "."),
  );
  const [busy, setBusy] = createSignal(false);
  const [err, setErr] = createSignal<string | null>(null);

  async function save(e: Event): Promise<void> {
    e.preventDefault();
    setErr(null);
    setBusy(true);
    try {
      await accounts.update(a.id, {
        name: name().trim(),
        currency: currency().trim().toUpperCase(),
        opening_balance: inputToApi(opening() || "0"),
      });
      pushToast(t("accounts.updatedToast", { name: name() }), "ok");
      props.onDone();
    } catch (e2) {
      setErr(e2 instanceof ApiError ? `${e2.code}: ${e2.message}` : "Error");
    } finally {
      setBusy(false);
    }
  }

  async function remove(): Promise<void> {
    if (!confirm(t("accounts.deleteConfirm", { name: a.name }))) return;
    setBusy(true);
    try {
      await accounts.remove(a.id);
      pushToast(t("accounts.deleted"), "ok");
      props.onDone();
    } catch (e2) {
      setErr(e2 instanceof ApiError ? `${e2.code}: ${e2.message}` : "Error");
      setBusy(false);
    }
  }

  return (
    <Portal>
      <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-4"
        onClick={(e) => { if (e.currentTarget === e.target) props.onClose(); }}>
        <form class="card w-full max-w-md flex flex-col p-0" onSubmit={save}>
          <div class="flex items-center justify-between border-b border-[color:var(--color-surface-sunken)] p-5">
            <h2 class="h2">{t("accounts.editTitle")}</h2>
            <button type="button" class="btn btn-ghost !px-2" onClick={props.onClose} aria-label={t("common.cancel")}>
              <span class="material-symbols-rounded">close</span>
            </button>
          </div>
          <div class="flex flex-col gap-4 p-5">
            <label>
              <span class="label">{t("accounts.name")}</span>
              <input class="field" value={name()} onInput={(e) => setName(e.currentTarget.value)} />
            </label>
            <div class="grid grid-cols-2 gap-3">
              <label>
                <span class="label">{t("accounts.currency")}</span>
                <input class="field" value={currency()} onInput={(e) => setCurrency(e.currentTarget.value)} maxLength={3} />
              </label>
              <label>
                <span class="label">{t("accounts.openingBalance")}</span>
                <input class="field" type="text" inputmode="decimal" value={opening()}
                  onInput={(e) => setOpening(e.currentTarget.value)} />
              </label>
            </div>
            <p class="meta">{t("accounts.openingHint")}</p>
            <Show when={err()}>{(m) => <p class="meta text-[color:var(--color-danger-fg)]">{m()}</p>}</Show>
          </div>
          <div class="flex gap-3 border-t border-[color:var(--color-surface-sunken)] p-5">
            <button type="button" class="btn btn-ghost" onClick={remove} disabled={busy()}>{t("common.delete")}</button>
            <span class="flex-1" />
            <button type="button" class="btn btn-ghost" onClick={props.onClose} disabled={busy()}>{t("common.cancel")}</button>
            <button class="btn btn-primary" type="submit" disabled={busy()}>{busy() ? t("accounts.saving") : t("accounts.save")}</button>
          </div>
        </form>
      </div>
    </Portal>
  );
}

function CardModal(props: {
  paymentAccount: Account;
  card?: Card;
  onClose: () => void;
  onDone: () => void;
}): JSX.Element {
  const { t } = useI18n();
  const c = props.card;
  const [brand, setBrand] = createSignal(c?.brand ?? "VISA");
  const [lastFour, setLastFour] = createSignal(c?.last_four_digits ?? "");
  const [limit, setLimit] = createSignal(c?.limit ? centsToApi(parseCents(c.limit)) : "");
  const [closeDay, setCloseDay] = createSignal(c?.close_day?.toString() ?? "");
  const [dueDay, setDueDay] = createSignal(c?.due_day?.toString() ?? "");
  const [busy, setBusy] = createSignal(false);
  const [err, setErr] = createSignal<string | null>(null);

  async function save(e: Event): Promise<void> {
    e.preventDefault();
    setErr(null);
    if (lastFour().trim().length < 2) {
      setErr(t("accounts.lastFourRequired"));
      return;
    }
    setBusy(true);
    const limitVal = limit().trim() ? inputToApi(limit()) : null;
    const close = closeDay().trim() ? Number(closeDay()) : null;
    const due = dueDay().trim() ? Number(dueDay()) : null;
    try {
      if (c) {
        await cards.update(c.id, {
          brand: brand(), last_four_digits: lastFour().trim(),
          limit: limitVal, close_day: close, due_day: due,
        });
      } else {
        await cards.create({
          payment_account_id: props.paymentAccount.id,
          brand: brand(), last_four_digits: lastFour().trim(), type: "CREDIT",
          limit: limitVal, close_day: close, due_day: due,
        });
      }
      pushToast(t("accounts.cardSaved"), "ok");
      props.onDone();
    } catch (e2) {
      setErr(e2 instanceof ApiError ? `${e2.code}: ${e2.message}` : "Error");
    } finally {
      setBusy(false);
    }
  }

  async function remove(): Promise<void> {
    if (!c) return;
    if (!confirm(t("accounts.removeCardConfirm"))) return;
    setBusy(true);
    try {
      await cards.remove(c.id);
      pushToast(t("accounts.cardRemoved"), "ok");
      props.onDone();
    } catch (e2) {
      setErr(e2 instanceof ApiError ? `${e2.code}: ${e2.message}` : "Error");
      setBusy(false);
    }
  }

  return (
    <Portal>
      <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-4"
        onClick={(e) => { if (e.currentTarget === e.target) props.onClose(); }}>
        <form class="card w-full max-w-md flex flex-col p-0" onSubmit={save}>
          <div class="flex items-center justify-between border-b border-[color:var(--color-surface-sunken)] p-5">
            <h2 class="h2">{c ? t("accounts.editCard") : t("accounts.attachCard")}</h2>
            <button type="button" class="btn btn-ghost !px-2" onClick={props.onClose} aria-label={t("common.cancel")}>
              <span class="material-symbols-rounded">close</span>
            </button>
          </div>
          <div class="flex flex-col gap-4 p-5">
            <p class="meta">{t("accounts.attachTo", { name: props.paymentAccount.name })}</p>
            <div class="grid grid-cols-2 gap-3">
              <label>
                <span class="label">{t("accounts.brand")}</span>
                <select class="field" value={brand()} onChange={(e) => setBrand(e.currentTarget.value)}>
                  <For each={BRANDS}>{(b) => <option value={b}>{b}</option>}</For>
                </select>
              </label>
              <label>
                <span class="label">{t("accounts.lastFour")}</span>
                <input class="field" inputmode="numeric" maxLength={4} value={lastFour()}
                  onInput={(e) => setLastFour(e.currentTarget.value.replace(/\D/g, ""))} placeholder="4242" />
              </label>
            </div>
            <label>
              <span class="label">{t("accounts.cardLimit")}</span>
              <input class="field" type="text" inputmode="decimal" placeholder="0,00"
                value={limit()} onInput={(e) => setLimit(e.currentTarget.value)} />
            </label>
            <div class="grid grid-cols-2 gap-3">
              <label>
                <span class="label">{t("accounts.closeDay")}</span>
                <input class="field" inputmode="numeric" maxLength={2} value={closeDay()}
                  onInput={(e) => setCloseDay(e.currentTarget.value.replace(/\D/g, ""))} placeholder="20" />
              </label>
              <label>
                <span class="label">{t("accounts.dueDayCard")}</span>
                <input class="field" inputmode="numeric" maxLength={2} value={dueDay()}
                  onInput={(e) => setDueDay(e.currentTarget.value.replace(/\D/g, ""))} placeholder="28" />
              </label>
            </div>
            <p class="meta">{t("accounts.cardDaysHint")}</p>
            <Show when={err()}>{(m) => <p class="meta text-[color:var(--color-danger-fg)]">{m()}</p>}</Show>
          </div>
          <div class="flex gap-3 border-t border-[color:var(--color-surface-sunken)] p-5">
            <Show when={c}>
              <button type="button" class="btn btn-ghost" onClick={remove} disabled={busy()}>{t("accounts.removeCard")}</button>
            </Show>
            <span class="flex-1" />
            <button type="button" class="btn btn-ghost" onClick={props.onClose} disabled={busy()}>{t("common.cancel")}</button>
            <button class="btn btn-primary" type="submit" disabled={busy()}>{busy() ? t("accounts.saving") : t("accounts.save")}</button>
          </div>
        </form>
      </div>
    </Portal>
  );
}
