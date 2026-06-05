import { For, type JSX } from "solid-js";
import { A } from "@solidjs/router";
import { useI18n } from "../lib/i18n";

interface Entry {
  href: string;
  icon: string;
  titleKey: string;
  hintKey: string;
}

const ENTRIES: Entry[] = [
  { href: "/budgets", icon: "savings", titleKey: "settings.budgets", hintKey: "settings.budgetsHint" },
  { href: "/categories", icon: "sell", titleKey: "settings.categories", hintKey: "settings.categoriesHint" },
  { href: "/profile", icon: "person", titleKey: "settings.profile", hintKey: "settings.profileHint" },
];

export default function Settings(): JSX.Element {
  const { t } = useI18n();
  return (
    <div class="mx-auto w-full max-w-3xl space-y-4 px-4 py-5">
      <h2 class="h-title">{t("settings.title")}</h2>
      <ul class="card divide-y divide-[color:var(--color-surface-sunken)]">
        <For each={ENTRIES}>
          {(e) => (
            <li>
              <A
                href={e.href}
                class="flex items-center gap-4 px-4 py-4 hover:bg-[color:var(--color-surface-muted)]"
              >
                <span class="tx-icon" aria-hidden>
                  <span class="material-symbols-rounded" style={{ "font-size": "20px" }}>{e.icon}</span>
                </span>
                <div class="min-w-0 flex-1">
                  <div class="body-strong">{t(e.titleKey)}</div>
                  <div class="meta">{t(e.hintKey)}</div>
                </div>
                <span class="material-symbols-rounded text-[color:var(--color-text-muted)]">chevron_right</span>
              </A>
            </li>
          )}
        </For>
      </ul>
    </div>
  );
}
