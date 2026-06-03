import { createResource, createSignal, For, Show, type JSX } from "solid-js";
import { useNavigate, A } from "@solidjs/router";
import { accounts, categories, transactions } from "../api";
import type { Account, Category, PaymentMethod } from "../api/types";
import { today } from "../lib/month";
import { centsToApi, inputToApi, parseCents, formatMoney } from "../lib/money";
import { ApiError } from "../api/client";
import { useMonth } from "../lib/useMonth";
import { pushToast } from "../components/Toast";
import { useI18n } from "../lib/i18n";
import { localizeCategoryName, isSeedName } from "../lib/categories";
import { CategoryThumb } from "../components/CategoryThumb";

type Mode = "out" | "in" | "transfer";

const PAYMENT_METHODS: { value: PaymentMethod; icon: string }[] = [
  { value: "PIX", icon: "bolt" },
  { value: "CARD", icon: "credit_card" },
  { value: "CASH", icon: "payments" },
  { value: "BOLETO", icon: "receipt_long" },
  { value: "VA", icon: "lunch_dining" },
  { value: "DEBIT", icon: "south" },
  { value: "CREDIT", icon: "north" },
  { value: "TED", icon: "swap_horiz" },
];

const PAYMENT_LABEL: Record<PaymentMethod, string> = {
  PIX: "PIX", CARD: "Card", CASH: "Cash", BOLETO: "Boleto",
  VA: "VA", DEBIT: "Debit", CREDIT: "Credit", TED: "TED",
};
const PAYMENT_LABEL_PT: Record<PaymentMethod, string> = {
  PIX: "PIX", CARD: "Cartão", CASH: "Dinheiro", BOLETO: "Boleto",
  VA: "VA", DEBIT: "Débito", CREDIT: "Crédito", TED: "TED",
};

