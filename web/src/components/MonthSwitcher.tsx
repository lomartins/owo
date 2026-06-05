import { type JSX, Show } from "solid-js";
import { monthLabel, shiftMonth } from "../lib/month";
import { formatSigned, parseCents } from "../lib/money";
import type { MonthlyReport } from "../api/types";
import { useI18n } from "../lib/i18n";

interface Props {
  month: string;
  onChange: (m: string) => void;
  report?: MonthlyReport | null;
  currency?: string;
  locale?: string;
}

/**
 * Month switcher: < current > with carry-over sub-line.
 * The current-month label is itself a button for future year-grid picker.
 */
export function MonthSwitcher(props: Props): JSX.Element {
  const { t, locale: i18nLocale } = useI18n();
  const carryIn = () => (props.report ? parseCents(props.report.carry_over_in) : 0n);
  const carryOut = () => (props.report ? parseCents(props.report.carry_over_out) : 0n);
  const flow = () => carryOut() - carryIn();

  return (
    <div class="flex flex-col items-center gap-1 px-4 py-3">
      <div class="flex items-center gap-1">
        <button
          type="button"
          class="rounded-full p-2 transition hover:bg-[color:var(--color-surface-muted)] active:scale-95"
          aria-label={t("month.prev")}
          onClick={() => props.onChange(shiftMonth(props.month, -1))}
        >
          <span class="material-symbols-rounded" style={{ "font-size": "20px" }}>chevron_left</span>
        </button>
        <button
          type="button"
          class="h-title rounded-md px-3 py-1.5 transition hover:bg-[color:var(--color-surface-muted)]"
          onClick={() => { /* year-grid picker — future */ }}
        >
          {monthLabel(props.month, props.locale ?? i18nLocale())}
        </button>
        <button
          type="button"
          class="rounded-full p-2 transition hover:bg-[color:var(--color-surface-muted)] active:scale-95"
          aria-label={t("month.next")}
          onClick={() => props.onChange(shiftMonth(props.month, 1))}
        >
          <span class="material-symbols-rounded" style={{ "font-size": "20px" }}>chevron_right</span>
        </button>
      </div>
      <Show when={props.report}>
        <div class="eyebrow">
          {t("month.carry_over")} ·{" "}
          <span class="tabular">
            {formatSigned(flow(), props.currency ?? "BRL", props.locale ?? i18nLocale()).text}
          </span>
        </div>
      </Show>
    </div>
  );
}
