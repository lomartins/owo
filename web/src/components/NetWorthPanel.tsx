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
        const r = rep();
        const points = r.points;
        const peak = points.reduce((m, p) => {
          const v = Math.abs(Number(parseCents(p.net_worth)));
          return v > m ? v : m;
        }, 1);
        const currentMonth = points[points.length - 1]?.month;
        const change = r.change_pct;
        const changeNum = change != null ? Number(change) : null;
        const changeText = change != null
          ? `${changeNum != null && changeNum >= 0 ? "+" : ""}${(changeNum ?? 0).toFixed(1).replace(".", ",")}%`
          : null;

        return (
          <section
            class="enter rounded-2xl p-5 text-[color:var(--color-text-on-primary)] shadow-[var(--shadow-card)]"
            style={{ background: "var(--color-neutral-800)" }}
          >
            <div class="flex items-start justify-between gap-3">
              <div>
                <div class="eyebrow !text-white/60">{t("netWorth.label")}</div>
                <div class="money-display mt-1 !text-white">
                  {formatMoney(parseCents(r.current), currency(), locale())}
                </div>
              </div>
              <Show when={changeText}>
                {(txt) => (
                  <span
                    class="pill tabular self-start"
                    classList={{
                      "!bg-[color:var(--color-success-bg)] !text-[color:var(--color-success-fg)]": (changeNum ?? 0) >= 0,
                      "!bg-[color:var(--color-danger-bg)] !text-[color:var(--color-danger-fg)]": (changeNum ?? 0) < 0,
                    }}
                  >
                    {txt()}
                  </span>
                )}
              </Show>
            </div>

            <div class="mt-5 flex h-20 items-end gap-1.5">
              <For each={points}>
                {(p) => {
                  const v = Math.abs(Number(parseCents(p.net_worth)));
                  const pct = Math.max(8, Math.round((v / peak) * 100));
                  const isCurrent = p.month === currentMonth;
                  return (
                    <div class="flex flex-1 flex-col items-center gap-1.5">
                      <div
                        class="w-full rounded-md transition-[height] duration-500"
                        style={{
                          height: `${pct}%`,
                          background: isCurrent ? "var(--color-primary-600)" : "rgba(255,255,255,0.12)",
                        }}
                      />
                      <div
                        class="text-[10px]"
                        classList={{
                          "text-white": isCurrent,
                          "text-white/45": !isCurrent,
                        }}
                      >
                        {monthAbbrev(p.month)}
                      </div>
                    </div>
                  );
                }}
              </For>
            </div>
          </section>
        );
      }}
    </Show>
  );
}