export default function AddTransaction(): JSX.Element {
  const navigate = useNavigate();
  const { currency, locale, bumpRefresh } = useMonth();
  const { t, lang } = useI18n();

  const [mode, setMode] = createSignal<Mode>("out");

  const [accs] = createResource(() => accounts.list());
  const [cats] = createResource(() => categories.list());

  const visibleCategories = (): Category[] => {
    const list = cats() ?? [];
    return mode() === "out"
      ? list.filter((c) => !c.archived && (c.kind === "EXPENSE" || c.kind === "BOTH"))
      : list.filter((c) => !c.archived && (c.kind === "INCOME" || c.kind === "BOTH"));
  };

  // Asset + credit_card accounts can both participate in transfers; only asset
  // accounts source non-transfer expenses (matches backend leg-pair rules).
  const transferableAccounts = (): Account[] =>
    (accs() ?? []).filter((a) => !a.archived && (a.type === "asset" || a.type === "credit_card"));
  const assetAccounts = (): Account[] =>
    (accs() ?? []).filter((a) => !a.archived && a.type === "asset");

  const [amountInput, setAmountInput] = createSignal("");
  const amountCents = (): bigint => parseCents(inputToApi(amountInput()));

  const [accountId, setAccountId] = createSignal<string>("");
  const [destAccountId, setDestAccountId] = createSignal<string>("");
  const [categoryId, setCategoryId] = createSignal<string>("");
  const [method, setMethod] = createSignal<PaymentMethod>("PIX");
  const [paid, setPaid] = createSignal(true);
  const [date, setDate] = createSignal(today());
  const [description, setDescription] = createSignal("");

  const [busy, setBusy] = createSignal(false);
  const [errMsg, setErrMsg] = createSignal<string | null>(null);

  function ensureDefaults(): void {
    if (mode() === "transfer") {
      const pool = transferableAccounts();
      if (!accountId() && pool[0]) setAccountId(pool[0].id);
      if (!destAccountId() && pool[1]) setDestAccountId(pool[1].id);
    } else {
      if (!accountId() && assetAccounts()[0]) setAccountId(assetAccounts()[0]!.id);
      if (!categoryId() && visibleCategories()[0]) setCategoryId(visibleCategories()[0]!.id);
    }
  }

  async function onSubmit(e: Event): Promise<void> {
    e.preventDefault();
    setErrMsg(null);
    if (amountCents() <= 0n) {
      setErrMsg(t("addTransaction.amountRequired"));
      return;
    }
    if (mode() === "transfer") {
      if (!accountId() || !destAccountId()) {
        setErrMsg(t("addTransaction.accountRequired"));
        return;
      }
      if (accountId() === destAccountId()) {
        setErrMsg(t("addTransaction.transferSamePair"));
        return;
      }
    } else {
      if (!categoryId()) {
        setErrMsg(t("addTransaction.categoryRequired"));
        return;
      }
      if (!accountId()) {
        setErrMsg(t("addTransaction.accountRequired"));
        return;
      }
    }

    setBusy(true);
    try {
      if (mode() === "transfer") {
        await transactions.transfer({
          source_account_id: accountId(),
          destination_account_id: destAccountId(),
          value: centsToApi(amountCents()),
          currency: currency(),
          tx_date: date(),
          description: description() || t("addTransaction.transfer"),
          payment_method: method(),
        });
        pushToast(t("addTransaction.transferSaved"), "ok");
      } else {
        const payload =
          mode() === "out"
            ? { source_account_id: accountId(), category_id: categoryId() }
            : { destination_account_id: accountId(), category_id: categoryId() };
        await transactions.create({
          ...payload,
          payment_method: method(),
          value: centsToApi(amountCents()),
          currency: currency(),
          description: description(),
          tx_date: date(),
          paid: paid(),
        });
        pushToast(`${t("addTransaction.title")} ✓`, "ok");
      }
      bumpRefresh();
      navigate("/transactions");
    } catch (err) {
      if (err instanceof ApiError) {
        setErrMsg(`${err.code}: ${err.message}`);
      } else {
        setErrMsg(t("auth.networkError"));
      }
    } finally {
      setBusy(false);
    }
  }

  const payLabel = (pm: PaymentMethod): string =>
    lang() === "pt" ? PAYMENT_LABEL_PT[pm] : PAYMENT_LABEL[pm];

  return (
    <div class="mx-auto w-full max-w-xl px-4 py-5">
      <form class="card flex flex-col gap-5 p-5" onSubmit={onSubmit}>
        <div class="chip-row self-center">
          <button
            type="button"
            class="chip"
            aria-pressed={mode() === "out"}
            classList={{ "is-active": mode() === "out" }}
            onClick={() => setMode("out")}
          >
            {t("addTransaction.expense")}
          </button>
          <button
            type="button"
            class="chip"
            aria-pressed={mode() === "in"}
            classList={{ "is-active": mode() === "in" }}
            onClick={() => setMode("in")}
          >
            {t("addTransaction.income")}
          </button>
          <button
            type="button"
            class="chip"
            aria-pressed={mode() === "transfer"}
            classList={{ "is-active": mode() === "transfer" }}
            onClick={() => setMode("transfer")}
          >
            {t("addTransaction.transfer")}
          </button>
        </div>

        <label class="block">
          <span class="label">{t("addTransaction.amount")}</span>
          <div class="flex items-baseline gap-3">
            <span class="text-[color:var(--color-text-muted)]" style={{ "font-size": "28px" }}>
              {new Intl.NumberFormat(locale(), { style: "currency", currency: currency() })
                .formatToParts(0)
                .find((p) => p.type === "currency")?.value}
            </span>
            <input
              class="field num !border-transparent !bg-transparent !p-0 !text-4xl !font-medium focus:!shadow-none"
              type="text"
              inputmode="decimal"
              placeholder="0,00"
              value={amountInput()}
              onInput={(e) => setAmountInput(e.currentTarget.value)}
              onBlur={(e) => {
                const v = e.currentTarget.value;
                if (v) e.currentTarget.value = inputToApi(v).replace(".", locale().startsWith("pt") ? "," : ".");
                setAmountInput(e.currentTarget.value);
              }}
            />
          </div>
          <Show when={amountCents() > 0n}>
            <p class="meta tabular mt-1">= {formatMoney(amountCents(), currency(), locale())}</p>
          </Show>
        </label>

        <Show when={mode() !== "transfer"}>
          <section>
            <span class="label">{t("transactions.category")}</span>
            <div class="flex flex-wrap gap-2">
              <For each={visibleCategories()}>
                {(c) => {
                  ensureDefaults();
                  const label = isSeedName(c.name) ? localizeCategoryName(c.name, lang()) : c.name;
                  return (
                    <button
                      type="button"
                      class="chip-select"
                      aria-pressed={categoryId() === c.id}
                      classList={{ "is-active": categoryId() === c.id }}
                      onClick={() => setCategoryId(c.id)}
                    >
                      <CategoryThumb
                        icon={c.icon}
                        color={c.color}
                        fallback={label.trim()[0]?.toUpperCase() ?? "?"}
                        size={16}
                        glyphSize={12}
                      />
                      {label}
                    </button>
                  );
                }}
              </For>
              <Show when={(cats()?.length ?? 0) === 0 && !cats.loading}>
                <span class="meta">{t("dashboard.emptyCategories")}</span>
              </Show>
            </div>
          </section>
        </Show>

        <Show when={mode() !== "transfer"} fallback={
          <section class="grid grid-cols-1 gap-3 sm:grid-cols-2">
            <label>
              <span class="label">{t("addTransaction.sourceAccount")}</span>
              <Show when={transferableAccounts().length > 0} fallback={<EmptyAccounts />}>
                <select class="field" value={accountId()} onChange={(e) => setAccountId(e.currentTarget.value)}>
                  <For each={transferableAccounts()}>
                    {(a) => (
                      <option value={a.id}>
                        {a.name} · {formatMoney(parseCents(a.current_balance), a.currency, locale())}
                      </option>
                    )}
                  </For>
                </select>
              </Show>
            </label>
            <label>
              <span class="label">{t("addTransaction.destinationAccount")}</span>
              <select class="field" value={destAccountId()} onChange={(e) => setDestAccountId(e.currentTarget.value)}>
                <For each={transferableAccounts()}>
                  {(a) => (
                    <option value={a.id} disabled={a.id === accountId()}>
                      {a.name} · {formatMoney(parseCents(a.current_balance), a.currency, locale())}
                    </option>
                  )}
                </For>
              </select>
            </label>
          </section>
        }>
          <section>
            <span class="label">{t("transactions.account")}</span>
            <Show when={assetAccounts().length > 0} fallback={<EmptyAccounts />}>
              <select class="field" value={accountId()} onChange={(e) => setAccountId(e.currentTarget.value)}>
                <For each={assetAccounts()}>
                  {(a) => (
                    <option value={a.id}>
                      {a.name} · {formatMoney(parseCents(a.current_balance), a.currency, locale())}
                    </option>
                  )}
                </For>
              </select>
            </Show>
          </section>
        </Show>

        <section>
          <span class="label">{t("addTransaction.paymentMethod")}</span>
          <div class="flex flex-wrap gap-2">
            <For each={PAYMENT_METHODS}>
              {(p) => (
                <button
                  type="button"
                  class="chip-select"
                  aria-pressed={method() === p.value}
                  classList={{ "is-active": method() === p.value }}
                  onClick={() => setMethod(p.value)}
                >
                  <span class="material-symbols-rounded" aria-hidden style={{ "font-size": "16px" }}>{p.icon}</span>
                  {payLabel(p.value)}
                </button>
              )}
            </For>
          </div>
        </section>

        <section class="grid grid-cols-2 gap-3">
          <label>
            <span class="label">{t("addTransaction.date")}</span>
            <input type="date" class="field" value={date()} onInput={(e) => setDate(e.currentTarget.value)} />
          </label>
          <Show when={mode() !== "transfer"}>
            <label class="flex flex-col">
              <span class="label">{t("addTransaction.paid")}</span>
              <button
                type="button"
                class="field flex items-center justify-between"
                onClick={() => setPaid(!paid())}
                aria-pressed={paid()}
              >
                <span>{paid() ? (lang() === "pt" ? "Sim" : "Yes") : (lang() === "pt" ? "Pendente" : "Pending")}</span>
                <span
                  class="inline-block h-6 w-11 rounded-full transition"
                  classList={{
                    "bg-[color:var(--color-primary-600)]": paid(),
                    "bg-[color:var(--color-surface-sunken)]": !paid(),
                  }}
                >
                  <span
                    class="block h-5 w-5 translate-y-0.5 rounded-full bg-white shadow transition"
                    classList={{ "translate-x-5": paid(), "translate-x-0.5": !paid() }}
                  />
                </span>
              </button>
            </label>
          </Show>
        </section>

        <label>
          <span class="label">{t("addTransaction.description")}</span>
          <input
            class="field"
            value={description()}
            onInput={(e) => setDescription(e.currentTarget.value)}
            placeholder={t("addTransaction.descriptionPlaceholder")}
          />
        </label>

        <Show when={errMsg()}>
          {(m) => <p class="meta text-[color:var(--color-danger-fg)]">{m()}</p>}
        </Show>

        <div class="flex gap-3">
          <button type="button" class="btn btn-ghost flex-1" onClick={() => navigate("/")}>
            {t("common.cancel")}
          </button>
          <button class="btn btn-primary flex-[2]" type="submit" disabled={busy()}>
            {busy() ? t("addTransaction.saving") : t("addTransaction.submit")}
          </button>
        </div>
      </form>
    </div>
  );
}

function EmptyAccounts(): JSX.Element {
  const { t } = useI18n();
  return (
    <div class="card-muted flex items-center justify-between gap-3 p-3">
      <span class="meta">{t("accounts.empty")}</span>
      <A href="/accounts" class="btn btn-primary">{t("accounts.addCta")}</A>
    </div>
  );
}
