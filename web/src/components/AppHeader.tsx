import { type JSX, Show } from "solid-js";
import { A } from "@solidjs/router";
import { Cowrie } from "./Cowrie";
import { MonthSwitcher } from "./MonthSwitcher";
import { LangToggle } from "./LangToggle";
import { Avatar } from "./Avatar";
import type { MonthlyReport } from "../api/types";
import { currentUser, logoutSession } from "../lib/session";
import { useI18n } from "../lib/i18n";

interface Props {
  month: string;
  onMonthChange: (m: string) => void;
  report?: MonthlyReport | null;
}

export function AppHeader(props: Props): JSX.Element {
  const { t } = useI18n();
  return (
    <header class="app-bar safe-top sticky top-0 z-20">
      <div class="mx-auto flex w-full max-w-4xl items-center justify-between gap-3 px-4 pt-3">
        <A href="/" class="flex items-center gap-2">
          <Cowrie class="h-8 w-8 rounded-sm" />
          <span class="h-appbar">{t("app.title")}</span>
        </A>
        <Show when={currentUser()}>
          {(u) => (
            <div class="flex items-center gap-3">
              <LangToggle />
              <A href="/profile" class="flex items-center gap-2 rounded-full p-1 hover:bg-[color:var(--color-surface-muted)]" aria-label={t("profile.title")}>
                <Avatar user={u()} size="sm" />
                <div class="hidden text-right leading-tight sm:block">
                  <div class="body-strong">{u().display_name}</div>
                  <Show when={u().partner_name}>
                    {(p) => <div class="meta">& {p()}</div>}
                  </Show>
                </div>
              </A>
              <button
                type="button"
                class="meta rounded-md px-3 py-1.5 hover:bg-[color:var(--color-surface-muted)]"
                onClick={() => void logoutSession()}
              >
                {t("app.signOut")}
              </button>
            </div>
          )}
        </Show>
      </div>
      <MonthSwitcher month={props.month} onChange={props.onMonthChange} report={props.report} />
      <nav class="mx-auto hidden w-full max-w-5xl gap-1 px-2 pb-2 text-sm md:flex">
        <NavLink href="/" label={t("nav.dashboard")} />
        <NavLink href="/transactions" label={t("nav.transactions")} />
        <NavLink href="/bills" label={t("nav.bills")} />
        <NavLink href="/accounts" label={t("nav.accounts")} />
        <NavLink href="/add" label={t("nav.add")} emphasis />
        <NavLink href="/budgets" label={t("nav.budgets")} />
        <NavLink href="/categories" label={t("nav.categories")} />
      </nav>
    </header>
  );
}

function NavLink(props: { href: string; label: string; emphasis?: boolean }): JSX.Element {
  return (
    <A
      href={props.href}
      end={props.href === "/"}
      class="flex-1 rounded-md px-3 py-2 text-center font-medium text-[color:var(--color-text-muted)] transition hover:bg-[color:var(--color-surface-muted)] hover:text-[color:var(--color-text-primary)]"
      activeClass="!text-[color:var(--color-primary-600)] !bg-[color:var(--color-primary-50)]"
    >
      <Show when={props.emphasis} fallback={props.label}>
        <span class="inline-flex items-center gap-1.5">
          <span class="material-symbols-rounded" style={{ "font-size": "18px" }}>add</span>
          {props.label}
        </span>
      </Show>
    </A>
  );
}
