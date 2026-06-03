import { createResource, createSignal, For, Show, type JSX } from "solid-js";
import { useMonth } from "../lib/useMonth";
import { budgets } from "../api";
import { centsToApi, formatMoney, formatSigned, inputToApi, parseCents } from "../lib/money";
import type { BudgetRow } from "../api/types";
import { EmptyState } from "../components/EmptyState";
import { ApiError } from "../api/client";
import { pushToast } from "../components/Toast";
import { useI18n } from "../lib/i18n";
import { localizeCategoryName, isSeedName } from "../lib/categories";
import { CategoryThumb } from "../components/CategoryThumb";
import { defaultColorForSeed, defaultIconForSeed } from "../lib/categoryIcons";

export default function BudgetEditor(): JSX.Element {
  const { month, currency, locale, bumpRefresh } = useMonth();
  const { t, lang } = useI18n();
  const [data, { refetch }] = createResource(month, (m) => budgets.list(m));

  return (
    <div class="mx-auto w-full max-w-3xl px-4 py-5">
      <div class="mb-3 px-1">
        <h2 class="h-title">{t("budgets.title")}</h2>
        <p class="meta mt-1">{t("budgets.hint")}</p>
      </div>
      <Show
        when={data()}
        fallback={<div class="card h-48 animate-pulse bg-[color:var(--color-bone-100)]" />}
      >
        {(d) => (
          <Show
            when={d().items.length > 0}
            fallback={
              <EmptyState
                icon="category"
                title={t("budgets.empty")}
                hint={t("budgets.emptyHint")}
              />
            }
          >
            <ul class="card divide-y divide-[color:var(--color-bone-200)]">
              <For each={d().items}>
                {(row) => (
                  <BudgetEntry
                    row={row}
                    month={month()}
                    currency={currency()}
                    locale={locale()}
                    displayName={isSeedName(row.category_name) ? localizeCategoryName(row.category_name, lang()) : row.category_name}
                    onSaved={() => {
                      refetch();
                      bumpRefresh();
                    }}
                    t={t}
                  />
                )}
              </For>
            </ul>
          </Show>
        )}
      </Show>
    </div>
  );
}

function BudgetEntry(props: {
  row: BudgetRow;
  month: string;
  currency: string;
  locale: string;
  displayName: string;
  onSaved: () => void;
  t: (key: string, params?: Record<string, string | number>) => string;
}): JSX.Element {
  const initialEstimated = (): bigint => parseCents(props.row.estimated);
  const initialDisplay = (): string => {
    if (initialEstimated() === 0n) return "";
    return formatMoney(initialEstimated(), props.currency, props.locale).replace(/[^0-9.,-]/g, "").trim();
  };
  const [input, setInput] = createSignal<string>(initialDisplay());
  const [busy, setBusy] = createSignal(false);

  const draftCents = (): bigint => parseCents(inputToApi(input()));
  const spent = (): bigint => parseCents(props.row.spent);
  const previewDiff = (): bigint => draftCents() - spent();

  const pct = (): number => {
    const e = draftCents();
    if (e <= 0n) return spent() > 0n ? 100 : 0;
    const ratio = Number(spent() * 100n) / Number(e);
    return Math.min(100, Math.max(0, Math.round(ratio)));
  };
  const over = (): boolean => draftCents() > 0n && spent() > draftCents();

  async function save(): Promise<void> {
    const cents = draftCents();
    if (cents < 0n) return;
    setBusy(true);
    try {
      if (props.row.budget_id) {
        await budgets.update(props.row.budget_id, centsToApi(cents));
      } else {
        await budgets.upsert({
          category_id: props.row.category_id,
          month: props.month,
          estimated_amount: centsToApi(cents),
          currency: props.currency,
        });
      }
      pushToast(props.t("budgets.saved", { name: props.row.category_name }), "ok");
      props.onSaved();
    } catch (err) {
      if (err instanceof ApiError) {
        pushToast(`${err.code}: ${err.message}`, "error");
      } else {
        pushToast(props.t("budgets.saveFailed"), "error");
      }
    } finally {
      setBusy(false);
    }
  }

  const signed = () => formatSigned(previewDiff(), props.currency, props.locale);

  // Prefer the category's own icon/color; fall back to the seed defaults so
  // fresh seed categories (not yet backfilled via the Categories page) still
  // show a curated tile instead of a bare monogram.
  const seed = (): boolean => isSeedName(props.row.category_name);
  const icon = (): string | null =>
    props.row.icon ?? (seed() ? defaultIconForSeed(props.row.category_name) : null);
  const color = (): string | null =>
    props.row.color ?? (seed() ? defaultColorForSeed(props.row.category_name) : null);

  return (
    <li class="flex flex-col gap-3 px-4 py-4">
      <div class="flex items-center justify-between gap-3">
        <div class="flex min-w-0 items-center gap-3">
          <CategoryThumb
            icon={icon()}
            color={color()}
            fallback={props.displayName.trim()[0]?.toUpperCase() ?? "?"}
          />
          <span class="body-strong truncate">{props.displayName}</span>
        </div>
        <span
          class="caption tabular"
          classList={{
            "money-overdue": signed().tone === "neg",
            "money-income": signed().tone === "pos",
          }}
        >
          {signed().text}
        </span>
      </div>
      <div class="flex flex-wrap items-end gap-3">
        <label class="min-w-[180px] flex-1">
          <span class="label">{props.t("budgets.estimated")}</span>
          <input
            class="field num"
            type="text"
            inputmode="decimal"
            placeholder="0,00"
            value={input()}
            onInput={(e) => setInput(e.currentTarget.value)}
          />
        </label>
        <div class="num min-w-[160px] text-sm">
          <div class="label">{props.t("budgets.spent")}</div>
          <div>{formatMoney(spent(), props.currency, props.locale)}</div>
        </div>
        <button
          type="button"
          class="btn btn-primary"
          disabled={busy()}
          onClick={() => void save()}
        >
          {busy() ? "…" : props.t("budgets.save")}
        </button>
      </div>
      <div class="gauge" classList={{ "is-over": over() }}>
        <span style={{ width: `${pct()}%` }} />
      </div>
    </li>
  );
}
