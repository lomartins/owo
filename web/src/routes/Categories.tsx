import { createEffect, createResource, createSignal, For, Show, type JSX } from "solid-js";
import { categories } from "../api";
import { useI18n } from "../lib/i18n";
import { localizeCategoryName, isSeedName } from "../lib/categories";
import {
  COLOR_PALETTE,
  defaultColorForSeed,
  defaultIconForSeed,
} from "../lib/categoryIcons";
import { IconPicker } from "../components/IconPicker";
import { CategoryThumb } from "../components/CategoryThumb";
import { pushToast } from "../components/Toast";
import { EmptyState } from "../components/EmptyState";
import { ApiError } from "../api/client";
import type { Category } from "../api/types";

export default function CategoriesView(): JSX.Element {
  const { t, lang } = useI18n();
  const [list, { refetch }] = createResource(() => categories.list());
  const [showForm, setShowForm] = createSignal(false);
  let seededOnce = false;

  // First time we receive a list, backfill icon/color for every seed-named
  // category that's missing them. One batch, one refetch.
  createEffect(() => {
    const items = list();
    if (!items || seededOnce) return;
    seededOnce = true;
    const missing = items.filter(
      (c) => c.icon == null && isSeedName(c.name) && defaultIconForSeed(c.name),
    );
    if (missing.length === 0) return;
    void Promise.all(
      missing.map((c) =>
        categories.update(c.id, {
          icon: defaultIconForSeed(c.name)!,
          color: defaultColorForSeed(c.name),
        }),
      ),
    )
      .then(() => refetch())
      .catch(() => {
        // Quietly ignore — page is still usable without defaults.
      });
  });

  return (
    <div class="mx-auto w-full max-w-2xl space-y-4 px-4 py-5">
      <header class="flex items-center justify-between gap-3">
        <h2 class="h-title">{t("categoriesPage.title")}</h2>
        <button
          type="button"
          class="btn btn-primary"
          onClick={() => setShowForm((s) => !s)}
        >
          <span class="material-symbols-rounded" style={{ "font-size": "20px" }}>add</span>
          {t("categoriesPage.addCta")}
        </button>
      </header>

      <p class="meta">{t("categoriesPage.seedHint")}</p>

      <Show when={showForm()}>
        <NewCategoryForm
          onCreated={() => {
            setShowForm(false);
            refetch();
          }}
        />
      </Show>

      <Show
        when={list()}
        fallback={<div class="card h-32 pulse-soft bg-[color:var(--color-surface-muted)]" />}
      >
        <Show
          when={(list() ?? []).filter((c) => !c.archived).length > 0}
          fallback={<EmptyState icon="category" title={t("categoriesPage.empty")} />}
        >
          <ul class="card divide-y divide-[color:var(--color-surface-sunken)] stagger-enter">
            <For each={(list() ?? []).filter((c) => !c.archived)}>
              {(c) => (
                <CategoryRow
                  category={c}
                  lang={lang()}
                  onChanged={() => refetch()}
                />
              )}
            </For>
          </ul>
        </Show>
      </Show>
    </div>
  );
}

