import { createResource, createSignal, For, Show, type JSX } from "solid-js";
import { Portal } from "solid-js/web";
import { A } from "@solidjs/router";
import { accounts, categories, transactions } from "../api";
import { useMonth } from "../lib/useMonth";
import { formatMoney, parseCents, centsToApi, inputToApi } from "../lib/money";
import { formatDate } from "../lib/month";
import type { Account, Category, PaymentMethod, Transaction } from "../api/types";
import { EmptyState } from "../components/EmptyState";
import { useI18n } from "../lib/i18n";
import { localizeCategoryName, isSeedName } from "../lib/categories";
import { ApiError } from "../api/client";
import { pushToast } from "../components/Toast";

const PAYMENT_METHODS: PaymentMethod[] = ["PIX", "CARD", "CASH", "BOLETO", "VA", "DEBIT", "CREDIT", "TED"];

export default function TransactionsList(): JSX.Element {
  const { month, currency, locale, bumpRefresh } = useMonth();
  const { t, lang } = useI18n();
  const localize = (name: string): string =>
    isSeedName(name) ? localizeCategoryName(name, lang()) : name;
  const [accountId, setAccountId] = createSignal<string>("");
  const [categoryId, setCategoryId] = createSignal<string>("");
  const [editing, setEditing] = createSignal<Transaction | null>(null);

  const [accs] = createResource(() => accounts.list());
  const [cats] = createResource(() => categories.list());

  type Args = { month: string; account?: string; category?: string; refresh: number };
  const [page, { refetch }] = createResource<{ items: Transaction[] }, Args>(
    () => ({ month: month(), account: accountId(), category: categoryId(), refresh: 0 }),
    (args) =>
      transactions.list({
        month: args.month,
        account_id: args.account || undefined,
        category_id: args.category || undefined,
        limit: 100,
      }),
  );

  const acctMap = (): Map<string, Account> => new Map((accs() ?? []).map((a) => [a.id, a]));
  const catMap = (): Map<string, Category> => new Map((cats() ?? []).map((c) => [c.id, c]));

  function visibleAccountName(tx: Transaction): string {
    const m = acctMap();
    return m.get(tx.source_account_id)?.name ?? m.get(tx.destination_account_id)?.name ?? "—";
  }

  function onEdited(): void {
    setEditing(null);
    refetch();
    bumpRefresh();
  }

  return (
    <div class="mx-auto w-full max-w-4xl space-y-4 px-4 py-5">
      <section class="flex flex-wrap gap-3">
        <label class="flex-1 min-w-[180px]">
          <span class="label">{t("transactions.account")}</span>
          <select
            class="field"
            value={accountId()}
            onChange={(e) => setAccountId(e.currentTarget.value)}
          >
            <option value="">{t("transactions.allAccounts")}</option>
            <For each={accs() ?? []}>{(a) => <option value={a.id}>{a.name}</option>}</For>
          </select>
        </label>
        <label class="flex-1 min-w-[180px]">
          <span class="label">{t("transactions.category")}</span>
          <select
            class="field"
            value={categoryId()}
            onChange={(e) => setCategoryId(e.currentTarget.value)}
          >
            <option value="">{t("transactions.allCategories")}</option>
            <For each={cats() ?? []}>{(c) => <option value={c.id}>{localize(c.name)}</option>}</For>
          </select>
        </label>
      </section>

      <Show
        when={page()}
        fallback={<div class="card h-48 animate-pulse bg-[color:var(--color-bone-100)]" />}
      >
        {(p) => (
          <Show
            when={p().items.length > 0}
            fallback={
              <EmptyState
                icon="receipt_long"
                title={t("transactions.empty")}
                hint={t("transactions.emptyHint")}
                action={<A href="/add" class="btn btn-primary mt-2">{t("transactions.addCta")}</A>}
              />
            }
          >
            <ul class="card divide-y divide-[color:var(--color-bone-200)]">
              <For each={p().items}>
                {(tx) => (
                  <Row
                    tx={tx}
                    accountName={visibleAccountName(tx)}
                    categoryName={tx.category_id ? localize(catMap().get(tx.category_id)?.name ?? "—") : t("transactions.transfer")}
                    currency={currency()}
                    locale={locale()}
                    pendingLabel={t("transactions.pending")}
                    onClick={() => setEditing(tx)}
                  />
                )}
              </For>
            </ul>
          </Show>
        )}
      </Show>

      <Show when={editing()}>
        {(tx) => (
          <EditTransactionModal
            tx={tx()}
            accounts={accs() ?? []}
            categories={cats() ?? []}
            currency={currency()}
            locale={locale()}
            onClose={() => setEditing(null)}
            onDone={onEdited}
          />
        )}
      </Show>
    </div>
  );
}

