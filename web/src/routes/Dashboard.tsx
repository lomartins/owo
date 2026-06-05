import { createResource, For, Show, type JSX } from "solid-js";
import { useMonth } from "../lib/useMonth";
import { budgets, categories as categoriesApi, reports } from "../api";
import { formatMoney, formatSigned, parseCents } from "../lib/money";
import { EmptyState } from "../components/EmptyState";
import { Avatar } from "../components/Avatar";
import { CategoryThumb } from "../components/CategoryThumb";
import { NetWorthPanel } from "../components/NetWorthPanel";
import { monthLabel } from "../lib/month";
import { currentUser } from "../lib/session";
import type { BudgetRow, Category } from "../api/types";
import { A } from "@solidjs/router";
import { useI18n } from "../lib/i18n";
import { localizeCategoryName, isSeedName } from "../lib/categories";

export default function Dashboard(): JSX.Element {
  const { month, currency, locale } = useMonth();
  const { t, lang } = useI18n();

  const greeting = (): string => {
    const u = currentUser();
    if (!u) return "";
    const name = u.display_name?.trim() || "";
    const partner = u.partner_name?.trim() || "";
    return partner
      ? t("dashboard.greetingPair", { name, partner })
      : t("dashboard.greetingSolo", { name });
  };

  const [report] = createResource(month, (m) => reports.monthly(m));
  const [budgetMonth] = createResource(month, (m) => budgets.list(m));
  const [cats] = createResource(() => categoriesApi.list());
  const [spend] = createResource(month, (m) => reports.spendable(m).catch(() => null));

  const catById = (): Map<string, Category> => {
    const m = new Map<string, Category>();
    for (const c of cats() ?? []) m.set(c.id, c);
    return m;
  };

  return (
    <div class="mx-auto w-full max-w-4xl space-y-6 px-4 py-5">
      <Show when={currentUser()}>
        <section class="card-tinted flex items-center gap-4 p-4 sm:p-5 enter">
          <Avatar user={currentUser()} size="md" />
          <div class="min-w-0 flex-1">
            <p class="h-section truncate">{greeting()}</p>
            <p class="meta">{t("dashboard.monthHere", { month: monthLabel(month(), locale()) })}</p>
          </div>
        </section>
      </Show>

      <NetWorthPanel />

      <Show when={spend()}>
        {(s) => (
          <section class="card-tinted p-4 sm:p-5 enter">
            <div class="flex items-baseline justify-between gap-3">
              <span class="eyebrow">{t("dashboard.spendable")}</span>
              <span class="meta">{t("dashboard.spendableHint")}</span>
            </div>
            <p
              class="money-display tabular mt-1 text-[28px]"
              classList={{ "money-overdue": parseCents(s().spendable) < 0n }}
            >
              {formatMoney(parseCents(s().spendable), currency(), locale())}
            </p>
            <div class="meta tabular mt-2 flex flex-wrap gap-x-3 gap-y-1">
              <span>{t("dashboard.spendAssets")}: {formatMoney(parseCents(s().asset_total), currency(), locale())}</span>
              <span>− {t("dashboard.spendCards")}: {formatMoney(parseCents(s().card_outstanding), currency(), locale())}</span>
              <span>− {t("dashboard.spendBills")}: {formatMoney(parseCents(s().pending_bills), currency(), locale())}</span>
            </div>
          </section>
        )}
      </Show>

      <section class="grid grid-cols-2 gap-3 stagger-enter">
        <Show
          when={report()}
          fallback={
            <For each={Array(4).fill(0)}>
              {() => <div class="card h-24 pulse-soft bg-[color:var(--color-surface-muted)]" />}
            </For>
          }
        >
          {(rep) => (
            <>
              <SummaryCard
                label={t("dashboard.income")}
                value={formatMoney(parseCents(rep().income_total), currency(), locale())}
                tone="brand"
              />
              <SummaryCard
                label={t("dashboard.spent")}
                value={formatMoney(parseCents(rep().spent_total), currency(), locale())}
                tone="muted"
              />
              <SummaryCard
                label={t("dashboard.budgetBalance")}
                value={formatMoney(parseCents(rep().budget_balance), currency(), locale())}
                signed={parseCents(rep().budget_balance)}
                currency={currency()}
                locale={locale()}
              />
              <SummaryCard
                label={t("dashboard.carryOver")}
                value={formatSigned(
                  parseCents(rep().carry_over_out) - parseCents(rep().carry_over_in),
                  currency(),
                  locale(),
                ).text}
                tone="muted"
              />
            </>
          )}
        </Show>
      </section>

      <section>
        <div class="mb-3 flex items-baseline justify-between px-1">
          <h2 class="h-section">{t("dashboard.byCategory")}</h2>
          <A href="/budgets" class="meta text-[color:var(--color-primary-600)]">
            {t("dashboard.editBudgets")}
          </A>
        </div>
        <Show
          when={budgetMonth()}
          fallback={<div class="card h-48 animate-pulse bg-[color:var(--color-bone-100)]" />}
        >
          {(bm) => (
            <Show
              when={bm().items.length > 0}
              fallback={
                <EmptyState
                  icon="category"
                  title={t("dashboard.emptyCategories")}
                  hint={t("dashboard.emptyCategoriesHint")}
                />
              }
            >
              <ul class="card divide-y divide-[color:var(--color-surface-sunken)] stagger-enter">
                <For each={bm().items}>{(row) => {
                  const cat = catById().get(row.category_id);
                  return <CategoryRow row={row} currency={currency()} locale={locale()} lang={lang()} icon={cat?.icon ?? null} color={cat?.color ?? null} />;
                }}</For>
              </ul>
            </Show>
          )}
        </Show>
      </section>
    </div>
  );
}