function CategoryRow(props: {
  category: Category;
  lang: "en" | "pt";
  onChanged: () => void;
}): JSX.Element {
  const { t } = useI18n();
  // Display name: if the DB name is a seed (English canonical), localize it.
  // Once the user renames, the DB stores the user's preferred name as-is.
  const displayName = (): string =>
    isSeedName(props.category.name)
      ? localizeCategoryName(props.category.name, props.lang)
      : props.category.name;

  const [editing, setEditing] = createSignal(false);
  const [draft, setDraft] = createSignal(displayName());
  const [icon, setIcon] = createSignal<string | null>(props.category.icon);
  const [color, setColor] = createSignal<string | null>(props.category.color);
  const [busy, setBusy] = createSignal(false);

  async function save(): Promise<void> {
    const next = draft().trim();
    const nameChanged = next && next !== displayName();
    const iconChanged = icon() !== props.category.icon;
    const colorChanged = color() !== props.category.color;
    if (!nameChanged && !iconChanged && !colorChanged) {
      setEditing(false);
      return;
    }
    if (!next) {
      setEditing(false);
      return;
    }
    setBusy(true);
    try {
      await categories.update(props.category.id, {
        name: next,
        icon: icon(),
        color: color(),
      });
      pushToast(t("categoriesPage.saved"), "ok");
      setEditing(false);
      props.onChanged();
    } catch (err) {
      pushToast(err instanceof ApiError ? err.code : "error", "error");
    } finally {
      setBusy(false);
    }
  }

  async function archive(): Promise<void> {
    if (!window.confirm(t("categoriesPage.confirmDelete", { name: displayName() }))) return;
    setBusy(true);
    try {
      await categories.update(props.category.id, { archived: true });
      props.onChanged();
    } catch (err) {
      pushToast(err instanceof ApiError ? err.code : "error", "error");
    } finally {
      setBusy(false);
    }
  }

  return (
    <li class="flex flex-col gap-3 px-4 py-3">
      <div class="flex items-center gap-3">
        <CategoryThumb
          icon={props.category.icon}
          color={props.category.color}
          fallback={displayName().trim()[0]?.toUpperCase() ?? "?"}
        />
        <Show
          when={editing()}
          fallback={
            <>
              <span class="body-strong min-w-0 flex-1 truncate">{displayName()}</span>
              <button
                type="button"
                class="icon-btn"
                aria-label={t("common.save")}
                onClick={() => {
                  setDraft(displayName());
                  setIcon(props.category.icon);
                  setColor(props.category.color);
                  setEditing(true);
                }}
              >
                <span class="material-symbols-rounded" style={{ "font-size": "18px" }}>edit</span>
              </button>
              <button
                type="button"
                class="icon-btn"
                aria-label={t("common.delete")}
                disabled={busy()}
                onClick={() => void archive()}
              >
                <span class="material-symbols-rounded" style={{ "font-size": "18px" }}>delete</span>
              </button>
            </>
          }
        >
          <input
            class="field min-w-0 flex-1"
            value={draft()}
            onInput={(e) => setDraft(e.currentTarget.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter") void save();
              if (e.key === "Escape") setEditing(false);
            }}
            autofocus
          />
          <button
            type="button"
            class="btn btn-primary"
            disabled={busy()}
            onClick={() => void save()}
          >
            {busy() ? t("categoriesPage.saving") : t("categoriesPage.save")}
          </button>
          <button
            type="button"
            class="icon-btn"
            aria-label={t("common.cancel")}
            onClick={() => setEditing(false)}
          >
            <span class="material-symbols-rounded" style={{ "font-size": "18px" }}>close</span>
          </button>
        </Show>
      </div>
      <Show when={editing()}>
        <div class="card-muted p-4">
          <IconPicker
            value={icon()}
            color={color()}
            onChange={(nextIcon, nextColor) => {
              setIcon(nextIcon);
              setColor(nextColor);
            }}
          />
        </div>
      </Show>
    </li>
  );
}

function NewCategoryForm(props: { onCreated: () => void }): JSX.Element {
  const { t } = useI18n();
  const [name, setName] = createSignal("");
  const [kind, setKind] = createSignal<"EXPENSE" | "INCOME" | "BOTH">("EXPENSE");
  const [icon, setIcon] = createSignal<string>("sell");
  const [color, setColor] = createSignal<string>(COLOR_PALETTE[0]!);
  const [busy, setBusy] = createSignal(false);

  async function submit(e: Event): Promise<void> {
    e.preventDefault();
    if (!name().trim()) return;
    setBusy(true);
    try {
      const created = await categories.create({ name: name().trim(), kind: kind() });
      // Backend create endpoint doesn't accept icon/color; PATCH right after.
      await categories.update(created.id, { icon: icon(), color: color() });
      setName("");
      props.onCreated();
    } catch (err) {
      pushToast(err instanceof ApiError ? err.code : "error", "error");
    } finally {
      setBusy(false);
    }
  }

  return (
    <form class="card flex flex-col gap-4 p-5" onSubmit={submit}>
      <label>
        <span class="label">{t("categoriesPage.name")}</span>
        <input
          class="field"
          value={name()}
          onInput={(e) => setName(e.currentTarget.value)}
          placeholder={t("categoriesPage.namePlaceholder")}
        />
      </label>
      <label>
        <span class="label">{t("categoriesPage.kind")}</span>
        <div class="chip-row">
          {(["EXPENSE", "INCOME", "BOTH"] as const).map((k) => (
            <button
              type="button"
              class="chip"
              aria-pressed={kind() === k}
              classList={{ "is-active": kind() === k }}
              onClick={() => setKind(k)}
            >
              {k === "EXPENSE"
                ? t("categoriesPage.kindExpense")
                : k === "INCOME"
                  ? t("categoriesPage.kindIncome")
                  : t("categoriesPage.kindBoth")}
            </button>
          ))}
        </div>
      </label>
      <div class="card-muted p-4">
        <IconPicker
          value={icon()}
          color={color()}
          onChange={(nextIcon, nextColor) => {
            setIcon(nextIcon);
            setColor(nextColor);
          }}
        />
      </div>
      <button class="btn btn-primary self-start" type="submit" disabled={busy()}>
        {busy() ? t("categoriesPage.saving") : t("categoriesPage.save")}
      </button>
    </form>
  );
}