function Row(props: {
  tx: Transaction;
  accountName: string;
  categoryName: string;
  currency: string;
  locale: string;
  pendingLabel: string;
  onClick: () => void;
}): JSX.Element {
  const value = parseCents(props.tx.value);
  const kind = props.tx.kind;
  const signed = kind === "withdrawal" ? -value : kind === "deposit" ? value : 0n;
  const sign = signed < 0n ? "−" : signed > 0n ? "+" : "";
  const amountAbs = signed < 0n ? -signed : signed === 0n ? value : signed;

  return (
    <li
      class="tx-row px-4 py-3 cursor-pointer hover:bg-[color:var(--color-surface-muted)]"
      onClick={props.onClick}
      role="button"
      tabIndex={0}
      onKeyDown={(e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          props.onClick();
        }
      }}
    >
      <div class="tx-icon">{props.tx.payment_method.slice(0, 1)}</div>
      <div class="min-w-0 flex-1">
        <div class="flex items-baseline justify-between gap-2">
          <span class="body-strong truncate">{props.tx.description || props.categoryName}</span>
          <span
            class="money tabular"
            classList={{
              "money-overdue": signed < 0n,
              "money-income": signed > 0n,
            }}
          >
            {sign}
            {formatMoney(amountAbs, props.currency, props.locale).replace(/\s/g, "")}
          </span>
        </div>
        <div class="meta flex items-center gap-1">
          <span>{props.categoryName}</span>
          <span aria-hidden="true">·</span>
          <span class="truncate">{props.accountName}</span>
          <span aria-hidden="true">·</span>
          <span>{formatDate(props.tx.tx_date, props.locale)}</span>
          <Show when={!props.tx.paid}>
            <span class="pill pill-overdue ml-1">{props.pendingLabel}</span>
          </Show>
        </div>
      </div>
    </li>
  );
}

