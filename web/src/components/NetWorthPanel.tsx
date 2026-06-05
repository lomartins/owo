import { createResource, For, Show, type JSX } from "solid-js";
import { reports } from "../api";
import { useMonth } from "../lib/useMonth";
import { formatMoney, parseCents } from "../lib/money";
import { useI18n } from "../lib/i18n";

/**
 * Dark hero card matching the mobile UI kit:
 *   NET WORTH eyebrow · big amount · % change pill · 7 month-bars.
 */
export function NetWorthPanel(): JSX.Element {
  const { month, currency, locale } = useMonth();
  const { t } = useI18n();

  const [report] = createResource(
    () => month(),
    (m) => reports.netWorth({ to: m, months: 7 }).catch(() => null),
  );

  const monthAbbrev = (m: string): string => {
    const [y, mo] = m.split("-").map((p) => parseInt(p, 10));
    const d = new Date(y, mo - 1, 1);
    return new Intl.DateTimeFormat(locale(), { month: "short" }).format(d).replace(".", "");
  };

  return (
    <Show
      when={report()}
      fallback={<div class="h-44 rounded-2xl pulse-soft bg-[color:var(--color-neutral-800)]/30" />}
    >
      {(rep) => {
        // Read the resource accessor reactively — destructuring `rep()` once
        // here would freeze the panel on the first month (Show's child fn runs
        // only when `when` flips truthy, not on every value change).
        const points = () => rep().points;
        const peak = () =>
          points().reduce((m, p) => {
            const v = Math.abs(Number(parseCents(p.net_worth)));
            return v > m ? v : m;
          }, 1);
        const currentMonth = () => points()[points().length - 1]?.month;
        const changeNum = (): number | null => {
          const c = rep().change_pct;
          return c != null ? Number(c) : null;
        };
        const changeText = (): string | null => {
          const n = changeNum();
          return n != null ? `${n >= 0 ? "+" : ""}${n.toFixed(1).replace(".", ",")}%` : null;
        };

        return (
          <section
            class="enter rounded-2xl p-5 text-[color:var(--color-text-on-primary)] shadow-[var(--shadow-card)]"
            style={{ background: "var(--color-neutral-800)" }}
          >
            <div class="flex items-start justify-between gap-3">
              <div>
                <div class="eyebrow !text-white/60">{t("netWorth.label")}</div>
                <div class="money-display mt-1 !text-white">
                  {formatMoney(parseCents(rep().current), currency(), locale())}
                </div>
              </div>
              <Show when={changeText()}>
                {(txt) => (
                  <span
                    class="pill tabular self-start"
                    classList={{
                      "!bg-[color:var(--color-success-bg)] !text-[color:var(--color-success-fg)]": (changeNum() ?? 0) >= 0,
                      "!bg-[color:var(--color-danger-bg)] !text-[color:var(--color-danger-fg)]": (changeNum() ?? 0) < 0,
                    }}
                  >
                    {txt()}
                  </span>
                )}
              </Show>
            </div>

            <div class="mt-5">
              <div class="flex h-20 items-end gap-1.5">
                <For each={points()}>
                  {(p) => {
                    const v = Math.abs(Number(parseCents(p.net_worth)));
                    const pct = Math.max(8, Math.round((v / peak()) * 100));
                    const isCurrent = p.month === currentMonth();
                    return (
                      <div
                        class="flex-1 rounded-md transition-[height] duration-500"
                        style={{
                          height: `${pct}%`,
                          background: isCurrent ? "var(--color-primary-600)" : "rgba(255,255,255,0.12)",
                        }}
                      />
                    );
                  }}
                </For>
              </div>
              <div class="mt-1.5 flex gap-1.5">
                <For each={points()}>
                  {(p) => (
                    <div
                      class="flex-1 text-center text-[10px]"
                      classList={{
                        "text-white": p.month === currentMonth(),
                        "text-white/45": p.month !== currentMonth(),
                      }}
                    >
                      {monthAbbrev(p.month)}
                    </div>
                  )}
                </For>
              </div>
            </div>
          </section>
        );
      }}
    </Show>
  );
}