function SummaryCard(props: {
  label: string;
  value: string;
  tone?: "brand" | "muted" | "default";
  signed?: bigint;
  currency?: string;
  locale?: string;
}): JSX.Element {
  return (
    <div
      class="card flex flex-col gap-1 p-4"
      classList={{
        "surface-brand": props.tone === "brand",
      }}
    >
      <span class="eyebrow" classList={{ "text-white/80": props.tone === "brand" }}>
        {props.label}
      </span>
      <span
        class="money tabular text-[22px]"
        classList={{
          "money-overdue": props.signed !== undefined && props.signed < 0n,
          "text-white": props.tone === "brand",
        }}
      >
        {props.signed !== undefined
          ? formatSigned(props.signed, props.currency ?? "BRL", props.locale ?? "pt-BR").text
          : props.value}
      </span>
    </div>
  );
}

function CategoryRow(props: { row: BudgetRow; currency: string; locale: string; lang: "en" | "pt"; icon?: string | null; color?: string | null }): JSX.Element {
  const estimated = (): bigint => parseCents(props.row.estimated);
  const spent = (): bigint => parseCents(props.row.spent);
  const diff = (): bigint => parseCents(props.row.difference);

  const pct = (): number => {
    const e = estimated();
    if (e <= 0n) return spent() > 0n ? 100 : 0;
    const ratio = Number(spent() * 100n) / Number(e);
    return Math.min(100, Math.max(0, Math.round(ratio)));
  };
  const over = (): boolean => estimated() > 0n && spent() > estimated();
  const signed = () => formatSigned(diff(), props.currency, props.locale);
  const label = (): string => isSeedName(props.row.category_name) ? localizeCategoryName(props.row.category_name, props.lang) : props.row.category_name;

  return (
    <li class="flex items-center gap-4 px-4 py-3">
      <CategoryThumb
        icon={props.icon ?? null}
        color={props.color ?? null}
        fallback={label().trim()[0]?.toUpperCase() ?? "?"}
        size={16}
        glyphSize={12}
      />
      <div class="min-w-0 flex-1">
        <div class="flex items-baseline justify-between gap-3">
          <span class="body-strong truncate">{label()}</span>
          <span class="meta tabular">
            {formatMoney(spent(), props.currency, props.locale)} /{" "}
            <span class="body-strong">
              {formatMoney(estimated(), props.currency, props.locale)}
            </span>
          </span>
        </div>
        <div class="mt-2 flex items-center gap-3">
          <div class="gauge flex-1" classList={{ "is-over": over() }}>
            <span style={{ width: `${pct()}%` }} />
          </div>
          <span
            class="caption tabular min-w-[5.5rem] text-right"
            classList={{
              "money-overdue": signed().tone === "neg",
              "money-income": signed().tone === "pos",
            }}
          >
            {signed().text}
          </span>
        </div>
      </div>
    </li>
  );
}