function EditTransactionModal(props: {
  tx: Transaction;
  accounts: Account[];
  categories: Category[];
  currency: string;
  locale: string;
  onClose: () => void;
  onDone: () => void;
}): JSX.Element {
  const { t, lang } = useI18n();
  const tx = props.tx;
  const isTransfer = tx.kind === "transfer";
  const localize = (name: string): string =>
    isSeedName(name) ? localizeCategoryName(name, lang()) : name;

  const [amountInput, setAmountInput] = createSignal(tx.value.replace(".", lang() === "pt" ? "," : "."));
  const [description, setDescription] = createSignal(tx.description);
  const [date, setDate] = createSignal(tx.tx_date);
  const [method, setMethod] = createSignal<string>(tx.payment_method);
  const [paid, setPaid] = createSignal(tx.paid);
  const [categoryId, setCategoryId] = createSignal<string>(tx.category_id ?? "");
  const [sourceId, setSourceId] = createSignal<string>(tx.source_account_id);
  const [destId, setDestId] = createSignal<string>(tx.destination_account_id);

  const [busy, setBusy] = createSignal(false);
  const [errMsg, setErrMsg] = createSignal<string | null>(null);

  const transferable = props.accounts.filter((a) => !a.archived && (a.type === "asset" || a.type === "credit_card"));
  const userAccounts = props.accounts.filter((a) => !a.archived);
  // Backend doesn't gate category.kind against tx direction, so show all
  // non-archived categories here. Filtering by kind would hide legitimately-
  // attached categories (e.g. legacy income tx pointing at an EXPENSE seed).
  const visibleCats = props.categories.filter((c) => !c.archived);

  // For non-transfer rows expose only the user-side account; leave bucket leg untouched.
  const userSideIsSource = tx.kind === "withdrawal";

  async function onSave(e: Event): Promise<void> {
    e.preventDefault();
    setErrMsg(null);
    const cents = parseCents(inputToApi(amountInput()));
    if (cents <= 0n) {
      setErrMsg(t("addTransaction.amountRequired"));
      return;
    }
    setBusy(true);
    try {
      const patch: Record<string, unknown> = {
        value: centsToApi(cents),
        description: description(),
        tx_date: date(),
        payment_method: method(),
        paid: paid(),
      };
      if (isTransfer) {
        patch.source_account_id = sourceId();
        patch.destination_account_id = destId();
      } else {
        patch.category_id = categoryId();
        if (userSideIsSource) patch.source_account_id = sourceId();
        else patch.destination_account_id = destId();
      }
      await transactions.update(tx.id, patch);
      pushToast(t("transactions.saved"), "ok");
      props.onDone();
    } catch (err) {
      if (err instanceof ApiError) setErrMsg(`${err.code}: ${err.message}`);
      else setErrMsg(t("transactions.saveFailed"));
    } finally {
      setBusy(false);
    }
  }

  async function onDelete(): Promise<void> {
    if (!confirm(t("transactions.deleteConfirm"))) return;
    setBusy(true);
    try {
      await transactions.remove(tx.id);
      pushToast(t("transactions.deleted"), "ok");
      props.onDone();
    } catch {
      setErrMsg(t("transactions.deleteFailed"));
    } finally {
      setBusy(false);
    }
  }

  return (
    <Portal>
    <div
      class="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-4"
      onClick={(e) => {
        if (e.currentTarget === e.target) props.onClose();
      }}
    >
      <form
        class="card w-full max-w-lg flex flex-col p-0 max-h-[90vh]"
        onSubmit={onSave}
      >
        <div class="flex items-center justify-between border-b border-[color:var(--color-bone-200)] p-5">
          <h2 class="h2">{t("transactions.edit")}</h2>
          <button type="button" class="btn btn-ghost" onClick={props.onClose} aria-label={t("common.cancel")}>
            <span class="material-symbols-rounded">close</span>
          </button>
        </div>

        <div class="flex-1 overflow-y-auto p-5 flex flex-col gap-4">
        <label>
          <span class="label">{t("addTransaction.amount")}</span>
          <input
            class="field num"
            type="text"
            inputmode="decimal"
            value={amountInput()}
            onInput={(e) => setAmountInput(e.currentTarget.value)}
          />
        </label>

        <label>
          <span class="label">{t("addTransaction.description")}</span>
          <input class="field" value={description()} onInput={(e) => setDescription(e.currentTarget.value)} />
        </label>

        <section class="grid grid-cols-2 gap-3">
          <label>
            <span class="label">{t("addTransaction.date")}</span>
            <input type="date" class="field" value={date()} onInput={(e) => setDate(e.currentTarget.value)} />
          </label>
          <label>
            <span class="label">{t("addTransaction.paymentMethod")}</span>
            <select class="field" value={method()} onChange={(e) => setMethod(e.currentTarget.value)}>
              <For each={PAYMENT_METHODS}>{(p) => <option value={p}>{p}</option>}</For>
            </select>
          </label>
        </section>

        <Show when={!isTransfer}>
          <label>
            <span class="label">{t("transactions.category")}</span>
            <select class="field" value={categoryId()} onChange={(e) => setCategoryId(e.currentTarget.value)}>
              <For each={visibleCats}>{(c) => <option value={c.id}>{localize(c.name)}</option>}</For>
            </select>
          </label>
          <label>
            <span class="label">{t("transactions.account")}</span>
            <Show when={userSideIsSource} fallback={
              <select class="field" value={destId()} onChange={(e) => setDestId(e.currentTarget.value)}>
                <For each={userAccounts.filter((a) => a.type !== "expense" && a.type !== "revenue")}>
                  {(a) => <option value={a.id}>{a.name}</option>}
                </For>
              </select>
            }>
              <select class="field" value={sourceId()} onChange={(e) => setSourceId(e.currentTarget.value)}>
                <For each={userAccounts.filter((a) => a.type !== "expense" && a.type !== "revenue")}>
                  {(a) => <option value={a.id}>{a.name}</option>}
                </For>
              </select>
            </Show>
          </label>
        </Show>

        <Show when={isTransfer}>
          <section class="grid grid-cols-2 gap-3">
            <label>
              <span class="label">{t("addTransaction.sourceAccount")}</span>
              <select class="field" value={sourceId()} onChange={(e) => setSourceId(e.currentTarget.value)}>
                <For each={transferable}>{(a) => <option value={a.id}>{a.name}</option>}</For>
              </select>
            </label>
            <label>
              <span class="label">{t("addTransaction.destinationAccount")}</span>
              <select class="field" value={destId()} onChange={(e) => setDestId(e.currentTarget.value)}>
                <For each={transferable}>
                  {(a) => <option value={a.id} disabled={a.id === sourceId()}>{a.name}</option>}
                </For>
              </select>
            </label>
          </section>
        </Show>

        <Show when={!isTransfer}>
          <label class="flex items-center gap-3">
            <input type="checkbox" checked={paid()} onChange={(e) => setPaid(e.currentTarget.checked)} />
            <span>{t("addTransaction.paid")}</span>
          </label>
        </Show>

        <Show when={errMsg()}>
          {(m) => <p class="meta text-[color:var(--color-danger-fg)]">{m()}</p>}
        </Show>
        </div>

        <div class="flex gap-3 border-t border-[color:var(--color-bone-200)] p-5">
          <button type="button" class="btn btn-ghost" onClick={onDelete} disabled={busy()}>
            {t("common.delete")}
          </button>
          <span class="flex-1" />
          <button type="button" class="btn btn-ghost" onClick={props.onClose} disabled={busy()}>
            {t("common.cancel")}
          </button>
          <button class="btn btn-primary" type="submit" disabled={busy()}>
            {busy() ? t("transactions.saving") : t("transactions.saveEdit")}
          </button>
        </div>
      </form>
    </div>
    </Portal>
  );
}
