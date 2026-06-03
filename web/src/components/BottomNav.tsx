import { createSignal, For, Show, type JSX } from "solid-js";
import { A, useNavigate } from "@solidjs/router";
import { useI18n } from "../lib/i18n";

interface Tab {
  href: string;
  labelKey: string;
  icon: string;
}

export const PRIMARY_TABS: Tab[] = [
  { href: "/",             labelKey: "nav.dashboard",    icon: "home" },
  { href: "/transactions", labelKey: "nav.transactions", icon: "receipt_long" },
  { href: "/bills",        labelKey: "nav.bills",        icon: "event_repeat" },
];

/** Ordered list of paths eligible for horizontal swipe navigation. Source of truth. */
export const SWIPE_TABS: readonly string[] = PRIMARY_TABS.map((t) => t.href);

const MORE_TABS: Tab[] = [
  { href: "/accounts",   labelKey: "nav.accounts",   icon: "account_balance_wallet" },
  { href: "/budgets",    labelKey: "nav.budgets",    icon: "savings" },
  { href: "/categories", labelKey: "nav.categories", icon: "sell" },
  { href: "/profile",    labelKey: "profile.title",  icon: "person" },
];

export function BottomNav(): JSX.Element {
  const { t } = useI18n();
  const navigate = useNavigate();
  const [moreOpen, setMoreOpen] = createSignal(false);

  return (
    <>
      <Show when={moreOpen()}>
        <button
          type="button"
          class="fixed inset-0 z-30 bg-black/30 backdrop-blur-[2px] enter-fade md:hidden"
          aria-label={t("common.cancel")}
          onClick={() => setMoreOpen(false)}
        />
        <div
          class="fixed bottom-[78px] left-1/2 z-40 w-[min(92vw,360px)] -translate-x-1/2 card p-2 shadow-[var(--shadow-sheet)] enter md:hidden"
          role="menu"
        >
          <For each={MORE_TABS}>
            {(tab) => (
              <A
                href={tab.href}
                class="flex items-center gap-3 rounded-md px-3 py-3 text-[color:var(--color-text-primary)] hover:bg-[color:var(--color-surface-muted)]"
                activeClass="!text-[color:var(--color-primary-600)] !bg-[color:var(--color-primary-50)]"
                onClick={() => setMoreOpen(false)}
              >
                <span class="material-symbols-rounded" style={{ "font-size": "20px" }}>
                  {tab.icon}
                </span>
                <span class="body-strong">{t(tab.labelKey)}</span>
              </A>
            )}
          </For>
        </div>
      </Show>

      <nav
        aria-label="primary"
        class="fixed bottom-0 left-0 right-0 z-30 border-t border-[color:rgba(31,27,23,0.08)] bg-[color:var(--color-surface)]/95 backdrop-blur safe-bottom md:hidden"
        style={{ "padding-top": "6px" }}
      >
        <div class="relative mx-auto grid max-w-md grid-cols-5 items-end px-2">
          <NavTab tab={PRIMARY_TABS[0]} t={t} />
          <NavTab tab={PRIMARY_TABS[1]} t={t} />

          {/* Center FAB: floats above the bar, primary action. */}
          <div class="flex justify-center">
            <button
              type="button"
              class="fab -translate-y-3 transition active:scale-95"
              aria-label={t("nav.add")}
              onClick={() => navigate("/add")}
            >
              <span class="material-symbols-rounded" style={{ "font-size": "28px" }}>add</span>
            </button>
          </div>

          <NavTab tab={PRIMARY_TABS[2]} t={t} />
          <button
            type="button"
            class="flex flex-col items-center gap-0.5 rounded-md px-1 py-2 text-[10px] font-medium text-[color:var(--color-text-muted)] transition hover:text-[color:var(--color-text-primary)]"
            aria-pressed={moreOpen()}
            classList={{ "!text-[color:var(--color-primary-600)]": moreOpen() }}
            onClick={() => setMoreOpen((s) => !s)}
          >
            <span class="material-symbols-rounded" style={{ "font-size": "22px" }}>more_horiz</span>
            <span>{t("nav.more")}</span>
          </button>
        </div>
      </nav>
    </>
  );
}

function NavTab(props: { tab: Tab; t: (k: string) => string }): JSX.Element {
  return (
    <A
      href={props.tab.href}
      end={props.tab.href === "/"}
      class="flex flex-col items-center gap-0.5 rounded-md px-1 py-2 text-[10px] font-medium text-[color:var(--color-text-muted)] transition"
      activeClass="!text-[color:var(--color-primary-600)]"
    >
      <span class="material-symbols-rounded" style={{ "font-size": "22px" }}>{props.tab.icon}</span>
      <span class="truncate">{props.t(props.tab.labelKey)}</span>
    </A>
  );
}
